use crate::apache2::bindings::{
    APR_SUCCESS,
    apr_palloc, apr_pool_userdata_get, apr_pool_userdata_set,
    apr_pool_t, apr_size_t, apr_status_t,
    conn_rec, memset, process_rec, request_rec,
};

use std::alloc::Layout;
use std::error::Error;
use std::ffi::{CString, c_void,};
use std::fmt;
use std::option::Option;
use std::os::raw::c_ulong;
use std::ptr;
use std::result::Result;

#[derive(Debug)]
pub struct AllocError {
    layout: Layout,
}

impl Error for AllocError {}

impl fmt::Display for AllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot allocate with size {} and alignment {}", self.layout.size(), self.layout.align())
    }
}

pub fn alloc<'p, T>(pool: &'p mut apr_pool_t) -> Result<&'p mut T, AllocError> {
    let layout = Layout::new::<T>();
    unsafe {
        let ptr_raw = apr_palloc(pool, layout.size() as apr_size_t);
        if ptr_raw == ptr::null_mut() {
            return Err(AllocError{ layout: layout });
        }
        else {
            let ptr_zeroed = memset(ptr_raw, 0, layout.size() as c_ulong) as *mut T;
            return Ok(&mut *ptr_zeroed)
        }
    }
}

pub type CleanUpFn = unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void) -> apr_status_t;

pub fn _alloc<'p, T>(
    pool: &'p mut apr_pool_t,
    id: &CString,
    cleanup: Option<CleanUpFn>,
) -> Result<&'p mut T, AllocError> {
    let layout = Layout::new::<T>();
    unsafe {
        let ptr_raw = apr_palloc(pool, layout.size() as apr_size_t);
        if ptr_raw == ptr::null_mut() {
            return Err(AllocError{ layout });
        }
        let ptr_zeroed = memset(ptr_raw, 0, layout.size() as c_ulong) as *mut T;
        let set_result = apr_pool_userdata_set(
            ptr_zeroed as *mut _ as *mut c_void,
            id.as_ptr(),
            cleanup,
            pool as *mut apr_pool_t,
        );
        if set_result == (APR_SUCCESS as i32) {
            return Ok(&mut *ptr_zeroed);
        } else {
            return Err(AllocError{ layout });
        }
    }
}

pub fn find<'p, T>(
    pool: &'p mut apr_pool_t,
    user_data_key: &CString,
) -> Option<&'p mut T> {
    let mut value_ptr: *mut T = ptr::null_mut();
    unsafe {
        let get_result = apr_pool_userdata_get(
            &mut value_ptr as *mut *mut T as *mut *mut c_void,
            user_data_key.as_ptr(),
            pool
        );
        if get_result == (APR_SUCCESS as i32) {
            let existing_value = &mut (*value_ptr);
            return Some(existing_value);
        } else {
            return None;
        }
    }
}

pub trait ContainsPool {
    fn access_pool(&mut self) -> *mut *mut apr_pool_t;
}

impl ContainsPool for conn_rec {
    fn access_pool(&mut self) -> *mut *mut apr_pool_t {
        return &mut (self.pool);
    }
}

impl ContainsPool for process_rec {
    fn access_pool(&mut self) -> *mut *mut apr_pool_t {
        return &mut (self.pool);
    }
}

impl ContainsPool for request_rec {
    fn access_pool(&mut self) -> *mut *mut apr_pool_t {
        return &mut (self.pool);
    }
}

#[cfg(test)]
pub mod test_utils {
    use crate::apache2::bindings::{
        apr_initialize, apr_pool_create_ex,
    };
    use super::*;

    pub fn apr_pool_create<'p, T: ContainsPool>(record: &'p mut T) -> Result<&'p mut apr_pool_t, AllocError> {
        unsafe {
            let pool_ptr_ptr = record.access_pool();
            let result = apr_pool_create_ex(
                pool_ptr_ptr,
                ptr::null_mut(),
                None,
                ptr::null_mut()
            );
            if result == (APR_SUCCESS as i32) {
                return Ok(&mut *(*pool_ptr_ptr));
            } else {
                return Err(AllocError{ layout: Layout::new::<apr_pool_t>() });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::apache2::bindings::{
        APR_BADARG, apr_initialize, apr_pool_create_ex, apr_pool_destroy, apr_terminate,
    };
    use super::*;
    use std::boxed::Box;
    use std::error::Error;

    struct Counter {
        count: u32,
    }

    impl Counter {
        pub fn new() -> Self {
            Counter {
                count: 0
            }
        }
    }

    struct CounterWrapper<'c> {
        counter: &'c mut Counter,
    }

    #[no_mangle]
    unsafe extern fn increment_counter(wrapper_void: *mut c_void) -> apr_status_t {
        if wrapper_void == ptr::null_mut() {
            return APR_BADARG as apr_status_t;
        }
        let wrapper_ptr = wrapper_void as *mut CounterWrapper;
        let wrapper_ref = &mut *wrapper_ptr;
        wrapper_ref.counter.count += 1;
        return APR_SUCCESS as apr_status_t;
    }

    #[test]
    fn test_find_when_never_allocated() {
        let mut pool_ptr: *mut apr_pool_t = ptr::null_mut();
        unsafe {
            assert_eq!(apr_initialize(), APR_SUCCESS as i32, "Failed to init APR");
            let create_result = apr_pool_create_ex(
                &mut pool_ptr as *mut *mut apr_pool_t,
                ptr::null_mut(),
                None,
                ptr::null_mut(),
            );
            assert_eq!(create_result, APR_SUCCESS as i32, "Failed to create pool");
            let pool = &mut *pool_ptr;
            assert!(find::<Counter>(pool, &CString::new("foo").unwrap()).is_none(), "Find succeeded on empty pool");
            apr_pool_destroy(pool_ptr);
            apr_terminate();
        }
    }

    #[test]
    fn test_find_when_already_allocated() -> Result<(), Box<dyn Error>> {
        let mut counter1 = Counter::new();
        let id1 = CString::new("id1")?;
        let mut pool_ptr: *mut apr_pool_t = ptr::null_mut();
        unsafe {
            assert_eq!(apr_initialize(), APR_SUCCESS as i32, "Failed to init APR");
            let create_result = apr_pool_create_ex(
                &mut pool_ptr as *mut *mut apr_pool_t,
                ptr::null_mut(),
                None,
                ptr::null_mut(),
            );
            assert_eq!(create_result, APR_SUCCESS as i32, "Failed to create pool");
            let pool = &mut *pool_ptr;
            let wrapper1 = _alloc::<CounterWrapper>(pool, &id1, Some(increment_counter))?;
            wrapper1.counter = &mut counter1;
            assert!(find::<Counter>(pool, &id1).is_some(), "Failed to find previous allocation");
            apr_pool_destroy(pool_ptr);
            apr_terminate();
        }
        Ok(())
    }

    #[test]
    fn test_cleanup_called() -> Result<(), Box<dyn Error>> {
        let mut counter1 = Counter::new();
        let id1 = CString::new("id1")?;
        let mut pool_ptr: *mut apr_pool_t = ptr::null_mut();
        unsafe {
            assert_eq!(apr_initialize(), APR_SUCCESS as i32, "Failed to init APR");
            let create_result = apr_pool_create_ex(
                &mut pool_ptr as *mut *mut apr_pool_t,
                ptr::null_mut(),
                None,
                ptr::null_mut(),
            );
            assert_eq!(create_result, APR_SUCCESS as i32, "Failed to create pool");
            let pool = &mut *pool_ptr;
            let wrapper1 = _alloc::<CounterWrapper>(pool, &id1, Some(increment_counter))?;
            wrapper1.counter = &mut counter1;
            apr_pool_destroy(pool_ptr);
            assert_eq!(1, counter1.count, "Cleanup callback not called one time");
            apr_terminate();
        }
        Ok(())
    }
}
