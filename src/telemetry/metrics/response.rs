use crate::apache2::response::ResponseContext;
use crate::interface::slippy::{ WriteResponseFunc, WriteResponseObserver, };
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::result::{ ReadRequestResult, WriteResponseResult, WriteOutcome };
use crate::schema::tile::config::MAX_ZOOM_SERVER;

use http::status::StatusCode;

use std::collections::hash_map::HashMap;
use std::vec::Vec;

pub struct ResponseAnalysis {
    response_count_by_status_and_zoom: HashMap<StatusCode, Vec<u32>>,
    tile_reponse_count_by_zoom: Vec<u32>,
}

impl ResponseAnalysis {
    pub fn new() -> ResponseAnalysis {
        ResponseAnalysis {
            response_count_by_status_and_zoom: HashMap::new(),
            tile_reponse_count_by_zoom: Vec::new(),
        }
    }
}

impl WriteResponseObserver for ResponseAnalysis {
    fn on_write(
        &mut self,
        _func: WriteResponseFunc,
        _context: &ResponseContext,
        _read_result: &ReadRequestResult,
        _handle_result: &HandleRequestResult,
        write_result: &WriteResponseResult,
    ) -> () {
        if let Ok(outcome) = write_result {
            if let WriteOutcome::Written(http_response) = outcome {
                self.response_count_by_status_and_zoom.insert(http_response.status_code, vec![0; MAX_ZOOM_SERVER]);
            }
        }
    }
}
