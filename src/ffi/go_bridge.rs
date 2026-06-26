use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// # Safety
/// `name` must be null or a valid, NUL-terminated C string that stays valid for
/// the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn simple_network_start_node(name: *const c_char) {
    if name.is_null() {
        return;
    }
    let c_str = unsafe { CStr::from_ptr(name) };
    if let Ok(name_str) = c_str.to_str() {
        println!("[Go Bridge] Starting node: {}", name_str);
    }
}

#[no_mangle]
pub extern "C" fn simple_network_get_version() -> *mut c_char {
    let version = "0.1.0";
    CString::new(version).unwrap().into_raw()
}

/// # Safety
/// `s` must be null or a pointer previously returned by
/// `simple_network_get_version` (i.e. from `CString::into_raw`) and not already
/// freed. Passing any other pointer is undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn simple_network_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}
