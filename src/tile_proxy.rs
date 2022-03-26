use crate::binding::apache2::{
    APR_BADARG, APR_SUCCESS, OK,
    apr_status_t, request_rec, server_rec,
};
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::context::HandleContext;
use crate::schema::handler::error::HandleError;
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult, };
use crate::schema::slippy::context::{ ReadContext, WriteContext };
use crate::schema::slippy::error::{ ReadError, WriteError };
use crate::schema::slippy::result::{
    ReadOutcome, ReadRequestResult,
    WriteOutcome, WriteResponseResult,
};
use crate::interface::apache2::PoolStored;
use crate::interface::handler::{ HandleRequestObserver, RequestHandler, };
use crate::interface::slippy::{
    ReadRequestFunc, ReadRequestObserver, WriteResponseFunc, WriteResponseObserver,
};
use crate::framework::apache2::config::Loadable;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve };
use crate::framework::apache2::record::{ RequestRecord, ServerRecord, };
use crate::framework::apache2::response::Apache2Response;
use crate::implement::handler::description::DescriptionHandler;
use crate::implement::slippy::reader::SlippyRequestReader;
use crate::implement::slippy::writer::SlippyResponseWriter;
use crate::implement::storage::file_system;
use crate::implement::telemetry::metrics::cache::CacheAnalysis;
use crate::implement::telemetry::metrics::render::RenderAnalysis;
use crate::implement::telemetry::metrics::response::ResponseAnalysis;
use crate::implement::telemetry::tracing::transaction::TransactionTrace;

use chrono::Utc;

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::CString;
use std::option::Option;
use std::os::raw::{ c_int, c_void, };
use std::path::PathBuf;
use std::result::Result;
use std::time::Duration;


pub enum HandleRequestError {
    Read(ReadError),
    Handle(HandleError),
    Write(WriteError),
}

pub struct TileProxy<'p> {
    record: &'p server_rec,
    config: ModuleConfig,
    config_file_path: Option<PathBuf>,
    read_request: ReadRequestFunc,
    layer_handler: DescriptionHandler,
    write_response: WriteResponseFunc,
    cache_analysis: CacheAnalysis,
    render_analysis: RenderAnalysis,
    response_analysis: ResponseAnalysis,
    trans_trace: TransactionTrace,
    read_observers: Option<[&'p mut dyn ReadRequestObserver; 1]>,
    handle_observers: Option<[&'p mut dyn HandleRequestObserver; 3]>,
    write_observers: Option<[&'p mut dyn WriteResponseObserver; 2]>,
}

impl<'p> TileProxy<'p> {
    pub fn get_id(record: &server_rec) -> CString {
        let id = CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap();
        id
    }

    pub fn find_or_allocate_new(record: &'p mut server_rec) -> Result<&'p mut Self, Box<dyn Error>> {
        info!(record, "TileServer::find_or_create - start");
        let proxy = match retrieve(
            record.get_pool()?,
            &(Self::get_id(record))
        ) {
            Some(existing_proxy) => {
                info!(record, "TileServer::find_or_create - existing found");
                existing_proxy
            },
            None => {
                info!(record, "TileServer::find_or_create - not found");
                let module_config = ModuleConfig::new();
                Self::new(record, module_config)?
            },
        };
        info!(record, "TileServer::find_or_create - finish");
        return Ok(proxy);
    }

    pub fn new(
        record: &'p server_rec,
        module_config: ModuleConfig,
    ) -> Result<&'p mut Self, Box<dyn Error>> {
        info!(record, "TileServer::create - start");
        let new_server = alloc::<TileProxy<'p>>(
            record.get_pool()?,
            &(Self::get_id(record)),
            Some(drop_tile_server),
        )?.0;
        new_server.record = record;
        new_server.config = module_config;
        new_server.config_file_path = None;
        new_server.read_request = SlippyRequestReader::read;
        new_server.layer_handler = DescriptionHandler { };
        new_server.write_response = SlippyResponseWriter::write;
        new_server.cache_analysis = CacheAnalysis { };
        new_server.render_analysis = RenderAnalysis { };
        new_server.response_analysis = ResponseAnalysis::new();
        new_server.trans_trace = TransactionTrace { };
        new_server.read_observers = None;
        new_server.handle_observers = None;
        new_server.write_observers = None;
        info!(new_server.record, "TileServer::create - finish");
        return Ok(new_server);
    }

    pub fn load_config(
        &mut self,
        file_path: PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let original_request_timeout = self.config.renderd.render_timeout.clone();
        let server_name = self.record.get_host_name();
        let module_config = ModuleConfig::load(file_path.as_path(), server_name)?;
        self.config = module_config;
        self.config.renderd.render_timeout = original_request_timeout;
        self.config_file_path = Some(file_path.clone());
        return Ok(());
    }

    pub fn set_render_timeout(
        &mut self,
        timeout: &Duration,
    ) -> () {
        self.config.renderd.render_timeout = *timeout;
    }

    pub fn initialise(
        &mut self,
        _record: &mut server_rec,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(original_path) = &self.config_file_path {
            let copied_path = original_path.clone();
            self.load_config(copied_path)?;
        }
        file_system::initialise()?;
        return Ok(());
    }

    pub fn handle_request(
        &mut self,
        record: &mut request_rec,
    ) -> Result<c_int, ReadError> {
        debug!(record.server, "TileServer::handle_request - start");
        let (read_result, self2) = self.read_request(record);
        let (handle_result, self3) = self2.call_handlers(record, &read_result);
        let (write_result, _) = self3.write_response(record, &read_result, &handle_result);
        debug!(record.server, "TileServer::handle_request - finish");
        return Ok(OK as c_int);
    }

    fn read_request(
        &mut self,
        record: &mut request_rec,
    ) -> (ReadRequestResult, &mut Self) {
        debug!(record.server, "TileServer::read_request - start");
        let mut read_observers: [&mut dyn ReadRequestObserver; 1] = match &mut self.read_observers {
            // TODO: find a nicer way to copy self.read_observers, clone method doesn't work with trait object elements
            Some([observer_0]) => [*observer_0],
            None => [&mut self.trans_trace],
        };
        let read = self.read_request;
        let context = ReadContext {
            module_config: &self.config,
            host: VirtualHost::find_or_allocate_new(record).unwrap(),
            connection: Connection::find_or_allocate_new(record).unwrap(),
        };
        let request = Apache2Request::find_or_allocate_new(record).unwrap();
        let read_result = read(&context, request);
        for observer_iter in read_observers.iter_mut() {
            debug!(context.host.record, "TileServer::read_request - calling observer {:p}", *observer_iter);
            (*observer_iter).on_read(read, &context, &read_result);
        }
        debug!(record.server, "TileServer::read_request - finish");
        return (read_result, self);
    }

    fn call_handlers(
        &mut self,
        record: &mut request_rec,
        read_result: &ReadRequestResult,
    ) -> (HandleRequestResult, &mut Self) {
        debug!(record.server, "TileServer::call_handlers - start");
        // TODO: combine the handlers using combinators
        let mut handle_observers: [&mut dyn HandleRequestObserver; 3] = match &mut self.handle_observers {
            // TODO: find a nicer way to copy self.handle_observers, clone method doesn't work with trait object elements
            Some([observer_0, observer_1, observer_2]) => [*observer_0, *observer_1, *observer_2],
            None => [&mut self.trans_trace, &mut self.cache_analysis, &mut self.render_analysis],
        };
        let handler: &mut dyn RequestHandler = &mut self.layer_handler;
        let context = HandleContext {
            module_config: &self.config,
            host: VirtualHost::find_or_allocate_new(record).unwrap(),
            connection: Connection::find_or_allocate_new(record).unwrap(),
            request: Apache2Request::find_or_allocate_new(record).unwrap(),
            cache_metrics: &self.response_analysis,
            render_metrics: &self.response_analysis,
            response_metrics: &self.response_analysis,
        };
        let handle_result = match read_result {
            Ok(outcome) => match outcome {
                ReadOutcome::NotMatched => HandleRequestResult {
                    before_timestamp: Utc::now(),
                    after_timestamp: Utc::now(),
                    result: Ok(HandleOutcome::NotHandled),
                },
                ReadOutcome::Matched(request) => handler.handle(&context, &request),
            },
            Err(err) => HandleRequestResult {
                before_timestamp: Utc::now(),
                after_timestamp: Utc::now(),
                result: Err(HandleError::RequestNotRead((*err).clone())),
            },
        };
        for observer_iter in handle_observers.iter_mut() {
            debug!(context.host.record, "TileServer::call_handlers - calling observer {:p}", *observer_iter);
            (*observer_iter).on_handle(handler, &context, &read_result, &handle_result);
        }
        debug!(record.server, "TileServer::call_handlers - finish");
        return (handle_result, self);
    }

    fn write_response(
        &mut self,
        record: &mut request_rec,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
    ) -> (WriteResponseResult, &mut Self) {
        debug!(record.server, "TileServer::write_response - start");
        let mut write_observers: [&mut dyn WriteResponseObserver; 2] = match &mut self.write_observers {
            // TODO: find a nicer way to copy self.write_observers, clone method doesn't work with trait object elements
            Some([observer_0, observer_1]) => [*observer_0, *observer_1],
            None => [&mut self.trans_trace, &mut self.response_analysis],
        };
        let write = self.write_response;
        // Work around the borrow checker below, but its necessary since request_rec from a foreign C framework
        let write_record = record as *mut request_rec;
        let mut response = Apache2Response::from(unsafe { write_record.as_mut().unwrap() });
        let mut context = WriteContext {
            module_config: &self.config,
            host: VirtualHost::find_or_allocate_new(record).unwrap(),
            connection: Connection::find_or_allocate_new(record).unwrap(),
            response: &mut response,
        };
        let write_result = match &handle_result.result {
            Ok(outcome) => match outcome {
                HandleOutcome::NotHandled => Ok(WriteOutcome::NotWritten),
                HandleOutcome::Handled(response) => write(&mut context, &response),
            },
            Err(_) => {
                Err(WriteError::RequestNotHandled) // FIXME: propagate the HandleError properly
            },
        };
        for observer_iter in write_observers.iter_mut() {
            debug!(
                context.response.record.get_server_record().unwrap(),
                "TileServer::write_response - calling observer {:p}", *observer_iter
            );
            (*observer_iter).on_write(write, &context, &read_result, &handle_result, &write_result);
        }
        debug!(record.server, "TileServer::write_response - finish");
        return (write_result, self);
    }
}

#[no_mangle]
extern "C" fn drop_tile_server(server_void: *mut c_void) -> apr_status_t {
    let server_ref = match access_pool_object::<TileProxy>(server_void) {
        None => {
            return APR_BADARG as apr_status_t;
        },
        Some(server) => server,
    };
    info!(server_ref.record, "drop_tile_server - dropping");
    drop(server_ref);
    return APR_SUCCESS as apr_status_t;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::slippy::result::ReadOutcome;
    use crate::schema::slippy::request;
    use crate::schema::slippy::response;
    use crate::framework::apache2::record::test_utils::{ with_request_rec, with_server_rec };
    use chrono::Utc;

    #[test]
    fn test_proxy_reload() -> Result<(), Box<dyn Error>> {
        with_server_rec(|record| {
            let module_config = ModuleConfig::new();
            let proxy = TileProxy::new(record, module_config).unwrap();

            let expected_timeout = Duration::new(30, 50);
            proxy.set_render_timeout(&expected_timeout);
            let mut expected_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            expected_path.push("resources/test/tile/basic_valid.conf");
            proxy.load_config(expected_path.clone())?;

            let actual_timeout = proxy.config.renderd.render_timeout.clone();
            assert_eq!(expected_timeout, actual_timeout, "Failed to preserve request timeout during reload");
            assert!(proxy.config_file_path.is_some(), "Config file path is None");
            if let Some(actual_path) = &proxy.config_file_path {
                assert_eq!(&expected_path, actual_path, "Failed to preserve config file path during reload");
            }
            Ok(())
        })
    }

    struct MockReadObserver {
        count: u32,
    }

    impl ReadRequestObserver for MockReadObserver {
        fn on_read(
            &mut self,
            _func: ReadRequestFunc,
            _context: &ReadContext,
            _result: &ReadRequestResult
        ) -> () {
            self.count += 1;
        }
    }

    #[test]
    fn test_read_request_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let mut mock1 = MockReadObserver {
                    count: 0,
                };
                let module_config = ModuleConfig::new();
                let proxy = TileProxy::new(server, module_config).unwrap();
                proxy.read_observers = Some([&mut mock1]);
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let (result, _) = proxy.read_request(request);
                result.unwrap();
                assert_eq!(1, mock1.count, "Read observer not called");
                Ok(())
            })
        })
    }

    struct MockHandleObserver {
        count: u32,
    }

    impl HandleRequestObserver for MockHandleObserver {
        fn on_handle(
            &mut self,
            _obj: &dyn RequestHandler,
            _context: &HandleContext,
            _read_result: &ReadRequestResult,
            _handle_result: &HandleRequestResult
        ) -> () {
            self.count += 1;
        }
    }

    #[test]
    fn test_call_handlers_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let mut mock1 = MockHandleObserver {
                    count: 0,
                };
                let mut mock2 = MockHandleObserver {
                    count: 0,
                };
                let mut mock3 = MockHandleObserver {
                    count: 0,
                };
                let module_config = ModuleConfig::new();
                let proxy = TileProxy::new(server, module_config).unwrap();
                proxy.handle_observers = Some([&mut mock1, &mut mock2, &mut mock3]);
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let context = Apache2Request::create_with_tile_config(request)?;
                let connection = Connection::find_or_allocate_new(request)?;
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                context.record,
                                connection.record,
                                proxy.record,
                            ),
                            body: request::BodyVariant::ReportStatistics,
                        }
                    )
                );
                let (handle_result, _) = proxy.call_handlers(request, &read_result);
                handle_result.result.unwrap();
                assert_eq!(1, mock1.count, "Handle observer not called");
                Ok(())
            })
        })
    }

    struct MockWriteObserver {
        count: u32,
    }

    impl WriteResponseObserver for MockWriteObserver {
        fn on_write(
            &mut self,
            _func: WriteResponseFunc,
            _context: &WriteContext,
            _read_result: &ReadRequestResult,
            _handle_result: &HandleRequestResult,
            _write_result: &WriteResponseResult,
        ) -> () {
            self.count += 1;
        }
    }

    #[test]
    fn test_write_response_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let mut mock1 = MockWriteObserver {
                    count: 0,
                };
                let mut mock2 = MockWriteObserver {
                    count: 0,
                };
                let module_config = ModuleConfig::new();
                let proxy = TileProxy::new(server, module_config).unwrap();
                proxy.write_observers = Some([&mut mock1, &mut mock2]);
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let context = Apache2Request::create_with_tile_config(request)?;
                let connection = Connection::find_or_allocate_new(request)?;
                let read_result: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::SlippyRequest {
                            header: request::Header::new(
                                context.record,
                                connection.record,
                                proxy.record,
                            ),
                            body: request::BodyVariant::ReportStatistics,
                        }
                    )
                );
                let handle_result = HandleRequestResult {
                    before_timestamp: Utc::now(),
                    after_timestamp: Utc::now(),
                    result: Ok(
                        HandleOutcome::Handled(
                            response::SlippyResponse {
                                header: response::Header::new(
                                    context.record,
                                    &mime::APPLICATION_JSON,
                                ),
                                body: response::BodyVariant::Description(
                                    response::Description {
                                        tilejson: "2.0.0",
                                        schema: "xyz",
                                        name: String::new(),
                                        description: String::new(),
                                        attribution: String::new(),
                                        minzoom: 0,
                                        maxzoom: 1,
                                        tiles: Vec::new(),
                                    }
                                ),
                            }
                        )
                    ),
                };
                let (result, _) = proxy.write_response(request, &read_result, &handle_result);
                result.unwrap();
                assert_eq!(1, mock1.count, "Write observer not called");
                Ok(())
            })
        })
    }
}
