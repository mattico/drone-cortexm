#![cfg_attr(feature = "std", allow(unreachable_code, unused_variables))]

use super::PORTS_COUNT;
use core::{
    fmt::{self, Write},
    slice,
};

const ADDRESS_BASE: usize = 0xE000_0000;

/// ITM stimulus port handle.
#[derive(Clone, Copy)]
pub struct Port {
    address: usize,
}

pub trait PortWrite: Copy {
    fn port_write(address: usize, value: Self);
}

impl Port {
    /// Creates a new ITM stimulus port handle.
    ///
    /// # Panics
    ///
    /// If `port` is more than or equal to [`PORTS_COUNT`].
    #[inline]
    pub fn new(address: u8) -> Self {
        assert!(address < PORTS_COUNT);
        Self { address: ADDRESS_BASE + (usize::from(address) << 2) }
    }

    /// Writes a sequence of bytes to the ITM stimulus port.
    ///
    /// The resulting byte sequence that will be read from the port may be
    /// interleaved with concurrent writes. See also [`Port::write`] for writing
    /// atomic byte sequences.
    #[inline]
    pub fn write_bytes(self, bytes: &[u8]) -> Self {
        fn write_slice<T: PortWrite>(port: Port, slice: &[T]) {
            for item in slice {
                port.write(*item);
            }
        }
        let mut end = bytes.len();
        if end < 4 {
            write_slice(self, bytes);
            return self;
        }
        let mut start = bytes.as_ptr() as usize;
        let mut rem = start & 0b11;
        end += start;
        if rem != 0 {
            rem = 0b100 - rem;
            write_slice(self, unsafe { slice::from_raw_parts(start as *const u8, rem) });
            start += rem;
        }
        rem = end & 0b11;
        end -= rem;
        write_slice(self, unsafe { slice::from_raw_parts(start as *const u32, end - start >> 2) });
        write_slice(self, unsafe { slice::from_raw_parts(end as *const u8, rem) });
        self
    }

    /// Writes an atomic byte sequence to the ITM stimulus port. `T` can be one
    /// of `u8`, `u16`, `u32`.
    ///
    /// Bytes are written in big-endian order. It's guaranteed that all bytes of
    /// `value` will not be split.
    #[inline]
    pub fn write<T: PortWrite>(self, value: T) -> Self {
        let Self { address } = self;
        T::port_write(address, value);
        self
    }
}

impl Write for Port {
    #[inline]
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_bytes(string.as_bytes());
        Ok(())
    }
}

impl PortWrite for u8 {
    fn port_write(address: usize, value: Self) {
        #[cfg(feature = "std")]
        return unimplemented!();
        unsafe {
            llvm_asm!("
            0:
                ldrexb r0, [$1]
                cmp r0, #0
                itt ne
                strexbne r0, $0, [$1]
                cmpne r0, #1
                beq 0b
            "   :
                : "r"(value), "r"(address as *mut Self)
                : "r0", "cc"
                : "volatile"
            );
        }
    }
}

impl PortWrite for u16 {
    fn port_write(address: usize, value: Self) {
        #[cfg(feature = "std")]
        return unimplemented!();
        unsafe {
            llvm_asm!("
            0:
                ldrexh r0, [$1]
                cmp r0, #0
                itt ne
                strexhne r0, $0, [$1]
                cmpne r0, #1
                beq 0b
            "   :
                : "r"(value), "r"(address as *mut Self)
                : "r0", "cc"
                : "volatile"
            );
        }
    }
}

impl PortWrite for u32 {
    fn port_write(address: usize, value: Self) {
        #[cfg(feature = "std")]
        return unimplemented!();
        unsafe {
            llvm_asm!("
            0:
                ldrex r0, [$1]
                cmp r0, #0
                itt ne
                strexne r0, $0, [$1]
                cmpne r0, #1
                beq 0b
            "   :
                : "r"(value), "r"(address as *mut Self)
                : "r0", "cc"
                : "volatile"
            );
        }
    }
}
