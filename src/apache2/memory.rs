use crate::schema::apache2::bindings::{
    APR_SUCCESS,
    apr_palloc, apr_pool_userdata_get, apr_pool_userdata_set,
    apr_pool_t, apr_size_t, apr_status_t, memset,
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

pub type CleanUpFn = unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void) -> apr_status_t;

pub fn alloc<'p, T>(
    pool: &'p mut apr_pool_t,
    id: &CString,
    cleanup: Option<CleanUpFn>,
) -> Result<(&'p mut T, &'p mut apr_pool_t), AllocError> {
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
            return Ok((&mut *ptr_zeroed, pool));
        } else {
            return Err(AllocError{ layout });
        }
    }
}

pub fn retrieve<'p, T>(
    pool: &'p apr_pool_t,
    user_data_key: &CString,
) -> Option<&'p mut T> {
    let mut value_ptr: *mut T = ptr::null_mut();
    unsafe {
        let get_result = apr_pool_userdata_get(
            &mut value_ptr as *mut *mut T as *mut *mut c_void,
            user_data_key.as_ptr(),
            pool as *const apr_pool_t as *mut apr_pool_t
        );
        if get_result == (APR_SUCCESS as i32) {
            let existing_value = &mut (*value_ptr);
            return Some(existing_value);
        } else {
            return None;
        }
    }
}

pub fn access_pool_object<'t, T>(object_void: *mut c_void) -> Option<&'t mut T> {
    if object_void == ptr::null_mut() {
        return None;
    }
    let object_ptr = object_void as *mut T;
    let object_ref = unsafe { &mut *object_ptr };
    return Some(object_ref);
}

#[cfg(test)]
pub mod test_utils {
    use crate::schema::apache2::bindings::{
        APR_SUCCESS, apr_pool_create_ex, apr_pool_t, apr_pool_destroy,
    };
    use std::boxed::Box;
    use std::error::Error;
    use std::ops::FnOnce;
    use std::ptr;

    pub fn with_pool<F>(func: F) -> Result<(), Box<dyn Error>>
    where F: FnOnce(&mut apr_pool_t) -> Result<(), Box<dyn Error>> {
        let mut pool_ptr: *mut apr_pool_t = ptr::null_mut();
        unsafe {
            let create_result = apr_pool_create_ex(
                &mut pool_ptr as *mut *mut apr_pool_t,
                ptr::null_mut(),
                None,
                ptr::null_mut(),
            );
            assert_eq!(create_result, APR_SUCCESS as i32, "Failed to create pool");
            let pool_ref = &mut *pool_ptr;
            let func_result = func(pool_ref);
            apr_pool_destroy(pool_ptr);
            return func_result;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::with_pool;
    use crate::schema::apache2::bindings::APR_BADARG;
    use super::*;

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
    fn test_retrieve_when_never_allocated() -> Result<(), Box<dyn Error>> {
        with_pool(|pool| {
            assert!(retrieve::<Counter>(pool, &CString::new("foo").unwrap()).is_none(), "Find succeeded on empty pool");
            Ok(())
        })
    }

    #[test]
    fn test_retrieve_when_already_allocated() -> Result<(), Box<dyn Error>> {
        let mut counter1 = Counter::new();
        let id1 = CString::new("id1")?;
        with_pool(|pool| {
            let wrapper1 = alloc::<CounterWrapper>(pool, &id1, Some(increment_counter))?.0;
            wrapper1.counter = &mut counter1;
            assert!(retrieve::<Counter>(pool, &id1).is_some(), "Failed to retrieve previous allocation");
            Ok(())
        })
    }

    #[test]
    fn test_cleanup_called() -> Result<(), Box<dyn Error>> {
        let mut counter1 = Counter::new();
        let mut counter2 = Counter::new();
        let id1 = CString::new("id1")?;
        with_pool(|pool| {
            let wrapper1 = alloc::<CounterWrapper>(pool, &id1, Some(increment_counter))?.0;
            let _wrapper2 = Box::new(CounterWrapper { counter: &mut counter2 });
            wrapper1.counter = &mut counter1;
            Ok(())
        })?;
        assert_eq!(1, counter1.count, "Cleanup callback not called one time");
        assert_eq!(0, counter2.count, "Callback called on non-pool allocated value");
        Ok(())
    }

    #[test]
    fn test_multiple_allocations() -> Result<(), Box<dyn Error>> {
        let mut counter1 = Counter::new();
        let mut counter2 = Counter::new();
        let id1 = CString::new("id1")?;
        let id2 = CString::new("id2")?;
        with_pool(|pool0| {
            let (wrapper1, pool1) = alloc::<CounterWrapper>(pool0, &id1, Some(increment_counter))?;
            let (wrapper2, pool2) = alloc::<CounterWrapper>(pool1, &id2, Some(increment_counter))?;
            wrapper1.counter = &mut counter1;
            wrapper2.counter = &mut counter2;
            assert!(retrieve::<Counter>(pool2, &id1).is_some(), "Failed to retrieve previous allocation");
            assert!(retrieve::<Counter>(pool2, &id2).is_some(), "Failed to retrieve previous allocation");
            Ok(())
        })?;
        assert_eq!(1, counter1.count, "Cleanup callback not called one time");
        assert_eq!(1, counter2.count, "Cleanup callback not called one time");
        Ok(())
    }
}
