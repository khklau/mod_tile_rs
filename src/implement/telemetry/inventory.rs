use crate::interface::handler::HandleRequestObserver;
use crate::interface::slippy::{ ReadRequestObserver, WriteResponseObserver, };
use crate::implement::telemetry::transaction::TransactionTrace;

use std::option::Option;


#[cfg(not(test))]
pub struct TracingState {
    pub trans_trace: TransactionTrace,
}

#[cfg(not(test))]
impl TracingState {
    pub fn new() -> TracingState {
        TracingState {
            trans_trace: TransactionTrace { },
        }
    }

    pub fn read_request_observers(&mut self) -> [&mut dyn ReadRequestObserver; 1] {
        [&mut self.trans_trace]
    }

    pub fn handle_request_observers(&mut self) -> [&mut dyn HandleRequestObserver; 1] {
        [&mut self.trans_trace]
    }

    pub fn write_response_observers(&mut self) -> [&mut dyn WriteResponseObserver; 1] {
        [&mut self.trans_trace]
    }
}


#[cfg(test)]
pub struct TracingState {
    pub trans_trace: TransactionTraceVariant,
}

#[cfg(test)]
impl TracingState {
    pub fn new() -> TracingState {
        TracingState {
            trans_trace: TransactionTraceVariant::Real(
                TransactionTrace { }
            ),
        }
    }

    pub fn new_mock(mock: TransactionTraceVariant) -> TracingState {
        TracingState {
            trans_trace: mock,
        }
    }

    pub fn read_request_observers(&mut self) -> [&mut dyn ReadRequestObserver; 1] {
        [
            match &mut self.trans_trace {
                TransactionTraceVariant::Real(trace) => &mut *trace,
                TransactionTraceVariant::MockNoOp(trace) => &mut *trace,
            },
        ]
    }

    pub fn handle_request_observers(&mut self) -> [&mut dyn HandleRequestObserver; 1] {
        [
            match &mut self.trans_trace {
                TransactionTraceVariant::Real(trace) => &mut *trace,
                TransactionTraceVariant::MockNoOp(trace) => &mut *trace,
            },
        ]
    }

    pub fn write_response_observers(&mut self) -> [&mut dyn WriteResponseObserver; 1] {
        [
            match &mut self.trans_trace {
                TransactionTraceVariant::Real(trace) => &mut *trace,
                TransactionTraceVariant::MockNoOp(trace) => &mut *trace,
            },
        ]
    }
}

#[cfg(test)]
pub enum TransactionTraceVariant {
    Real(TransactionTrace),
    MockNoOp(crate::implement::telemetry::transaction::test_utils::MockNoOpTransactionTrace),
}


pub struct TracingInventory<'i> {
    pub read_observer: &'i mut dyn ReadRequestObserver,
    pub handle_observer: &'i mut dyn HandleRequestObserver,
    pub write_observer: &'i mut dyn WriteResponseObserver,
}

pub struct TracingFactory<'f> {
    pub read_observer: Option<&'f mut dyn ReadRequestObserver>,
    pub handle_observer: Option<&'f mut dyn HandleRequestObserver>,
    pub write_observer: Option<&'f mut dyn WriteResponseObserver>,
}

impl<'f> TracingFactory<'f> {
    pub fn new() -> TracingFactory<'f> {
        TracingFactory {
            read_observer: None,
            handle_observer: None,
            write_observer: None,
        }
    }
}
