use crate::analytics::statistics::ModuleStatistics;
use crate::apache2::bindings::{
    APR_BADARG, APR_SUCCESS, OK,
    apr_status_t, request_rec, server_rec,
};
use crate::apache2::memory::{ access_pool_object, alloc, retrieve };
use crate::apache2::request::RequestContext;
use crate::apache2::response::ResponseContext;
use crate::apache2::virtual_host::{ ServerRecord, ProcessRecord, VirtualHostContext, };
use crate::handler::description::DescriptionHandler;
use crate::interface::handler::{ HandleRequestObserver, RequestHandler, };
use crate::interface::slippy::{
    ReadRequestFunc, ReadRequestObserver, WriteResponseFunc, WriteResponseObserver,
};
use crate::schema::handler::error::HandleError;
use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult, };
use crate::schema::slippy::error::{ ReadError, WriteError };
use crate::schema::slippy::result::{
    ReadOutcome, ReadRequestResult,
    WriteOutcome, WriteResponseResult,
};
use crate::schema::tile::config::{ TileConfig, load };
use crate::slippy::reader::SlippyRequestReader;
use crate::slippy::writer::SlippyResponseWriter;
use crate::storage::file_system;

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
    record: &'p mut server_rec,
    config: TileConfig,
    config_file_path: Option<PathBuf>,
    read_request: ReadRequestFunc,
    layer_handler: DescriptionHandler,
    write_response: WriteResponseFunc,
    statistics: ModuleStatistics,
    read_observers: Option<[&'p mut dyn ReadRequestObserver; 1]>,
    handle_observers: Option<[&'p mut dyn HandleRequestObserver; 1]>,
    write_observers: Option<[&'p mut dyn WriteResponseObserver; 1]>,
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

    pub fn find_or_create(record: &'p mut server_rec) -> Result<&'p mut Self, Box<dyn Error>> {
        info!(record, "TileServer::find_or_create - start");
        let proc_record = server_rec::get_process_record(record.process)?;
        let context = match retrieve(
            proc_record.get_pool(),
            &(Self::get_id(record))
        ) {
            Some(existing_context) => {
                info!(record, "TileServer::find_or_create - existing found");
                existing_context
            },
            None => {
                info!(record, "TileServer::find_or_create - not found");
                let tile_config = TileConfig::new();
                Self::create(record, tile_config)?
            },
        };
        info!(context.record, "TileServer::find_or_create - finish");
        return Ok(context);
    }

    pub fn create(
        record: &'p mut server_rec,
        tile_config: TileConfig,
    ) -> Result<&'p mut Self, Box<dyn Error>> {
        info!(record, "TileServer::create - start");
        let proc_record = server_rec::get_process_record(record.process)?;
        let new_server = alloc::<TileProxy<'p>>(
            proc_record.get_pool(),
            &(Self::get_id(record)),
            Some(drop_tile_server),
        )?.0;
        new_server.record = record;
        new_server.config = tile_config;
        new_server.config_file_path = None;
        new_server.read_request = SlippyRequestReader::read;
        new_server.layer_handler = DescriptionHandler { };
        new_server.write_response = SlippyResponseWriter::write;
        new_server.statistics = ModuleStatistics { };
        new_server.read_observers = None;
        new_server.handle_observers = None;
        new_server.write_observers = None;
        info!(new_server.record, "TileServer::create - finish");
        return Ok(new_server);
    }

    pub fn load_tile_config(
        &mut self,
        file_path: PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let original_request_timeout = self.config.renderd.render_timeout.clone();
        let server_name = self.record.get_host_name();
        let tile_config = load(file_path.as_path(), server_name)?;
        self.config = tile_config;
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
        record: &mut server_rec,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(original_path) = &self.config_file_path {
            let copied_path = original_path.clone();
            self.load_tile_config(copied_path)?;
        }
        let context = VirtualHostContext::find_or_create(record, &self.config).unwrap();
        file_system::initialise(context)?;
        return Ok(());
    }

    pub fn handle_request(
        &mut self,
        record: &mut request_rec,
    ) -> Result<c_int, ReadError> {
        debug!(record.server, "TileServer::handle_request - start");
        let (read_result, self2) = self.read_request(record);
        let (handle_result, self3) = self2.call_handlers(record, read_result);
        let (write_result, _) = self3.write_response(record, handle_result);
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
            Some(observers) => [observers[0]],
            None => [&mut self.statistics],
        };
        let read = self.read_request;
        let context = RequestContext::find_or_create(record, &self.config).unwrap();
        let read_result = read(context);
        for observer_iter in read_observers.iter_mut() {
            debug!(context.get_host().record, "TileServer::read_request - calling observer {:p}", *observer_iter);
            (*observer_iter).on_read(read, context, &read_result);
        }
        debug!(record.server, "TileServer::read_request - finish");
        return (read_result, self);
    }

    fn call_handlers(
        &mut self,
        record: &mut request_rec,
        read_result: ReadRequestResult,
    ) -> (HandleRequestResult, &mut Self) {
        debug!(record.server, "TileServer::call_handlers - start");
        // TODO: combine the handlers using combinators
        let mut handle_observers: [&mut dyn HandleRequestObserver; 1] = match &mut self.handle_observers {
            // TODO: find a nicer way to copy self.handle_observers, clone method doesn't work with trait object elements
            Some(observers) => [observers[0]],
            None => [&mut self.statistics],
        };
        let handler: &mut dyn RequestHandler = &mut self.layer_handler;
        let context = RequestContext::find_or_create(record, &self.config).unwrap();
        let handle_result = match read_result {
            Ok(outcome) => match outcome {
                ReadOutcome::NotMatched => Ok(HandleOutcome::NotHandled),
                ReadOutcome::Matched(request) => {
                    let result = handler.handle(context, &request);
                    for observer_iter in handle_observers.iter_mut() {
                        debug!(context.get_host().record, "TileServer::call_handlers - calling observer {:p}", *observer_iter);
                        (*observer_iter).on_handle(handler, context, &request, &result);
                    }
                    result
                },
            },
            Err(err) => {
                Err(HandleError::RequestNotRead(err))
            },
        };
        debug!(record.server, "TileServer::call_handlers - finish");
        return (handle_result, self);
    }

    fn write_response(
        &mut self,
        record: &mut request_rec,
        handle_result: HandleRequestResult,
    ) -> (WriteResponseResult, &mut Self) {
        debug!(record.server, "TileServer::write_response - start");
        let mut write_observers: [&mut dyn WriteResponseObserver; 1] = match &mut self.write_observers {
            // TODO: find a nicer way to copy self.write_observers, clone method doesn't work with trait object elements
            Some(observers) => [observers[0]],
            None => [&mut self.statistics],
        };
        let write = self.write_response;
        let request_context = RequestContext::find_or_create(record, &self.config).unwrap();
        let mut response_context = ResponseContext::from(request_context);
        let write_result = match handle_result {
            Ok(outcome) => match outcome {
                HandleOutcome::NotHandled => Ok(WriteOutcome::NotWritten),
                HandleOutcome::Handled(response) => {
                    let result = write(&mut response_context, &response);
                    for observer_iter in write_observers.iter_mut() {
                        debug!(
                            response_context.get_host().record,
                            "TileServer::write_response - calling observer {:p}", *observer_iter
                        );
                        (*observer_iter).on_write(write, &response_context, &response, &result);
                    }
                    result
                },
            },
            Err(_) => {
                Err(WriteError::RequestNotHandled) // FIXME: propagate the HandleError properly
            },
        };
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
    use crate::apache2::request::test_utils::with_request_rec;
    use crate::apache2::virtual_host::test_utils::with_server_rec;
    use crate::schema::slippy::result::ReadOutcome;
    use crate::schema::slippy::request;
    use crate::schema::slippy::response;

    #[test]
    fn test_proxy_reload() -> Result<(), Box<dyn Error>> {
        with_server_rec(|record| {
            let tile_config = TileConfig::new();
            let proxy = TileProxy::create(record, tile_config).unwrap();

            let expected_timeout = Duration::new(30, 50);
            proxy.set_render_timeout(&expected_timeout);
            let mut expected_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            expected_path.push("resources/test/tile/basic_valid.conf");
            proxy.load_tile_config(expected_path.clone())?;

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
            _context: &RequestContext,
            _result: &ReadRequestResult
        ) -> () {
            self.count += 1;
        }
    }

    #[test]
    fn test_read_request_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let mut mock = MockReadObserver {
                    count: 0,
                };
                let tile_config = TileConfig::new();
                let proxy = TileProxy::create(server, tile_config).unwrap();
                proxy.read_observers = Some([&mut mock]);
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let (result, _) = proxy.read_request(request);
                result.unwrap();
                assert_eq!(1, mock.count, "Read observer not called");
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
            _context: &RequestContext,
            _request: &request::Request,
            _result: &HandleRequestResult
        ) -> () {
            self.count += 1;
        }
    }

    #[test]
    fn test_call_handlers_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let mut mock = MockHandleObserver {
                    count: 0,
                };
                let tile_config = TileConfig::new();
                let proxy = TileProxy::create(server, tile_config).unwrap();
                proxy.handle_observers = Some([&mut mock]);
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let context = RequestContext::create_with_tile_config(request, &proxy.config)?;
                let input: ReadRequestResult = Ok(
                    ReadOutcome::Matched(
                        request::Request {
                            header: request::Header::new(
                                context.record,
                                context.connection.record,
                                context.get_host().record,
                            ),
                            body: request::BodyVariant::ReportStatistics,
                        }
                    )
                );
                let (result, _) = proxy.call_handlers(request, input);
                result.unwrap();
                assert_eq!(1, mock.count, "Handle observer not called");
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
            _context: &ResponseContext,
            _response: &response::Response,
            _result: &WriteResponseResult,
        ) -> () {
            self.count += 1;
        }
    }

    #[test]
    fn test_write_response_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let mut mock = MockWriteObserver {
                    count: 0,
                };
                let tile_config = TileConfig::new();
                let proxy = TileProxy::create(server, tile_config).unwrap();
                proxy.write_observers = Some([&mut mock]);
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let context = RequestContext::create_with_tile_config(request, &proxy.config)?;
                let input: HandleRequestResult = Ok(
                    HandleOutcome::Handled(
                        response::Response {
                            header: response::Header::new(
                                context.record,
                                context.connection.record,
                                context.get_host().record,
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
                );
                let (result, _) = proxy.write_response(request, input);
                result.unwrap();
                assert_eq!(1, mock.count, "Write observer not called");
                Ok(())
            })
        })
    }
}
