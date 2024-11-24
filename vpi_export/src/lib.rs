#![feature(const_mut_refs)]
#![feature(maybe_uninit_uninit_array)]

#[allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
pub mod vpi_user;
use std::sync::Mutex;

pub use ctor::ctor;
pub use vpi_export_macro::vpi_export;

pub struct __FunctionCollections__ {
    pub values: Mutex<Vec<fn()>>,
}

unsafe impl Sync for __FunctionCollections__ {}

pub static __FUNCTION_COLLECTIONS__: __FunctionCollections__ = __FunctionCollections__ {
    values: Mutex::new(Vec::new()),
};

pub extern "C" fn register_vpi_functions() {
    let values = __FUNCTION_COLLECTIONS__.values.lock().unwrap();
    for e in values.iter() {
        e();
    }
}

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static vlog_startup_routines: [Option<extern "C" fn()>; 2] =
    [Some(register_vpi_functions), None];

pub trait FromVpiHandle {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self;
}

impl FromVpiHandle for i32 {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiIntVal as i32,
            ..Default::default()
        };
        vpi_user::vpi_get_value(handle, &mut value as *mut vpi_user::t_vpi_value);
        value.value.integer
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tests() {
        let t = trybuild::TestCases::new();
        t.pass("tests/*.rs");
    }
}
