use crate::implement::telemetry::tracing::transaction::TransactionTrace;


pub struct TracingState {
    pub trans_trace: TransactionTrace,
}

impl TracingState {
    pub fn new() -> TracingState {
        TracingState {
            trans_trace: TransactionTrace { },
        }
    }
}
