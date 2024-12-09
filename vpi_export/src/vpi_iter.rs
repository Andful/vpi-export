use core::net;

pub struct VpiIter(crate::vpi_user::vpiHandle);

impl VpiIter {
    pub unsafe fn new(t: crate::vpi_user::PLI_INT32, handle: crate::vpi_user::vpiHandle) -> Self {
        let args_iter = crate::vpi_user::vpi_iterate(t, handle);
        Self(args_iter)
    }
}

impl Iterator for VpiIter {
    type Item = crate::vpi_user::vpiHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == core::ptr::null_mut() {
            return None;
        }

        //Safety: proper use of function
        Some(unsafe { crate::vpi_user::vpi_scan(self.0) })
    }
}

impl Drop for VpiIter {
    fn drop(&mut self) {
        if self.0 != ::core::ptr::null_mut() {
            unsafe { crate::vpi_user::vpi_free_object(self.0) };
        }
    }
}
