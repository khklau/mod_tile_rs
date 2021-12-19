use crate::apache2::request::RequestContext;
use crate::apache2::response::ResponseContext;
use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::response::Response;
use crate::schema::slippy::result::{ ReadRequestResult, WriteResponseResult };


pub type ReadRequestFunc = fn(&RequestContext) -> ReadRequestResult;

pub type WriteResponseFunc = fn(&mut ResponseContext, &Response) -> WriteResponseResult;

pub trait ReadRequestObserver {
    fn on_read(
        &mut self,
        func: ReadRequestFunc,
        context: &RequestContext,
        result: &ReadRequestResult,
    ) -> ();
}

pub trait WriteResponseObserver {
    fn on_write(
        &mut self,
        func: WriteResponseFunc,
        context: &ResponseContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
        write_result: &WriteResponseResult,
    ) -> ();
}
