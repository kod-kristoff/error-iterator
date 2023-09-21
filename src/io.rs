use std::io;
use std::io::Write;

use crate::EIterator;

pub trait EIteratorIoExt: EIterator {
    fn write_to<W: Write>(self, mut hout: W) -> Result<(), Self::Error>
    where
        Self: EIterator<Item = u8>,
        Self: Sized,
        Self::Error: From<io::Error>,
    {
        const SIZE: usize = 4096;
        let mut buf = [0; SIZE];
        let mut i: usize = 0;

        for next in self.iter() {
            let b = next?;
            buf[i] = b;
            i += 1;

            if i == SIZE {
                hout.write_all(&buf)?;
                i = 0;
            }
        }

        if i > 0 {
            hout.write_all(&buf[..i])?;
        }

        Ok(())
    }
}

impl<I, E> EIteratorIoExt for I where I: EIterator<Item = u8, Error = E> {}
