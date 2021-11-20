use crate::apache2::request::RequestContext;
use crate::schema::handler::outcome::HandleRequestResult;
use crate::schema::slippy::request::Request;


pub trait RequestHandler {
    fn handle(
        &mut self,
        context: &RequestContext,
        request: &Request,
    ) -> HandleRequestResult;
}
