use crate::{on_delay, on_value_change, Handle, VpiCallbackHandle, VpiError};

///Clock
#[derive(Clone)]
pub struct Clk {
    value: bool,
}

impl crate::FromVpiHandle for Clk {
    unsafe fn from_vpi_handle(handle: crate::RawHandle) -> crate::Result<Self> {
        use vpi_user;
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiIntVal as i32,
            ..Default::default()
        };

        //Safety: correct use of ffi
        let size = unsafe { vpi_user::vpi_get(vpi_user::vpiSize as i32, handle.as_ptr()) } as usize;
        if size != 1 {
            return Err(VpiError::BitVectorLengthMissMatch {
                expected: 1,
                actual: size as usize,
            });
        }

        //Safety: correct use of ffi function
        unsafe {
            vpi_user::vpi_get_value(handle.as_ptr(), &mut value as *mut vpi_user::t_vpi_value);
        }
        Ok(Clk {
            value: value.value.integer != 0,
        })
    }
}

impl crate::StoreIntoVpiHandle for Clk {
    unsafe fn store_into_vpi_handle(&self, handle: crate::RawHandle) -> crate::Result<()> {
        use vpi_user;
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiIntVal as i32,
            ..Default::default()
        };

        value.value.integer = if self.value { 1 } else { 0 };

        vpi_user::vpi_put_value(
            handle.as_ptr(),
            &mut value as *mut vpi_user::t_vpi_value,
            core::ptr::null_mut(),
            vpi_user::vpiNoDelay as i32,
        );

        Ok(())
    }
}

impl Clk {
    fn to_low(mut handle: Handle<Self>, low: u64, high: u64) {
        handle.borrow_mut().unwrap().value = false;
        crate::on_delay(low, move || Clk::to_high(handle, low, high));
    }

    fn to_high(mut handle: Handle<Self>, low: u64, high: u64) {
        handle.borrow_mut().unwrap().value = true;
        crate::on_delay(high, move || Clk::to_low(handle, low, high));
    }

    ///Posedge
    pub fn on_posedge<F: FnMut() + 'static>(handle: Handle<Self>, mut f: F) -> VpiCallbackHandle {
        on_value_change(handle.clone(), move || {
            if handle.clone().get_value().unwrap().value {
                f();
            }
        })
    }

    ///Negedge
    pub fn on_negedge<F: FnMut() + 'static>(handle: Handle<Self>, mut f: F) -> VpiCallbackHandle {
        on_value_change(handle.clone(), move || {
            if !handle.clone().get_value().unwrap().value {
                f();
            }
        })
    }

    /// Start clock
    pub fn start(
        handle: Handle<Self>,
        period: u64,
        delay: u64,
    ) -> core::result::Result<(), VpiError> {
        if period < 2 {
            return Err(VpiError::PeriodTooSmall);
        }
        let handle = handle.clone();
        let low = period / 2;
        let high = period - low;
        on_delay(delay, move || {
            Self::to_high(handle, low, high);
        });
        Ok(())
    }
}
