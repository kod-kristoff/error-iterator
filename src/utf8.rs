use crate::{EIterator, Step};

pub trait EIteratorUtf8Ext: EIterator {
    fn decode_utf8(self) -> DecodeUtf8<Self>
    where
        Self: Sized,
        Self: EIterator<Item = u8>,
        Self::Error: From<DecodeUtf8Error>,
    {
        DecodeUtf8 {
            iter: self,
            count: 0,
            res: 0,
        }
    }

    fn encode_utf8(self) -> EncodeUtf8<Self>
    where
        Self: Sized,
        Self: EIterator<Item = char>,
    {
        EncodeUtf8 {
            iter: self,
            buf: [0; 4],
            index: 4,
        }
    }
}

impl<I, T, E> EIteratorUtf8Ext for I where I: EIterator<Item = T, Error = E> {}

pub struct DecodeUtf8<I> {
    iter: I,
    count: u8,
    res: u32,
}

#[derive(Debug)]
pub enum DecodeUtf8Error {
    InvalidUtf8Codepoint,
}
impl std::fmt::Display for DecodeUtf8Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DecodeUtf8Error::InvalidUtf8Codepoint => write!(fmt, "Invalid UTF8 codepoint"),
        }
    }
}
impl std::error::Error for DecodeUtf8Error {
    fn description(&self) -> &str {
        "UTF8 decode error"
    }
}

impl<I> EIterator for DecodeUtf8<I>
where
    I: EIterator<Item = u8>,
    I::Error: From<DecodeUtf8Error>,
{
    type Item = char;
    type Error = I::Error;

    fn enext(&mut self) -> Step<Self::Item, Self::Error> {
        let b = match self.iter.enext() {
            Step::Done => {
                if self.count == 0 {
                    return Step::Done;
                } else {
                    return Step::Error(From::from(DecodeUtf8Error::InvalidUtf8Codepoint));
                }
            }
            Step::Error(e) => {
                return Step::Error(e);
            }
            Step::Skip => {
                return Step::Skip;
            }
            Step::Yield(b) => b,
        };

        if self.count == 0 {
            if b & 0b1000_0000 == 0 {
                // ASCII
                Step::Yield(unsafe { std::char::from_u32_unchecked(b.into()) })
            } else {
                self.count = if b & 0b1110_0000 == 0b1100_0000 {
                    // 2 bytes
                    self.res = u32::from(b & 0b0001_1111);
                    1
                } else if b & 0b1111_0000 == 0b1110_0000 {
                    // 3 bytes
                    self.res = u32::from(b & 0b0000_1111);
                    2
                } else {
                    // 4 bytes
                    assert!(b & 0b1111_1000 == 0b1111_0000);
                    self.res = u32::from(b & 0b0000_0111);
                    3
                };
                Step::Skip
            }
        } else {
            self.count -= 1;
            self.res = (self.res << 6) | (u32::from(b) & 0b0011_1111);
            if self.count == 0 {
                Step::Yield(unsafe { std::char::from_u32_unchecked(self.res) })
            } else {
                Step::Skip
            }
        }
    }
}

pub struct EncodeUtf8<I> {
    iter: I,
    buf: [u8; 4],
    index: usize,
}

impl<I, E> EIterator for EncodeUtf8<I>
where
    I: EIterator<Item = char, Error = E>,
{
    type Item = u8;
    type Error = E;

    fn enext(&mut self) -> Step<Self::Item, Self::Error> {
        if self.index < 4 {
            let res = self.buf[self.index];
            self.index += 1;
            if self.index < 4 && self.buf[self.index] == 0 {
                self.index = 4;
            }
            Step::Yield(res)
        } else {
            match self.iter.enext() {
                Step::Done => Step::Done,
                Step::Error(e) => Step::Error(e),
                Step::Skip => Step::Skip,
                Step::Yield(c) => {
                    let len = c.encode_utf8(&mut self.buf).len();
                    if len > 1 {
                        self.index = 1;
                        if len < 4 {
                            self.buf[len] = 0;
                        }
                    }
                    Step::Yield(self.buf[0])
                }
            }
        }
    }
}
