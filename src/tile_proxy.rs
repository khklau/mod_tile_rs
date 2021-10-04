#![allow(unused_unsafe)]

use crate::analytics::interface::ParseRequestObserver;
use crate::analytics::statistics::ModuleStatistics;
use crate::apache2::bindings::{
    APR_BADARG, APR_SUCCESS, DECLINED, HTTP_INTERNAL_SERVER_ERROR, OK,
    apr_status_t, process_rec, request_rec, server_rec,
};
use crate::apache2::error::InvalidRecordError;
use crate::apache2::memory::{ alloc, retrieve };
use crate::apache2::request::RequestContext;
use crate::apache2::virtual_host::VirtualHostContext;
use crate::slippy::error::ParseError;
use crate::slippy::interface::{ ParseOutcome, ParseRequestFunc };
use crate::slippy::parser::SlippyRequestParser;
use crate::storage::file_system;
use crate::tile::config::{ TileConfig, load };

use std::any::type_name;
use std::boxed::Box;
use std::error::Error;
use std::ffi::{ CStr, CString };
use std::option::Option;
use std::os::raw::{ c_int, c_void, };
use std::path::PathBuf;
use std::ptr;
use std::result::Result;
use std::time::Duration;


pub struct TileProxy<'p> {
    record: &'p mut server_rec,
    config: TileConfig,
    config_file_path: Option<PathBuf>,
    parse_request: ParseRequestFunc,
    statistics: ModuleStatistics,
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
        let proc_record = Self::access_proc_record(record.process)?;
        let context = match retrieve(
            unsafe { &mut *(proc_record.pool) },
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

    fn access_proc_record(process: *mut process_rec) -> Result<&'p mut process_rec, Box<dyn Error>> {
        if process == ptr::null_mut() {
            return Err(Box::new(InvalidRecordError::new(
                process,
                "null pointer",
            )));
        }
        let proc_record = unsafe { &mut *process };
        if proc_record.pool == ptr::null_mut() {
            return Err(Box::new(InvalidRecordError::new(
                proc_record as *const process_rec,
                "pool field is null pointer",
            )));
        }
        Ok(proc_record)
    }

    pub fn create(
        record: &'p mut server_rec,
        tile_config: TileConfig,
    ) -> Result<&'p mut Self, Box<dyn Error>> {
        info!(record, "TileServer::create - start");
        let proc_record = Self::access_proc_record(record.process)?;
        let new_server = alloc::<TileProxy<'p>>(
            unsafe { &mut *(proc_record.pool) },
            &(Self::get_id(record)),
            Some(drop_tile_server),
        )?.0;
        new_server.record = record;
        new_server.config = tile_config;
        new_server.config_file_path = None;
        new_server.parse_request = SlippyRequestParser::parse;
        new_server.statistics = ModuleStatistics { };
        info!(new_server.record, "TileServer::create - finish");
        return Ok(new_server);
    }

    pub fn load_tile_config(
        &mut self,
        file_path: PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let original_request_timeout = self.config.renderd.render_timeout.clone();
        let server_name = if self.record.server_hostname == ptr::null_mut() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.record.server_hostname).to_str()? })
        };
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
    ) -> Result<c_int, ParseError> {
        debug!(record.server, "TileServer::handle_request - start");
        let context = RequestContext::find_or_create(record, &self.config).unwrap();
        let request_url = context.uri;
        // TODO: use the macros & traits in the moveit crate to create a move constructor for TileProxy
        //       then move the following self referential values below into TileProxy
        let parse = self.parse_request;
        let mut parse_observers: [&mut dyn ParseRequestObserver; 1] = [&mut self.statistics];

        let parse_result = parse(context, request_url);
        for observer_iter in parse_observers.iter_mut() {
            (*observer_iter).on_parse(parse, context, request_url, &parse_result);
        }

        match parse_result {
            Ok(result) => {
                match result {
                    ParseOutcome::Matched(request) => request,
                    ParseOutcome::NotMatched => {
                        return Ok(DECLINED as c_int);
                    },
                }
            },
            Err(err) => match err {
                ParseError::Param(err) => {
                    error!(record.server, "Parameter {} error: {}", err.param, err.reason);
                    return Ok(DECLINED as c_int);
                },
                ParseError::Io(why) => {
                    error!(record.server, "IO error: {}", why);
                    return Ok(HTTP_INTERNAL_SERVER_ERROR as c_int);
                },
                ParseError::Utf8(why) => {
                    error!(record.server, "UTF8 error: {}", why);
                    return Ok(HTTP_INTERNAL_SERVER_ERROR as c_int);
                },
            },
        };

        debug!(record.server, "TileServer::handle_request - finish");
        return Ok(OK as c_int);
    }
}

#[no_mangle]
extern "C" fn drop_tile_server(server_void: *mut c_void) -> apr_status_t {
    if server_void == ptr::null_mut() {
        return APR_BADARG as apr_status_t;
    }
    let server_ptr = server_void as *mut TileProxy;
    info!((&mut *server_ptr).record, "drop_tile_server - start");
    let server_ref = unsafe { &mut *server_ptr };
    drop(server_ref);
    info!((&mut *server_ptr).record, "drop_tile_server - finish");
    return APR_SUCCESS as apr_status_t;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apache2::virtual_host::test_utils::with_server_rec;

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
}
