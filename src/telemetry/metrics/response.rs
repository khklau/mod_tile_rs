use crate::apache2::response::ResponseContext;
use crate::interface::slippy::{ WriteResponseFunc, WriteResponseObserver, };
use crate::schema::slippy::response::Response;
use crate::schema::slippy::result::WriteResponseResult;


pub struct ResponseAnalysis {}

impl WriteResponseObserver for ResponseAnalysis {
    fn on_write(
        &mut self,
        _func: WriteResponseFunc,
        _context: &ResponseContext,
        _response: &Response,
        _result: &WriteResponseResult,
    ) -> () {
    }
}
