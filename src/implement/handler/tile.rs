use crate::schema::handler::result::{ HandleOutcome, HandleRequestResult };
use crate::schema::slippy::request::SlippyRequest;
use crate::schema::slippy::response;
use crate::schema::tile::age::TileAge;
use crate::schema::tile::identity::TileIdentity;
use crate::schema::tile::source::TileSource;
use crate::interface::handler::{ HandleContext, RequestHandler, };
use crate::interface::storage::TileStorage;

use chrono::Utc;

use std::collections::HashMap;


pub struct TileHandlingState {
    render_requests_by_tile_id: HashMap<TileIdentity, i32>,
}

pub struct TileHandler { }

impl RequestHandler for TileHandler {
    fn handle(
        &mut self,
        context: &HandleContext,
        request: &SlippyRequest,
    ) -> HandleOutcome {
        let before_timestamp = Utc::now();
        let response = response::SlippyResponse {
            header: response::Header::new(
                context.request.record,
                &mime::TEXT_PLAIN,
            ),
            body: response::BodyVariant::Tile(
                response::TileResponse {
                    source: TileSource::Cache,
                    age: TileAge::Fresh,
                }
            ),
        };
        let after_timestamp = Utc::now();
        return HandleOutcome::Processed(
            HandleRequestResult {
                before_timestamp,
                after_timestamp,
                result: Ok(response),
            }
        );
    }

}
