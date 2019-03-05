use super::*;

use libc::c_void;

#[repr(C)]
pub struct NodeVtbl {
    pub create: Option<unsafe extern "C" fn(Core, StrView, *mut *mut c_void) -> c_int>,
    pub destroy: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub run: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub stop: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub get_type: Option<unsafe extern "C" fn(*const c_void) -> StrView>,
    pub get_err_msg: Option<unsafe extern "C" fn(*const c_void, c_int) -> StrView>,
}
