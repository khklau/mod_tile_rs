use crate::adapter::slippy::interface::{
    ReadRequestFunc,
    ReadRequestObserver,
    WriteResponseFunc,
    WriteResponseObserver,
};
use crate::service::telemetry::interface::TelemetryInventory;
use crate::adapter::slippy::reader::SlippyRequestReader;
use crate::adapter::slippy::writer::SlippyResponseWriter;
use crate::core::debugging::function_name;


pub struct SlippyInventory { }

impl SlippyInventory {
    pub fn read_request_func() -> (ReadRequestFunc, &'static str) {
        let func = SlippyRequestReader::read;
        let name = function_name(func);
        (func, name)
    }

    pub fn write_response_func() -> (WriteResponseFunc, &'static str) {
        let func = SlippyResponseWriter::write;
        let name = function_name(func);
        (func, name)
    }
}

pub struct SlippyObserverInventory { }

impl SlippyObserverInventory {
    pub fn read_observers<'i>(
        telemetry: &'i mut dyn TelemetryInventory
    ) -> [&'i mut dyn ReadRequestObserver; 2] {
        let [read_observer_0, read_observer_1] = telemetry.read_request_observers();
        return [read_observer_0, read_observer_1];
    }

    pub fn write_observers<'i>(
        telemetry: &'i mut dyn TelemetryInventory,
    ) -> [&'i mut dyn WriteResponseObserver; 4] {
        let [
            write_observer_0,
            write_observer_1,
            write_observer_2,
            write_observer_3,
        ] = telemetry.write_response_observers();
        return [
            write_observer_0,
            write_observer_1,
            write_observer_2,
            write_observer_3,
        ];
    }
}
