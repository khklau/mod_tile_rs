use crate::schema::core::processed::ProcessOutcome;
use crate::schema::http::response::HttpResponse;
use crate::schema::slippy::error::{ ReadError, WriteError };
use crate::schema::slippy::request::SlippyRequest;

use std::result::Result;


pub type ReadRequestResult = Result<SlippyRequest, ReadError>;

pub type ReadOutcome = ProcessOutcome<ReadRequestResult>;


pub type WriteResponseResult = Result<HttpResponse, WriteError>;
