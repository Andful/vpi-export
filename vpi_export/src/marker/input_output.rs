use core::ops::{Deref, DerefMut};

use crate::{FromVpiHandle, IntoVpiHandle, Result, VpiTaskArg, __private::Sealed};

/// Mark a value as `inout`. Changes
/// will update argument to the set value at the end of the function call.
pub struct InputOutput<'a, E>(&'a mut E)
where
    E: IntoVpiHandle + FromVpiHandle;

impl<E> Deref for InputOutput<'_, E>
where
    E: IntoVpiHandle + FromVpiHandle,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<E> DerefMut for InputOutput<'_, E>
where
    E: IntoVpiHandle + FromVpiHandle,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'a, E> Sealed for InputOutput<'a, E> where E: IntoVpiHandle + FromVpiHandle {}

impl<'a, E> VpiTaskArg<'a> for InputOutput<'a, E>
where
    E: IntoVpiHandle + FromVpiHandle,
{
    type Data = E;
    unsafe fn initialize_data(handle: vpi_user::vpiHandle) -> Result<Self::Data> {
        E::from_vpi_handle(handle)
    }

    fn new(e: &'a mut Self::Data) -> Self {
        Self(e)
    }

    unsafe fn finalize_data(d: Self::Data, handle: vpi_user::vpiHandle) -> Result<()> {
        d.into_vpi_handle(handle)
    }
}
