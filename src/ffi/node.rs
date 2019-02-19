use super::*;

use std::ptr;

use libc::c_void;

#[repr(C)]
pub struct Node {
    impl_ptr: *mut c_void,
    vptr: *const NodeVtbl,
}

impl Node {
    pub unsafe fn new(core: Core, vtbl: *mut NodeVtbl) -> Result<'static, Node> {
        assert!(!vtbl.is_null());
        assert!((*vtbl).is_non_null());

        let mut node = Node{ impl_ptr: ptr::null_mut(), vptr: vtbl };
        let err = ((*vtbl).create.unwrap())(core, &mut node.impl_ptr);

        if err == 0 {
            Ok(node)
        } else {
            let msg = ((*vtbl).get_err_msg.unwrap())(ptr::null(), err).into_str().unwrap();

            Err(ForeignError::new(err, msg))
        }
    }
}

impl<'a> Node {
    pub unsafe fn run(&mut self) -> Result<'a, ()> {
        let err = ((*self.vptr).run.unwrap())(self.impl_ptr);

        match self.get_err_msg(err) {
            None => Ok(()),
            Some(m) => Err(ForeignError::new(err, m))
        }
    }

    pub unsafe fn stop(&mut self) -> Result<'a, ()> {
        let err = ((*self.vptr).stop.unwrap())(self.impl_ptr);

        match self.get_err_msg(err) {
            None => Ok(()),
            Some(m) => Err(ForeignError::new(err, m))
        }
    }

    pub unsafe fn get_type(&self) -> &'a str {
        ((*self.vptr).get_type.unwrap())(self.impl_ptr).into_str().unwrap()
    }

    pub unsafe fn get_err_msg(&self, err: c_int) -> Option<&'a str> {
        if err == 0 {
            None
        } else {
            Some(((*self.vptr).get_err_msg.unwrap())(self.impl_ptr, err).into_str().unwrap())
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        let err = ((*self.vptr).destroy.unwrap())(self.impl_ptr);

        if err == 0 {
            return;
        }

        let msg = ((*self.vptr).get_err_msg.unwrap())(self.impl_ptr, err).into_str().unwrap();
        panic!("couldn't destroy Node: {} ({})", err, msg);
    }
}

#[repr(C)]
pub struct NodeVtbl {
    create: Option<extern "C" fn(Core, *mut *mut c_void) -> c_int>,
    destroy: Option<extern "C" fn(*mut c_void) -> c_int>,
    run: Option<extern "C" fn(*mut c_void) -> c_int>,
    stop: Option<extern "C" fn(*mut c_void) -> c_int>,
    get_type: Option<extern "C" fn(*const c_void) -> StrView>,
    get_err_msg: Option<extern "C" fn(*const c_void, c_int) -> StrView>,
}

impl NodeVtbl {
    pub fn is_non_null(self) -> bool {
        self.create.is_some() && self.destroy.is_some() && self.run.is_some()
            && self.stop.is_some() && self.get_type.is_some()
    }
}
