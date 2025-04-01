//! module relating to simulation

use crate::{vpi_user::vpi_handle_by_name, FromVpiHandle, Handle, Result, VpiError};
use alloc::borrow::ToOwned;
use core::{ffi::CStr, mem::MaybeUninit, ptr::NonNull};
use vvp_sys::{vvp_init, vvp_no_signals, vvp_run};

/**
 * Net
 */
pub struct Net(crate::RawHandle);

impl Net {
    /**
     * Obtain handle corresponding to the net
     */
    pub fn handle<E>(&self) -> Result<Handle<E>>
    where
        E: FromVpiHandle,
    {
        unsafe { Handle::from_vpi_handle(self.0.clone()) }
    }
}

/**
 * Module
 */
pub struct Module(crate::RawHandle);

impl Module {
    /**
     * Obtain net by name
     */
    pub fn net_by_name(&self, name: &CStr) -> Result<Net> {
        let net = NonNull::new(unsafe {
            crate::vpi_user::vpi_handle_by_name(name.as_ptr() as *mut i8, self.0.as_ptr())
        })
        .ok_or(VpiError::NoModule(c"TODO change error type".into()))?;
        Ok(Net(net))
    }
}

/**
 * Context of simulation
 */
pub struct Context<'a>(core::marker::PhantomData<&'a ()>);

impl Context<'_> {
    /**
     * Obtain module by name
     */
    pub fn module(&self, module_name: &CStr) -> crate::Result<Module> {
        let module = unsafe {
            vpi_handle_by_name(
                module_name.as_ptr() as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut(),
            )
        };
        let Some(module) = core::ptr::NonNull::new(module) else {
            return Err(VpiError::NoModule(module_name.to_owned().into()));
        };
        Ok(Module(module))
    }
    /**
     * Get simulation time
     */
    pub fn get_time(&self) -> u64 {
        crate::get_time()
    }
    /**
     * Finish simulation
     */
    pub fn finish(&self) {
        crate::finish();
    }
}

fn start_simulation_internal<F: FnMut() + Sized>(f: F) -> crate::VpiCallbackHandle {
    use alloc::alloc::{alloc, handle_alloc_error, Layout};
    let callback_layout = Layout::new::<F>();
    let raw_callback_pointer = unsafe { alloc(callback_layout) };
    let callback = raw_callback_pointer as *mut F;
    if callback.is_null() {
        handle_alloc_error(callback_layout);
    }
    unsafe {
        callback.write(f);
    }
    let data_layout = Layout::new::<crate::CallbackData>();
    let data = unsafe { alloc(data_layout) } as *mut crate::CallbackData;
    unsafe {
        data.write(crate::CallbackData {
            raw_callback_pointer,
            callback: callback as *mut dyn FnMut(),
            callback_layout,
        });
    }

    let mut cb_data = vpi_user::t_cb_data {
        reason: vpi_user::cbStartOfSimulation as i32,
        cb_rtn: Some(crate::cb),
        time: &mut vpi_user::t_vpi_time {
            type_: vpi_user::vpiSimTime as i32,
            high: 0,
            low: 0,
            ..Default::default()
        },
        user_data: data as *mut vpi_user::PLI_BYTE8,
        ..Default::default()
    };
    crate::VpiCallbackHandle(unsafe { vpi_user::vpi_register_cb(&mut cb_data) }, data)
}

/**
 * Start simulation
 */
pub fn start_simulation<R>(path: &core::ffi::CStr, f: impl FnOnce(Context<'_>) -> R) -> R {
    let mut result = MaybeUninit::<R>::uninit();
    let r = &mut result;
    let mut f = Some(f);
    start_simulation_internal(move || {
        let f = core::mem::take(&mut f).unwrap();
        r.write(f(Context(Default::default())));
    });

    unsafe {
        vvp_no_signals();
        vvp_init(core::ptr::null(), 0, core::ptr::null_mut());
        vvp_run(path.as_ptr());
    }
    unsafe { result.assume_init() }
}
