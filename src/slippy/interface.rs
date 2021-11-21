use crate::apache2::request::RequestContext;
use crate::apache2::response::ResponseContext;
use crate::schema::slippy::response::Response;
use crate::schema::slippy::result::{ ReadRequestResult, WriteResponseResult };


pub type ReadRequestFunc = fn(&RequestContext) -> ReadRequestResult;

pub type WriteResponseFunc = fn(&mut ResponseContext, &Response) -> WriteResponseResult;
