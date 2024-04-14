use crate::binding::apache2::{
    APR_BADARG, APR_SUCCESS, DECLINED,
    apr_status_t, request_rec, server_rec,
};
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::apache2::virtual_host::VirtualHost;
use crate::schema::handler::error::HandleError;
use crate::schema::handler::result::{HandleOutcome, HandleRequestResult,};
use crate::schema::slippy::request::{BodyVariant, SlippyRequest};
use crate::schema::slippy::error::{ReadError, WriteError,};
use crate::schema::slippy::result::{ReadOutcome, WriteOutcome,};
use crate::core::memory::PoolStored;
use crate::io::communication::interface::HttpResponseWriter;
use crate::io::interface::IOContext;
use crate::framework::apache2::context::HostContext;
use crate::framework::apache2::config::Loadable;
use crate::framework::apache2::memory::{ access_pool_object, alloc, retrieve };
use crate::framework::apache2::record::ServerRecord;
use crate::io::communication::state::CommunicationState;
use crate::use_case::inventory::{HandlerObserverInventory, HandlerState,};
use crate::adapter::http::reader::read_apache2_request;
use crate::adapter::slippy::interface::{ReadContext, WriteContext,};
use crate::adapter::slippy::inventory::{SlippyInventory, SlippyObserverInventory,};
use crate::io::storage::state::StorageState;
use crate::service::interface::ServicesContext;
use crate::service::rendering::inventory::RenderingState;
use crate::service::telemetry::inventory::TelemetryState;
use crate::use_case::description::DescriptionContext;
use crate::use_case::statistics::StatisticsContext;
use crate::use_case::tile::TileContext;

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

pub struct TileProxy {
    config: ModuleConfig,
    config_file_path: Option<PathBuf>,
    comms_state: CommunicationState,
    storage_state: StorageState,
    rendering_state: RenderingState,
    telemetry_state: TelemetryState,
    handler_state: HandlerState,
}

impl TileProxy {
    pub fn find_or_allocate_new(record: &mut server_rec) -> Result<&mut Self, Box<dyn Error>> {
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
        record: &server_rec,
        module_config: ModuleConfig,
    ) -> Result<&mut Self, Box<dyn Error>> {
        info!(record, "TileServer::create - start");
        let new_server = alloc::<TileProxy>(
            record.get_pool()?,
            &(Self::get_id(record)),
            Some(drop_tile_server),
        )?.0;
        new_server.config = module_config;
        new_server.config_file_path = None;
        new_server.comms_state = CommunicationState::new(&new_server.config)?;
        new_server.storage_state = StorageState::new(&new_server.config)?;
        new_server.rendering_state = RenderingState::new(&new_server.config)?;
        new_server.telemetry_state = TelemetryState::new(&new_server.config)?;
        new_server.handler_state = HandlerState::new(&new_server.config)?;
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
        let (read, read_func_name) = SlippyInventory::read_request_func();
        let context = ReadContext {
            host_context: HostContext {
                module_config: &self.config,
                host: VirtualHost::find_or_allocate_new(record).unwrap(),
            }
        };
        let request = match read_apache2_request(record) {
            Ok(request) => request,
            Err(err) => return (
                ReadOutcome::Processed(
                    Err(
                        ReadError::Utf8(err)
                    )
                ),
                self,
            )
        };
        let read_outcome = read(&context, &request);
        for observer_iter in SlippyObserverInventory::read_observers(&mut self.telemetry_state).iter_mut() {
            debug!(context.host().record, "TileServer::read_request - calling observer {:p}", *observer_iter);
            (*observer_iter).on_read(&context, &request, &read_outcome, read_func_name);
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
                    match &(request.body) {
                        BodyVariant::DescribeLayer => {
                            self.call_description_handler(record, request)
                        },
                        BodyVariant::ReportStatistics => {
                            self.call_statistics_handler(record, request)
                        },
                        BodyVariant::ServeTileV2(_) | BodyVariant::ServeTileV3(_) => {
                            self.call_tile_handler(record, request)
                        }
                    }
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

    fn call_description_handler(
        &mut self,
        record: &mut request_rec,
        request: &SlippyRequest,
    ) -> HandleOutcome {
        debug!(record.server, "TileServer::call_description_handler - start");
        let outcome_option = {
            let context = DescriptionContext {
                host: HostContext::new(&self.config, record),
            };
            self.handler_state.description.describe_layer(
                &context,
                request,
            ).as_some_when_processed(self.handler_state.description.type_name())
        };
        let outcome = match outcome_option {
            Some((handle_outcome, handler_name)) => {
                for observer_iter in HandlerObserverInventory::description_use_case_observers(&mut self.telemetry_state).iter_mut() {
                    (*observer_iter).on_describe_layer(request, &handle_outcome, handler_name);
                }
                handle_outcome
            },
            None => HandleOutcome::Ignored,
        };
        debug!(record.server, "TileServer::call_description_handler - finish");
        return outcome;
    }

    fn call_statistics_handler(
        &mut self,
        record: &mut request_rec,
        request: &SlippyRequest,
    ) -> HandleOutcome {
        debug!(record.server, "TileServer::call_statistics_handler - start");
        let outcome_option = {
            let context = StatisticsContext {
                host: HostContext::new(&self.config, record),
                services: ServicesContext {
                    telemetry: &self.telemetry_state,
                    rendering: &mut self.rendering_state,
                },
            };
            self.handler_state.statistics.report_statistics(
                &context,
                request
            ).as_some_when_processed(self.handler_state.statistics.type_name())
        };
        let outcome = match outcome_option {
            Some((handle_outcome, handler_name)) => {
                for observer_iter in HandlerObserverInventory::statistics_use_case_observers(&mut self.telemetry_state).iter_mut() {
                    (*observer_iter).on_report_statistics(request, &handle_outcome, handler_name);
                }
                handle_outcome
            },
            None => HandleOutcome::Ignored,
        };
        debug!(record.server, "TileServer::call_statistics_handler - finish");
        return outcome;
    }

    fn call_tile_handler(
        &mut self,
        record: &mut request_rec,
        request: &SlippyRequest,
    ) -> HandleOutcome {
        debug!(record.server, "TileServer::call_tile_handler - start");
        let outcome_option = {
            let mut context = TileContext {
                host: HostContext::new(&self.config, record),
                io: IOContext {
                    communication: &mut self.comms_state,
                    storage: &mut self.storage_state,
                },
                services: ServicesContext {
                    telemetry: &self.telemetry_state,
                    rendering: &mut self.rendering_state,
                },
            };
            self.handler_state.tile.fetch_tile(
                &mut context,
                request,
            ).as_some_when_processed(self.handler_state.tile.type_name())
        };
        let outcome = match outcome_option {
            Some((handle_outcome, handler_name)) => {
                for observer_iter in HandlerObserverInventory::tile_use_case_observers(&mut self.telemetry_state).iter_mut() {
                    (*observer_iter).on_fetch_tile(request, &handle_outcome, handler_name);
                }
                handle_outcome
            },
            None => HandleOutcome::Ignored,
        };
        debug!(record.server, "TileServer::call_tile_handler - finish");
        return outcome;
    }

    fn write_response(
        &mut self,
        record: &mut request_rec,
        read_outcome: &ReadOutcome,
        handle_outcome: &HandleOutcome,
    ) -> (WriteOutcome, &mut Self) {
        debug!(record.server, "TileServer::write_response - start");
        let (write, write_func_name) = SlippyInventory::write_response_func();
        // Work around the borrow checker below, but its necessary since request_rec from a foreign C framework
        let write_record = record as *mut request_rec;
        let writer: &mut dyn HttpResponseWriter = unsafe { write_record.as_mut().unwrap() };
        let context = WriteContext {
            host_context: HostContext::new(&self.config, record),
            read_outcome,
        };
        let write_outcome = match handle_outcome {
            HandleOutcome::Processed(result) => match &result.result {
                Ok(response) => {
                    let outcome = write(&context, &response, writer);
                    for observer_iter in SlippyObserverInventory::write_observers(&mut self.telemetry_state).iter_mut() {
                        debug!(
                            context.host().record,
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::identifier::generate_id;
    use crate::schema::slippy::result::ReadOutcome;
    use crate::schema::slippy::request;
    use crate::schema::slippy::response;
    use crate::schema::tile::identity::LayerName;
    use crate::framework::apache2::record::test_utils::{ with_request_rec, with_server_rec };
    use chrono::Utc;
    use std::boxed::Box;
    use std::error::Error;
    use std::string::String;

    #[test]
    fn test_proxy_reload() -> Result<(), Box<dyn Error>> {
        with_server_rec(|record| {
            let module_config = ModuleConfig::new();
            let proxy = TileProxy::new(record, module_config)?;

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
    }

    #[test]
    fn test_read_request_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let module_config = ModuleConfig::new();
                let proxy = TileProxy::new(server, module_config)?;
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.into_raw();
                let (outcome, _) = proxy.read_request(request);
                outcome.expect_processed().unwrap();
                let actual_count = proxy.telemetry_state.read_counter().count;
                assert_eq!(1, actual_count, "Read observer not called");
                Ok(())
            })
        })
    }

    #[test]
    fn test_call_handlers_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let module_config = ModuleConfig::new();
                let proxy = TileProxy::new(server, module_config)?;
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.clone().into_raw();
                let read_outcome = ReadOutcome::Processed(
                    Ok(
                        request::SlippyRequest {
                            header: request::Header {
                                layer: LayerName::new(),
                                request_id: generate_id(),
                                uri: uri.into_string()?,
                                received_timestamp: Utc::now(),
                            },
                            body: request::BodyVariant::ReportStatistics,
                        }
                    )
                );
                let (handle_outcome, _) = proxy.call_handlers(request, &read_outcome);
                handle_outcome.expect_processed().result?;
                let actual_count = proxy.telemetry_state.handle_counter().count;
                assert_eq!(1, actual_count, "Handle observer not called");
                Ok(())
            })
        })
    }

    #[test]
    fn test_call_description_handler_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let module_config = ModuleConfig::new();
                let proxy = TileProxy::new(server, module_config)?;
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.clone().into_raw();
                let slippy_request = request::SlippyRequest {
                    header: request::Header {
                        layer: LayerName::from("default"),
                        request_id: generate_id(),
                        uri: uri.into_string()?,
                        received_timestamp: Utc::now(),
                    },
                    body: request::BodyVariant::DescribeLayer,
                };
                let handle_outcome = proxy.call_description_handler(request, &slippy_request);
                handle_outcome.expect_processed().result?;
                let actual_count = proxy.telemetry_state.handle_counter().count;
                assert_eq!(1, actual_count, "Handle observer not called");
                Ok(())
            })
        })
    }

    #[test]
    fn test_call_statistics_handler_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let module_config = ModuleConfig::new();
                let proxy = TileProxy::new(server, module_config)?;
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.clone().into_raw();
                let slippy_request = request::SlippyRequest {
                    header: request::Header {
                        layer: LayerName::from("default"),
                        request_id: generate_id(),
                        uri: uri.into_string()?,
                        received_timestamp: Utc::now(),
                    },
                    body: request::BodyVariant::ReportStatistics,
                };
                let handle_outcome = proxy.call_statistics_handler(request, &slippy_request);
                handle_outcome.expect_processed().result?;
                let actual_count = proxy.telemetry_state.handle_counter().count;
                assert_eq!(1, actual_count, "Handle observer not called");
                Ok(())
            })
        })
    }

    #[test]
    fn test_write_response_calls_mock_observer() -> Result<(), Box<dyn Error>> {
        with_server_rec(|server| {
            with_request_rec(|request| {
                let module_config = ModuleConfig::new();
                let proxy = TileProxy::new(server, module_config)?;
                let uri = CString::new("/mod_tile_rs")?;
                request.uri = uri.clone().into_raw();
                let read_outcome = ReadOutcome::Processed(
                    Ok(
                        request::SlippyRequest {
                            header: request::Header {
                                layer: LayerName::new(),
                                request_id: generate_id(),
                                uri: uri.into_string()?,
                                received_timestamp: Utc::now(),
                            },
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
                                header: response::Header {
                                    mime_type: mime::APPLICATION_JSON.clone(),
                                },
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
                let actual_count = proxy.telemetry_state.write_counter().count;
                assert_eq!(1, actual_count, "Write observer not called");
                Ok(())
            })
        })
    }
}
