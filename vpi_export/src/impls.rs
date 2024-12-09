use crate::{FromVpiHandle, IntoVpiHandle, Result, VpiConversionError, VpiTaskResult, __private};

macro_rules! impl_from_vpi_handle {
    ($t:ty) => {
        impl FromVpiHandle for $t {
            unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Result<Self> {
                let mut value = vpi_user::t_vpi_value {
                    format: vpi_user::vpiIntVal as i32,
                    ..Default::default()
                };
                vpi_user::vpi_get_value(handle, &mut value as *mut vpi_user::t_vpi_value);
                Ok(unsafe { value.value.integer } as $t)
            }
        }
    };
}

macro_rules! impl_into_vpi_handle {
    ($t:ty) => {
        impl IntoVpiHandle for $t {
            unsafe fn into_vpi_handle(&self, handle: vpi_user::vpiHandle) -> Result<()> {
                let mut value = vpi_user::t_vpi_value {
                    format: vpi_user::vpiIntVal as i32,
                    ..Default::default()
                };
                value.value.integer = *self as i32;
                vpi_user::vpi_put_value(
                    handle,
                    &mut value as *mut vpi_user::t_vpi_value,
                    core::ptr::null_mut(),
                    vpi_user::vpiNoDelay as i32,
                );
                Ok(())
            }
        }
    };
}

impl_from_vpi_handle!(i8);
impl_from_vpi_handle!(i16);
impl_from_vpi_handle!(i32);
impl_from_vpi_handle!(i64);

impl_from_vpi_handle!(u8);
impl_from_vpi_handle!(u16);
impl_from_vpi_handle!(u32);
impl_from_vpi_handle!(u64);

impl_into_vpi_handle!(i8);
impl_into_vpi_handle!(i16);
impl_into_vpi_handle!(i32);
impl_into_vpi_handle!(i64);

impl_into_vpi_handle!(u8);
impl_into_vpi_handle!(u16);
impl_into_vpi_handle!(u32);
impl_into_vpi_handle!(u64);

impl IntoVpiHandle for () {
    unsafe fn into_vpi_handle(&self, _handle: vpi_user::vpiHandle) -> Result<()> {
        Ok(())
    }
}

impl IntoVpiHandle for f32 {
    unsafe fn into_vpi_handle(&self, handle: vpi_user::vpiHandle) -> Result<()> {
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiRealVal as i32,
            ..Default::default()
        };
        value.value.real = *self as f64;
        vpi_user::vpi_put_value(
            handle,
            &mut value as *mut vpi_user::t_vpi_value,
            core::ptr::null_mut(),
            vpi_user::vpiNoDelay as i32,
        );
        Ok(())
    }
}

impl FromVpiHandle for &str {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Result<Self> {
        let s: &core::ffi::CStr = FromVpiHandle::from_vpi_handle(handle)?;
        s.to_str().map_err(VpiConversionError::Utf8Error)
    }
}

impl FromVpiHandle for &core::ffi::CStr {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Result<Self> {
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiStringVal as i32,
            ..Default::default()
        };
        vpi_user::vpi_get_value(handle, &mut value as *mut vpi_user::t_vpi_value);
        Ok(core::ffi::CStr::from_ptr(value.value.str_))
    }
}

impl IntoVpiHandle for &core::ffi::CStr {
    unsafe fn into_vpi_handle(&self, handle: vpi_user::vpiHandle) -> Result<()> {
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiStringVal as i32,
            ..Default::default()
        };
        value.value.str_ = self.as_ptr() as *mut i8;
        vpi_user::vpi_put_value(
            handle,
            &mut value as *mut vpi_user::t_vpi_value,
            core::ptr::null_mut(),
            vpi_user::vpiNoDelay as i32,
        );
        Ok(())
    }
}

impl __private::Sealed for () {}

impl VpiTaskResult for () {
    fn into_vpi_result(self) -> Result<()> {
        Ok(())
    }
}

impl __private::Sealed for Result<()> {}

impl VpiTaskResult for Result<()> {
    fn into_vpi_result(self) -> Result<()> {
        self
    }
}
