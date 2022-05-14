use crate::binding::apache2::{
    APR_BADARG, APR_SUCCESS, OK,
    apr_status_t, request_rec, server_rec,
};
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::connection::Connection;
use crate::schema::apache2::request::Apache2Request;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::error::HandleError;
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult, };
use crate::schema::slippy::error::{ ReadError, WriteError };
use crate::schema::slippy::result::{ ReadOutcome, WriteOutcome, };
use crate::interface::apache2::{ PoolStored, Writer, };
use crate::interface::handler::{
    HandleContext, HandleRequestObserver, RequestHandler,
};
use crate::interface::slippy::{
    ReadContext, ReadRequestFunc, ReadRequestObserver,
    WriteContext, WriteResponseFunc, WriteResponseObserver,
};
use crate::interface::telemetry::MetricsInventory;
use crate::framework::apache2::config::Loadable;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve };
use crate::framework::apache2::record::ServerRecord;
use crate::implement::handler::description::DescriptionHandler;
use crate::implement::handler::statistics::StatisticsHandler;
use crate::implement::slippy::reader::SlippyRequestReader;
use crate::implement::slippy::writer::SlippyResponseWriter;
use crate::implement::storage::file_system;
use crate::implement::telemetry::metrics::response::ResponseAnalysis;
use crate::implement::telemetry::metrics::tile_handling::TileHandlingAnalysis;
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
use std::string::String;
use std::time::Duration;


pub enum HandleRequestError {
    Read(ReadError),
    Handle(HandleError),
    Write(WriteError),
    Inventory(String),
}

pub struct TileProxy<'p> {
    record: &'p server_rec,
    config: ModuleConfig,
    config_file_path: Option<PathBuf>,
    read_request: ReadRequestFunc,
    write_response: WriteResponseFunc,
    description_handler: DescriptionHandler,
    response_analysis: ResponseAnalysis,
    tile_handling_analysis: TileHandlingAnalysis,
    trans_trace: TransactionTrace,
    read_observers: Option<[&'p mut dyn ReadRequestObserver; 1]>,
    handle_observers: Option<[&'p mut dyn HandleRequestObserver; 1]>,
    write_observers: Option<[&'p mut dyn WriteResponseObserver; 3]>,
}

impl<'p> TileProxy<'p> {
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

    pub fn get_id(record: &server_rec) -> CString {
        let id = CString::new(format!(
            "{}@{:p}",
            type_name::<Self>(),
            record,
        )).unwrap();
        id
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
        new_server.write_response = SlippyResponseWriter::write;
        new_server.description_handler = DescriptionHandler { };
        new_server.response_analysis = ResponseAnalysis::new();
        new_server.tile_handling_analysis = TileHandlingAnalysis::new();
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
        let (read_outcome, self2) = self.read_request(record);
        let (handle_outcome, self3) = self2.call_handlers(record, &read_outcome);
        let (write_outcome, _) = self3.write_response(record, &read_outcome, &handle_outcome);
        debug!(record.server, "TileServer::handle_request - finish");
        return Ok(OK as c_int);
    }

    fn read_request(
        &mut self,
        record: &mut request_rec,
    ) -> (ReadOutcome, &mut Self) {
        debug!(record.server, "TileServer::read_request - start");
        let read = self.read_request;
        let context = ReadContext {
            module_config: &self.config,
            host: VirtualHost::find_or_allocate_new(record).unwrap(),
            connection: Connection::find_or_allocate_new(record).unwrap(),
        };
        let request = Apache2Request::find_or_allocate_new(record).unwrap();
        let read_outcome = read(&context, request);
        let mut read_observers: [&mut dyn ReadRequestObserver; 1] = match &mut self.read_observers {
            // TODO: find a nicer way to copy self.read_observers, clone method doesn't work with trait object elements
            Some([observer_0]) => [*observer_0],
            None => [&mut self.trans_trace],
        };
        for observer_iter in read_observers.iter_mut() {
            debug!(context.host.record, "TileServer::read_request - calling observer {:p}", *observer_iter);
            (*observer_iter).on_read(read, &context, request, &read_outcome);
        }
        debug!(record.server, "TileServer::read_request - finish");
        return (read_outcome, self);
    }

    fn call_handlers(
        &mut self,
        record: &mut request_rec,
        read_outcome: &ReadOutcome,
    ) -> (HandleOutcome, &mut Self) {
        debug!(record.server, "TileServer::call_handlers - start");
        let before_timestamp = Utc::now();
        // TODO: combine the handlers using combinators
        let metrics = MetricsInventory {
            response_metrics: &(self.response_analysis),
            tile_handling_metrics: &(self.tile_handling_analysis),
        };
        let mut statistics = StatisticsHandler::new(&metrics);
        let mut handlers: Vec<&mut dyn RequestHandler> = Vec::new();
        handlers.push(&mut self.description_handler);
        handlers.push(&mut statistics);
        let context = HandleContext {
            module_config: &self.config,
            host: VirtualHost::find_or_allocate_new(record).unwrap(),
            connection: Connection::find_or_allocate_new(record).unwrap(),
            request: Apache2Request::find_or_allocate_new(record).unwrap(),
            response_metrics: &self.response_analysis,
            tile_handling_metrics: &self.tile_handling_analysis,
        };
        let outcome = match read_outcome {
            ReadOutcome::Ignored => HandleOutcome::Ignored,
            ReadOutcome::Processed(result) => match result {
                Ok(request) => {
                    let mut handle_outcome = HandleOutcome::Ignored;
                    for handler in handlers.iter_mut() {
                        let processed_outcome = (*handler).handle(&context, request);
                        if let HandleOutcome::Processed(_) = &processed_outcome {
                            let mut handle_observers: [&mut dyn HandleRequestObserver; 1] = match &mut self.handle_observers {
                                // TODO: find a nicer way to copy self.handle_observers, clone method doesn't work with trait object elements
                                Some([observer_0]) => [*observer_0],
                                None => [&mut self.trans_trace],
                            };
                            for observer_iter in handle_observers.iter_mut() {
                                debug!(context.host.record, "TileServer::call_handlers - calling observer {:p}", *observer_iter);
                                (*observer_iter).on_handle(*handler, &context, request, &handle_outcome);
                            }
                            handle_outcome = processed_outcome;
                        };
                    };
                    handle_outcome
                },
                Err(err) => {
                    HandleOutcome::Processed(
                        HandleRequestResult {
                            before_timestamp,
                            after_timestamp: Utc::now(),
                            result: Err(
                                HandleError::RequestNotRead((*err).clone())
                            ),
                        }
                    )
                },
            }
        };
        debug!(record.server, "TileServer::call_handlers - finish");
        return (outcome, self);
    }

    fn write_response(
        &mut self,
        record: &mut request_rec,
        read_outcome: &ReadOutcome,
        handle_outcome: &HandleOutcome,
    ) -> (WriteOutcome, &mut Self) {
        debug!(record.server, "TileServer::write_response - start");
        let write = self.write_response;
        // Work around the borrow checker below, but its necessary since request_rec from a foreign C framework
        let write_record = record as *mut request_rec;
        let writer: &mut dyn Writer = unsafe { write_record.as_mut().unwrap() };
        let context = WriteContext {
            module_config: &self.config,
            host: VirtualHost::find_or_allocate_new(record).unwrap(),
            connection: Connection::find_or_allocate_new(record).unwrap(),
            request: Apache2Request::find_or_allocate_new(record).unwrap(),
        };
        let write_outcome = match handle_outcome {
            HandleOutcome::Processed(result) => match &result.result {
                Ok(response) => {
                    let outcome = write(&context, &response, writer);
                    let mut write_observers: [&mut dyn WriteResponseObserver; 3] = match &mut self.write_observers {
                        // TODO: find a nicer way to copy self.write_observers, clone method doesn't work with trait object elements
                        Some([observer_0, observer_1, observer_2]) => [*observer_0, *observer_1, *observer_2],
                        None => [&mut self.trans_trace, &mut self.response_analysis, &mut self.tile_handling_analysis],
                    };
                    for observer_iter in write_observers.iter_mut() {
                        debug!(
                            context.host.record,
                            "TileServer::write_response - calling observer {:p}", *observer_iter
                        );
                        (*observer_iter).on_write(&context, response, writer, &outcome, write, &read_outcome, &handle_outcome);
                    }
                    outcome
                }
                Err(_) => WriteOutcome::Processed(
                    Err(
                        WriteError::RequestNotHandled
                    ) // FIXME: propagate the HandleError properly
                )
            },
            HandleOutcome::Ignored => WriteOutcome::Ignored,
        };
        debug!(record.server, "TileServer::write_response - finish");
        return (write_outcome, self);
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
            _request: &Apache2Request,
            _outcome: &ReadOutcome,
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
                let (outcome, _) = proxy.read_request(request);
                outcome.expect_processed().unwrap();
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
            _request: &request::SlippyRequest,
            _handle_outcome: &HandleOutcome,
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
                let module_config = ModuleConfig::new();
                let proxy = TileProxy::new(server, module_config).unwrap();
                proxy.handle_observers = Some([&mut mock1]);
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let context = Apache2Request::create_with_tile_config(request)?;
                let read_outcome = ReadOutcome::Processed(
                    Ok(
                        request::SlippyRequest {
                            header: request::Header::new(
                                context.record,
                            ),
                            body: request::BodyVariant::ReportStatistics,
                        }
                    )
                );
                let (handle_outcome, _) = proxy.call_handlers(request, &read_outcome);
                handle_outcome.expect_processed().result?;
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
            _context: &WriteContext,
            _response: &response::SlippyResponse,
            _writer: &dyn Writer,
            _write_result: &WriteOutcome,
            _func: WriteResponseFunc,
            _read_outcome: &ReadOutcome,
            _handle_outcome: &HandleOutcome,
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
                let mut mock3 = MockWriteObserver {
                    count: 0,
                };
                let module_config = ModuleConfig::new();
                let proxy = TileProxy::new(server, module_config).unwrap();
                proxy.write_observers = Some([&mut mock1, &mut mock2, &mut mock3]);
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let context = Apache2Request::create_with_tile_config(request)?;
                let read_outcome = ReadOutcome::Processed(
                    Ok(
                        request::SlippyRequest {
                            header: request::Header::new(
                                context.record,
                            ),
                            body: request::BodyVariant::ReportStatistics,
                        }
                    )
                );
                let handle_result = HandleOutcome::Processed(
                    HandleRequestResult {
                        before_timestamp: Utc::now(),
                        after_timestamp: Utc::now(),
                        result: Ok(
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
                        ),
                    }
                );
                let (result, _) = proxy.write_response(request, &read_outcome, &handle_result);
                result.expect_processed().unwrap();
                assert_eq!(1, mock1.count, "Write observer not called");
                Ok(())
            })
        })
    }
}
