use crate::RawHandle;

///Wrapper around a [crate::vpi_user::vpiHandle] generate by [crate::vpi_user::vpi_iterate].
pub struct VpiIter(Option<RawHandle>);

impl VpiIter {
    ///Equivalend to [crate::vpi_user::vpi_iterate], returning a wrapped.
    /// # Safety
    /// `t` must be valid and `handle` must be valid, not dangling or null
    pub unsafe fn new(t: crate::vpi_user::PLI_INT32, handle: crate::vpi_user::vpiHandle) -> Self {
        let args_iter = crate::vpi_user::vpi_iterate(t, handle);
        Self(RawHandle::new(args_iter))
    }
}

impl Iterator for VpiIter {
    type Item = crate::RawHandle;

    fn next(&mut self) -> Option<Self::Item> {
        //Safety: proper use of function
        crate::RawHandle::new(unsafe { crate::vpi_user::vpi_scan(self.0?.as_ptr()) })
    }
}

impl Drop for VpiIter {
    fn drop(&mut self) {
        if let Some(p) = self.0 {
            unsafe { crate::vpi_user::vpi_free_object(p.as_ptr()) };
        }
    }
}
