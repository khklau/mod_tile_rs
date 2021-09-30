use crate::handler::error::HandleError;

use crate::apache2::request::RequestContext;
use crate::slippy::interface::ParseRequestResult;
use crate::slippy::response::Response;


pub type HandleResult = Result<Option<Response>, HandleError>;
pub trait RequestHandler {
    fn handle(
        &mut self,
        context: &RequestContext,
        request: &ParseRequestResult,
    ) -> HandleResult;
}
