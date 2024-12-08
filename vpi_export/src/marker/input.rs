use core::ops::{Deref, DerefMut};

use crate::{FromVpiHandle, Result, VpiTaskArg, __private::Sealed};

/// Mark a value as `input`. Changes on this argument will have no effect on
/// the task. This is equivalent as using the argument `E`
#[repr(transparent)]
pub struct Input<'a, E>(E, core::marker::PhantomData<&'a ()>)
where
    E: FromVpiHandle;

impl<E> Deref for Input<'_, E>
where
    E: FromVpiHandle,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for Input<'_, E>
where
    E: FromVpiHandle,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E> Input<'_, E>
where
    E: FromVpiHandle,
{
    /// Into inner data
    pub fn into_inner(self) -> E {
        self.0
    }
}

impl<E> Sealed for Input<'_, E> where E: FromVpiHandle {}

impl<'a, E> VpiTaskArg<'a> for Input<'a, E>
where
    E: FromVpiHandle,
{
    type Data = Option<E>;
    unsafe fn initialize_data(handle: vpi_user::vpiHandle) -> Result<Self::Data> {
        Ok(Some(E::from_vpi_handle(handle)?))
    }

    fn new(e: &mut Self::Data) -> Self {
        Self(core::mem::take(e).unwrap(), Default::default())
    }

    unsafe fn finalize_data(_: Self::Data, _: vpi_user::vpiHandle) -> Result<()> {
        //do nothing
        Ok(())
    }
}
