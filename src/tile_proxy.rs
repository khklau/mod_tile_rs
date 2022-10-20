use crate::binding::apache2::{
    APR_BADARG, APR_SUCCESS, DECLINED,
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
use crate::interface::apache2::{ PoolStored, HttpResponseWriter, };
use crate::interface::handler::{
    HandleContext, HandleRequestObserver, RequestHandler,
};
use crate::interface::slippy::{
    ReadContext, ReadRequestFunc, ReadRequestObserver,
    WriteContext, WriteResponseFunc, WriteResponseObserver,
};
use crate::interface::telemetry::{
    MetricsInventory, ResponseMetrics, TileHandlingMetrics,
};
use crate::framework::apache2::config::Loadable;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve };
use crate::framework::apache2::record::ServerRecord;
use crate::implement::handler::description::{ DescriptionHandler, DescriptionHandlerState, };
use crate::implement::handler::statistics::{ StatisticsHandler, StatisticsHandlerState, };
use crate::implement::handler::tile::{ TileHandler, TileHandlerState, };
use crate::implement::slippy::reader::SlippyRequestReader;
use crate::implement::slippy::writer::SlippyResponseWriter;
use crate::implement::telemetry::metrics::inventory::{ MetricsFactory, MetricsState, };
use crate::implement::telemetry::metrics::response::ResponseAnalysis;
use crate::implement::telemetry::metrics::tile_handling::TileHandlingAnalysis;
use crate::implement::telemetry::tracing::inventory::TracingState;
use crate::implement::telemetry::tracing::transaction::TransactionTrace;
use crate::utility::debugging::function_name;

use chrono::Utc;

use std::any::type_name;
use std::boxed::Box;
use std::convert::From;
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

impl From<ReadError> for HandleRequestError {
    fn from(error: ReadError) -> Self {
        HandleRequestError::Read(error)
    }
}

impl From<HandleError> for HandleRequestError {
    fn from(error: HandleError) -> Self {
        HandleRequestError::Handle(error)
    }
}

impl From<WriteError> for HandleRequestError {
    fn from(error: WriteError) -> Self {
        HandleRequestError::Write(error)
    }
}

impl std::fmt::Display for HandleRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleRequestError::Read(err) => return write!(f, "{}", err),
            HandleRequestError::Handle(err) => return write!(f, "{}", err),
            HandleRequestError::Write(err) => return write!(f, "{}", err),
        }
    }
}

pub struct TileProxy<'p> {
    config: ModuleConfig,
    config_file_path: Option<PathBuf>,
    metrics_factory: MetricsFactory<'p>,
    handler_factory: HandlerFactory<'p>,
    metrics_state: MetricsState,
    tracing_state: TracingState,
    description_handler_state: DescriptionHandlerState,
    statistics_handler_state: StatisticsHandlerState,
    tile_handler_state: TileHandlerState,
    read_request: ReadRequestFunc,
    read_func_name: &'static str,
    write_response: WriteResponseFunc,
    write_func_name: &'static str,
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
        new_server.config = module_config;
        new_server.config_file_path = None;
        new_server.metrics_factory = MetricsFactory::new();
        new_server.handler_factory = HandlerFactory::new();
        new_server.metrics_state = MetricsState::new();
        new_server.tracing_state = TracingState::new();
        new_server.description_handler_state = DescriptionHandlerState::new(&new_server.config)?;
        new_server.statistics_handler_state = StatisticsHandlerState::new(&new_server.config)?;
        new_server.tile_handler_state = TileHandlerState::new(&new_server.config)?;
        new_server.read_request = SlippyRequestReader::read;
        new_server.read_func_name = function_name(SlippyRequestReader::read);
        new_server.write_response = SlippyResponseWriter::write;
        new_server.write_func_name = function_name(SlippyResponseWriter::write);
        new_server.trans_trace = TransactionTrace { };
        new_server.read_observers = None;
        new_server.handle_observers = None;
        new_server.write_observers = None;
        info!(record, "TileServer::create - finish");
        return Ok(new_server);
    }

    pub fn load_config(
        &mut self,
        file_path: PathBuf,
        server_name: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let original_request_timeout = self.config.renderd.render_timeout.clone();
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
        record: &mut server_rec,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(original_path) = &self.config_file_path {
            let copied_path = original_path.clone();
            self.load_config(copied_path, record.get_host_name())?;
        }
        return Ok(());
    }

    pub fn handle_request(
        &mut self,
        record: &mut request_rec,
    ) -> Result<c_int, HandleRequestError> {
        debug!(record.server, "TileServer::handle_request - start");
        let (read_outcome, self2) = self.read_request(record);
        let (handle_outcome, self3) = self2.call_handlers(record, &read_outcome);
        let (write_outcome, _) = self3.write_response(record, &read_outcome, &handle_outcome);
        let result: Result<c_int, HandleRequestError> = match write_outcome {
            WriteOutcome::Processed(write_result) => {
                Ok(write_result?.status_code.as_u16() as c_int)
            },
            WriteOutcome::Ignored => Ok(DECLINED as c_int)
        };
        debug!(record.server, "TileServer::handle_request - finish");
        return result;
    }

    fn read_request(
        &mut self,
        record: &mut request_rec,
    ) -> (ReadOutcome, &mut Self) {
        debug!(record.server, "TileServer::read_request - start");
        let (read, read_func_name) = (self.read_request, self.read_func_name);
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
            (*observer_iter).on_read(&context, request, &read_outcome, read_func_name);
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
        let outcome = match read_outcome {
            ReadOutcome::Ignored => HandleOutcome::Ignored,
            ReadOutcome::Processed(result) => match result {
                Ok(request) => {
                    let (
                        module_config,
                        metrics_state,
                        metrics_factory,
                        tracing_state,
                        description_handler_state,
                        statistics_handler_state,
                        tile_handler_state,
                        handler_factory
                    ) = (
                        &self.config,
                        &self.metrics_state,
                        &self.metrics_factory,
                        &mut self.tracing_state,
                        &mut self.description_handler_state,
                        &mut self.statistics_handler_state,
                        &mut self.tile_handler_state,
                        &mut self.handler_factory,
                    );
                    let context = HandleContext::new(record, module_config) ;
                    metrics_factory.with_metrics_inventory(metrics_state, |metrics_inventory| {
                        handler_factory.with_handler_inventory(
                            module_config,
                            tracing_state,
                            description_handler_state,
                            statistics_handler_state,
                            tile_handler_state,
                            metrics_inventory,
                            |handler_inventory| {
                            let outcome_option = handler_inventory.handlers.iter_mut().find_map(|handler| {
                                (*handler).handle(&context, request).as_some_when_processed(handler.type_name())
                            });
                            if let Some((handle_outcome, handler_name)) = outcome_option {
                                for observer_iter in handler_inventory.handle_observers.iter_mut() {
                                    debug!(context.host.record, "TileServer::call_handlers - calling observer {:p}", *observer_iter);
                                    (*observer_iter).on_handle(&context, request, &handle_outcome, handler_name, read_outcome);
                                }
                                handle_outcome
                            } else {
                                HandleOutcome::Ignored
                            }
                        })
                    })
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
        let (write, write_func_name) = (self.write_response, self.write_func_name);
        // Work around the borrow checker below, but its necessary since request_rec from a foreign C framework
        let write_record = record as *mut request_rec;
        let writer: &mut dyn HttpResponseWriter = unsafe { write_record.as_mut().unwrap() };
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
                        None => [
                            &mut self.trans_trace,
                            &mut self.metrics_state.response_analysis,
                            &mut self.metrics_state.tile_handling_analysis
                        ],
                    };
                    for observer_iter in write_observers.iter_mut() {
                        debug!(
                            context.host.record,
                            "TileServer::write_response - calling observer {:p}", *observer_iter
                        );
                        (*observer_iter).on_write(&context, response, writer, &outcome, write_func_name, &read_outcome, &handle_outcome);
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
    drop(server_ref);
    return APR_SUCCESS as apr_status_t;
}

struct HandlerInventory<'i> {
    handlers: [&'i mut dyn RequestHandler; 3],
    handle_observers: [&'i mut dyn HandleRequestObserver; 1],
}

struct HandlerFactory<'f> {
    handlers: Option<[&'f mut dyn RequestHandler; 3]>,
    handle_observers: Option<[&'f mut dyn HandleRequestObserver; 1]>,
}

impl<'f> HandlerFactory<'f> {
    fn new() -> HandlerFactory<'f> {
        HandlerFactory {
            handlers: None,
            handle_observers: None,
        }
    }

    fn with_handler_inventory<F, R>(
        &mut self,
        _module_config: &ModuleConfig,
        tracing_state: &mut TracingState,
        description_handler_state: &mut DescriptionHandlerState,
        statistics_handler_state: &mut StatisticsHandlerState,
        tile_handler_state: &mut TileHandlerState,
        metrics_inventory: &MetricsInventory,
        func: F,
    ) -> R
    where
        F: FnOnce(&mut HandlerInventory) -> R {
        let mut description_handler = DescriptionHandler::new(description_handler_state);
        let mut statistics_handler = StatisticsHandler::new(statistics_handler_state, &metrics_inventory);
        let mut tile_handler = TileHandler::new(tile_handler_state, None);
        let mut handler_inventory = HandlerInventory {
            handlers: match &mut self.handlers {
                // TODO: find a nicer way to copy, clone method doesn't work with trait object elements
                Some([handler_0, handler_1, handler_2]) => [*handler_0, *handler_1, *handler_2],
                None => [&mut description_handler, &mut statistics_handler, &mut tile_handler],
            },
            handle_observers: match &mut self.handle_observers {
                // TODO: find a nicer way to copy, clone method doesn't work with trait object elements
                Some([observer_0]) => [*observer_0],
                None => [&mut tracing_state.trans_trace],
            }
        };
        func(&mut handler_inventory)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::slippy::result::ReadOutcome;
    use crate::schema::slippy::request;
    use crate::schema::slippy::response;
    use crate::framework::apache2::record::test_utils::{ with_request_rec, with_server_rec };
    use chrono::Utc;
    use mktemp::Temp;
    use thread_id;
    use std::boxed::Box;
    use std::error::Error;
    use std::marker::Send;
    use std::ops::FnOnce;
    use std::os::unix::net::UnixListener;
    use std::string::String;
    use std::sync::{ Arc, Mutex, };
    use std::thread::{ ScopedJoinHandle, scope, } ;

    struct MockEnv<'e> {
        mock_config: ModuleConfig,
        main_condition: Arc<Mutex<bool>>,
        renderd_thread: ScopedJoinHandle<'e, ()>,
    }

    fn with_mock_env<R, F>(func: F, render: R) -> Result<(), Box<dyn Error>>
        where F: FnOnce(&MockEnv) -> Result<(), Box<dyn Error>>,
              R: FnOnce(Arc<Mutex<bool>>, ModuleConfig, Temp, UnixListener) -> () + Send {
        let main_condition = Arc::new(Mutex::new(true));
        let temp_ipc_dir = Temp::new_dir()?;
        let mut ipc_path = temp_ipc_dir.to_path_buf();
        ipc_path.push("renderd.sock");
        let mut config = ModuleConfig::new();
        config.renderd.ipc_uri = ipc_path.to_str().unwrap().to_string();
        scope(|thread_scope| {
            let thread_condition = Arc::clone(&main_condition);
            let render_config_copy = config.clone();
            let listener = UnixListener::bind(&ipc_path).expect("Failed to bind");
            let env = MockEnv {
                mock_config: config,
                main_condition,
                renderd_thread: thread_scope.spawn(move|| {
                    render(thread_condition, render_config_copy, temp_ipc_dir, listener)
                }),
            };
            let result = func(&env);
            let mut renderd_persisting = true;
            while renderd_persisting {
                let lock = &(*env.main_condition);
                if let Ok(mut guard) = lock.try_lock() {
                    *guard = false;
                    renderd_persisting = *guard;
                }
            }
            return result;
        })
    }

    fn render_no_op(
        thread_condition: Arc<Mutex<bool>>,
        _config: ModuleConfig,
        _listener_dir: Temp,
        listener: UnixListener
    ) -> () {
        let mut persist = true;
        while persist {
            let lock = &(*thread_condition);
            if let Ok(guard) = lock.try_lock() {
                persist = *guard;
            }
        }
        drop(listener);
        return ();
    }

    #[test]
    fn test_proxy_reload() -> Result<(), Box<dyn Error>> {
        with_mock_env(|env| {
            with_server_rec(|record| {
                let proxy = TileProxy::new(record, env.mock_config.clone())?;

                let expected_timeout = Duration::new(30, 50);
                proxy.set_render_timeout(&expected_timeout);
                let mut expected_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                expected_path.push("resources/test/tile/basic_valid.conf");
                proxy.load_config(expected_path.clone(), record.get_host_name())?;

                let actual_timeout = proxy.config.renderd.render_timeout.clone();
                assert_eq!(expected_timeout, actual_timeout, "Failed to preserve request timeout during reload");
                assert!(proxy.config_file_path.is_some(), "Config file path is None");
                if let Some(actual_path) = &proxy.config_file_path {
                    assert_eq!(&expected_path, actual_path, "Failed to preserve config file path during reload");
                }
                Ok(())
            })
        }, render_no_op)
    }

    struct MockReadObserver {
        count: u32,
    }

    impl ReadRequestObserver for MockReadObserver {
        fn on_read(
            &mut self,
            _context: &ReadContext,
            _request: &Apache2Request,
            _outcome: &ReadOutcome,
            _read_func_name: &'static str,
        ) -> () {
            self.count += 1;
        }
    }

    #[test]
    fn test_read_request_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_mock_env(|env| {
            with_server_rec(|server| {
                with_request_rec(|request| {
                    let mut mock1 = MockReadObserver {
                        count: 0,
                    };
                    let proxy = TileProxy::new(server, env.mock_config.clone())?;
                    proxy.read_observers = Some([&mut mock1]);
                    let uri = CString::new("/mod_tile_rs")?;
                    request.uri = uri.into_raw();
                    let (outcome, _) = proxy.read_request(request);
                    outcome.expect_processed().unwrap();
                    assert_eq!(1, mock1.count, "Read observer not called");
                    Ok(())
                })
            })
        }, render_no_op)
    }

    struct MockHandleObserver {
        count: u32,
    }

    impl HandleRequestObserver for MockHandleObserver {
        fn on_handle(
            &mut self,
            _context: &HandleContext,
            _request: &request::SlippyRequest,
            _handle_outcome: &HandleOutcome,
            _handler_name: &'static str,
            _read_outcome: &ReadOutcome,
        ) -> () {
            self.count += 1;
        }
    }

    #[test]
    fn test_call_handlers_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_mock_env(|env| {
            with_server_rec(|server| {
                with_request_rec(|request| {
                    let mut mock1 = MockHandleObserver {
                        count: 0,
                    };
                    let proxy = TileProxy::new(server, env.mock_config.clone())?;
                    proxy.handler_factory.handle_observers = Some([&mut mock1]);
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
        }, render_no_op)
    }

    struct MockWriteObserver {
        count: u32,
    }

    impl WriteResponseObserver for MockWriteObserver {
        fn on_write(
            &mut self,
            _context: &WriteContext,
            _response: &response::SlippyResponse,
            _writer: &dyn HttpResponseWriter,
            _write_result: &WriteOutcome,
            _write_func_name: &'static str,
            _read_outcome: &ReadOutcome,
            _handle_outcome: &HandleOutcome,
        ) -> () {
            self.count += 1;
        }
    }

    #[test]
    fn test_write_response_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_mock_env(|env| {
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
                    let proxy = TileProxy::new(server, env.mock_config.clone())?;
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
        }, render_no_op)
    }
}
