use core::ops::{Deref, DerefMut};

use crate::{IntoVpiHandle, Result, VpiTaskArg, __private::Sealed};

/// Mark a value as `output`. Its value will default to [Default::default]. Changes
/// will update argument to the set value at the end of the function call.
#[repr(transparent)]
pub struct Output<'a, E>(&'a mut E)
where
    E: Default + IntoVpiHandle;

impl<E> Deref for Output<'_, E>
where
    E: Default + IntoVpiHandle,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<E> DerefMut for Output<'_, E>
where
    E: Default + IntoVpiHandle,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<E> Sealed for Output<'_, E> where E: Default + IntoVpiHandle {}

impl<'a, E> VpiTaskArg<'a> for Output<'a, E>
where
    E: Default + IntoVpiHandle,
{
    type Data = E;
    unsafe fn initialize_data(_: vpi_user::vpiHandle) -> Result<Self::Data> {
        Ok(Default::default())
    }

    fn new(e: &'a mut Self::Data) -> Self {
        Self(e)
    }

    unsafe fn finalize_data(d: Self::Data, handle: vpi_user::vpiHandle) -> Result<()> {
        d.into_vpi_handle(handle)
    }
}
