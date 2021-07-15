#[cfg(not(test))]
#[macro_export(local_inner_macros_)]
macro_rules! error {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(crate::apache2::bindings::APLOG_ERR, $server_expr, $($arg)+)
    );
}

#[cfg(not(test))]
#[macro_export(local_inner_macros_)]
macro_rules! warn {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(crate::apache2::bindings::APLOG_WARNING, $server_expr, $($arg)+)
    );
}

#[cfg(not(test))]
#[macro_export(local_inner_macros_)]
macro_rules! info {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(crate::apache2::bindings::APLOG_INFO, $server_expr, $($arg)+)
    );
}

#[cfg(not(test))]
#[macro_export(local_inner_macros_)]
macro_rules! debug {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(crate::apache2::bindings::APLOG_DEBUG, $server_expr, $($arg)+)
    );
}

#[cfg(not(test))]
#[macro_export(local_inner_macros_)]
macro_rules! trace {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(crate::apache2::bindings::APLOG_TRACE1, $server_expr, $($arg)+)
    );
}

#[cfg(not(test))]
macro_rules! _log { (
    $level:expr,
    $server_expr:expr,
    $($arg:tt)+) => {
        let msg = std::ffi::CString::new(format!($($arg)+)).unwrap();
        unsafe {
            crate::apache2::bindings::ap_log_error_(
                cstr!(file!()),
                line!() as std::os::raw::c_int,
                crate::apache2::bindings::APLOG_NO_MODULE as std::os::raw::c_int,
                $level as std::os::raw::c_int,
                -1,
                $server_expr,
                msg.as_ptr(),
            );
        };
    }
}

#[cfg(test)]
use crate::apache2::bindings::{ apr_initialize, apr_terminate, };

#[cfg(test)]
#[ctor::ctor]
unsafe fn mod_test_setup() {
    apr_initialize();
    env_logger::init();
}

#[cfg(test)]
#[ctor::dtor]
unsafe fn mod_test_teardown() {
    apr_terminate();
}

#[cfg(test)]
#[macro_export]
macro_rules! error {
    ($server_expr:expr, $($arg:tt)+) => (
        (log::log!(log::Level::Error, $($arg)+))
    );
}

#[cfg(test)]
#[macro_export]
macro_rules! warn {
    ($server_expr:expr, $($arg:tt)+) => (
        (log::log!(log::Level::Warn, $($arg)+))
    );
}

#[cfg(test)]
#[macro_export]
macro_rules! info {
    ($server_expr:expr, $($arg:tt)+) => (
        (log::log!(log::Level::Info, $($arg)+))
    );
}

#[cfg(test)]
#[macro_export]
macro_rules! debug {
    ($server_expr:expr, $($arg:tt)+) => (
        (log::log!(log::Level::Debug, $($arg)+))
    );
}

#[cfg(test)]
#[macro_export]
macro_rules! trace {
    ($server_expr:expr, $($arg:tt)+) => (
        (log::log!(log::Level::Trace, $($arg)+))
    );
}
