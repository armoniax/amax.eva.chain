use crate::{listeners::raw::Listener, types::single::TransactionTrace};

pub struct Formatter;

impl super::ResponseFormatter for Formatter {
    type Listener = Listener;
    type Response = TransactionTrace;

    fn format(listener: Listener) -> Option<TransactionTrace> {
        Some(TransactionTrace::Raw {
            step_logs: listener.step_logs,
            gas: listener.final_gas.into(),
            return_value: listener.return_value,
        })
    }
}
