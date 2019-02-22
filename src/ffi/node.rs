use super::*;

use libc::c_void;

#[repr(C)]
pub struct NodeVtbl {
    pub create: Option<extern "C" fn(Core, *mut *mut c_void) -> c_int>,
    pub destroy: Option<extern "C" fn(*mut c_void) -> c_int>,
    pub run: Option<extern "C" fn(*mut c_void) -> c_int>,
    pub stop: Option<extern "C" fn(*mut c_void) -> c_int>,
    pub get_type: Option<extern "C" fn(*const c_void) -> StrView>,
    pub get_err_msg: Option<extern "C" fn(*const c_void, c_int) -> StrView>,
}
