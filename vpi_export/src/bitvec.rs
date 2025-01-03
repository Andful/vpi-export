use core::fmt::Write;

use crate::{FromVpiHandle, RawHandle, StoreIntoVpiHandle, VpiError};

///Verilog bit vector type.
#[derive(Clone)]
pub struct BitVector<const N: usize>([u32; (N - 1) / 32 + 1])
where
    [u32; (N - 1) / 32 + 1]:;

impl From<u32> for BitVector<32>
where
    [u32; 1]:,
{
    fn from(value: u32) -> Self {
        Self([value])
    }
}

impl<const N: usize> BitVector<N>
where
    [u32; (N - 1) / 32 + 1]:,
{
    ///Convert from raw [u32] data
    pub fn from_raw(data: &[u32]) -> Self {
        let mut result: BitVector<N> = Default::default();
        let l = result.0.len().min(data.len());
        result.0[0..l].copy_from_slice(&data[0..l]);
        result
    }
    ///Concatenate two [BitVector] objects
    pub fn concat<const M: usize>(self, b: BitVector<M>) -> BitVector<{ N + M }>
    where
        [u32; (M - 1) / 32 + 1]:,
        [u32; (N + M - 1) / 32 + 1]:,
    {
        let mut result = BitVector::<{ N + M }>::default();
        result.0[0..b.0.len()].copy_from_slice(&b.0);
        let shift = N % 32;
        if shift == 0 {
            result.0[b.0.len()..].copy_from_slice(&self.0);
        } else {
            let mut prev = 0;
            for i in 0..self.0.len() {
                let current = self.0[i];
                result.0[b.0.len() + i - 1] |= (current << (32 - shift)) | (prev >> shift);
                prev = current;
            }
        }

        result
    }
}

impl<const N: usize> core::fmt::Debug for BitVector<N>
where
    [u32; (N - 1) / 32 + 1]:,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{N}'b"))?;
        for i in (0..self.0.len()).rev() {
            let mut bits = self.0[i];
            let mut n = 32;
            if i == self.0.len() - 1 {
                bits <<= 31 - ((N - 1) % 32);
                n = (N - 1) % 32 + 1;
            }
            for _ in 0..n {
                if (bits >> 31) > 0 {
                    f.write_char('1')?;
                } else {
                    f.write_char('0')?;
                }
                bits <<= 1;
            }
        }
        Ok(())
    }
}

impl<const N: usize> Default for BitVector<N>
where
    [u32; (N - 1) / 32 + 1]:,
{
    fn default() -> Self {
        Self([0u32; (N - 1) / 32 + 1])
    }
}

impl<const N: usize> FromVpiHandle for BitVector<N>
where
    [(); (N - 1) / 32 + 1]:,
{
    unsafe fn from_vpi_handle(handle: RawHandle) -> crate::Result<Self> {
        use vpi_user;
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiVectorVal as i32,
            ..Default::default()
        };

        //Safety: correct use of ffi
        let size = unsafe { vpi_user::vpi_get(vpi_user::vpiSize as i32, handle.as_ptr()) } as usize;
        if size != N {
            return Err(VpiError::BitVectorLengthMissMatch {
                expected: N,
                actual: size as usize,
            });
        }

        //Safety: correct use of ffi function
        unsafe {
            vpi_user::vpi_get_value(handle.as_ptr(), &mut value as *mut vpi_user::t_vpi_value);
        }
        let mut result = Self::default();
        for i in 0..result.0.len() {
            //Safety: value obtained correctly
            result.0[i] = unsafe { *(value.value.vector.add(i)) }.aval;
        }
        Ok(result)
    }
}

impl<const N: usize> StoreIntoVpiHandle for BitVector<N>
where
    [(); (N - 1) / 32 + 1]:,
{
    unsafe fn store_into_vpi_handle(&self, handle: RawHandle) -> crate::Result<()> {
        use vpi_user;
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiVectorVal as i32,
            ..Default::default()
        };
        let mut ret = [vpi_user::t_vpi_vecval::default(); N];

        for (val, e) in ret.iter_mut().zip(self.0) {
            #[allow(clippy::useless_transmute)]
            {
                val.aval = core::mem::transmute(e);
            }
        }

        value.value.vector = ret.as_mut_ptr();

        vpi_user::vpi_put_value(
            handle.as_ptr(),
            &mut value as *mut vpi_user::t_vpi_value,
            core::ptr::null_mut(),
            vpi_user::vpiNoDelay as i32,
        );

        Ok(())
    }
}
