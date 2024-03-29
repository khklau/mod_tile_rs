#[cfg(not(test))]
#[macro_export(local_inner_macros_)]
macro_rules! error {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(crate::binding::apache2::APLOG_ERR, $server_expr, $($arg)+)
    );
}

#[cfg(not(test))]
#[macro_export(local_inner_macros_)]
macro_rules! warn {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(crate::binding::apache2::APLOG_WARNING, $server_expr, $($arg)+)
    );
}

#[cfg(not(test))]
#[macro_export(local_inner_macros_)]
macro_rules! info {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(crate::binding::apache2::APLOG_INFO, $server_expr, $($arg)+)
    );
}

#[cfg(not(test))]
#[macro_export(local_inner_macros_)]
macro_rules! debug {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(crate::binding::apache2::APLOG_DEBUG, $server_expr, $($arg)+)
    );
}

#[cfg(not(test))]
#[macro_export(local_inner_macros_)]
macro_rules! trace {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(crate::binding::apache2::APLOG_TRACE1, $server_expr, $($arg)+)
    );
}

#[cfg(not(test))]
macro_rules! _log { (
    $level:expr,
    $server_expr:expr,
    $($arg:tt)+) => {
        let msg = std::ffi::CString::new(format!($($arg)+)).unwrap();
        unsafe {
            crate::binding::apache2::ap_log_error_(
                cstr!(file!()),
                line!() as std::os::raw::c_int,
                crate::binding::apache2::APLOG_NO_MODULE as std::os::raw::c_int,
                $level as std::os::raw::c_int,
                -1,
                $server_expr,
                msg.as_ptr(),
            );
        };
    }
}

#[cfg(test)]
use crate::binding::apache2::{ apr_initialize, apr_terminate, };

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
        _log!(log::Level::Error, $server_expr, $($arg)+)
    );
}

#[cfg(test)]
#[macro_export]
macro_rules! warn {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(log::Level::Warn, $server_expr, $($arg)+)
    );
}

#[cfg(test)]
#[macro_export]
macro_rules! info {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(log::Level::Info, $server_expr, $($arg)+)
    );
}

#[cfg(test)]
#[macro_export]
macro_rules! debug {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(log::Level::Debug, $server_expr, $($arg)+)
    );
}

#[cfg(test)]
#[macro_export]
macro_rules! trace {
    ($server_expr:expr, $($arg:tt)+) => (
        _log!(log::Level::Trace, $server_expr, $($arg)+)
    );
}

#[cfg(test)]
macro_rules! _log { (
    $level:expr,
    $server_expr:expr,
    $($arg:tt)+) => {
        let _ = &($server_expr);
        (log::log!($level, $($arg)+));
    }
}
