use crate::schema::handler::result::HandleRequestResult;
use crate::schema::slippy::context::{ ReadContext, WriteContext };
use crate::schema::slippy::response::Response;
use crate::schema::slippy::result::{ ReadRequestResult, WriteResponseResult };


pub type ReadRequestFunc = fn(&ReadContext) -> ReadRequestResult;

pub type WriteResponseFunc = fn(&mut WriteContext, &Response) -> WriteResponseResult;

pub trait ReadRequestObserver {
    fn on_read(
        &mut self,
        func: ReadRequestFunc,
        context: &ReadContext,
        result: &ReadRequestResult,
    ) -> ();
}

pub trait WriteResponseObserver {
    fn on_write(
        &mut self,
        func: WriteResponseFunc,
        context: &WriteContext,
        read_result: &ReadRequestResult,
        handle_result: &HandleRequestResult,
        write_result: &WriteResponseResult,
    ) -> ();
}
