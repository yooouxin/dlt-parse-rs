use crate::error::{Layer, UnexpectedEndOfSliceError, VerboseDecodeError};

/// Helper for parsing verbose messages.
pub struct FieldSlicer<'a> {
    /// Unparsed part of the verbose message.
    rest: &'a [u8],

    /// Offset since the parsing has started.
    offset: usize,
}

impl<'a> FieldSlicer<'a> {

    #[inline]
    pub fn new(data: &[u8], offset: usize) -> FieldSlicer {
        FieldSlicer {
            rest: data,
            offset,
        }
    }

    #[inline]
    pub fn rest(&self) -> &'a [u8] {
        self.rest
    }

    pub fn read_u8(&mut self) -> Result<u8, VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check length
        if self.rest.len() < 1 {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: Layer::VerboseValue,
                    minimum_size: self.offset + 1,
                    actual_size: self.offset + self.rest.len(),
                }
            ));
        }

        // SAFETY: Length of at least 1 verified in the previous if.
        let result = unsafe {
            *self.rest.get_unchecked(0)
        };

        // move slice
        // SAFETY: Length of at least 1 verified in the previous if.
        self.rest = unsafe {
            core::slice::from_raw_parts(
                self.rest.as_ptr().add(1),
                self.rest.len() - 1
            )
        };
        self.offset += 1;

        Ok(result)
    }

    pub fn read_2bytes(&mut self) -> Result<[u8;2], VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check length
        if self.rest.len() < 2 {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: Layer::VerboseValue,
                    minimum_size: self.offset + 2,
                    actual_size: self.offset + self.rest.len(),
                }
            ));
        }

        // read value
        // SAFETY: Length of at least 2 verified in the previous if.
        let result = unsafe {[
            *self.rest.get_unchecked(0),
            *self.rest.get_unchecked(1)
        ]};

        // move slice
        // SAFETY: Length of at least 2 verified in the previous if.
        self.rest = unsafe {
            core::slice::from_raw_parts(
                self.rest.as_ptr().add(2),
                self.rest.len() - 2
            )
        };
        self.offset += 2;

        Ok(result)
    }

    pub fn read_u16(&mut self, is_big_endian: bool) -> Result<u16, VerboseDecodeError> {
        self.read_2bytes().map(
            |bytes| if is_big_endian {
                u16::from_be_bytes(bytes)
            } else {
                u16::from_le_bytes(bytes)
            }
        )
    }

    pub fn read_var_name(&mut self, is_big_endian: bool) -> Result<&'a str, VerboseDecodeError> {
        use VerboseDecodeError::*;
        
        // check length
        if self.rest.len() < 2 {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: Layer::VerboseValue,
                    minimum_size: self.offset + 2,
                    actual_size: self.offset + self.rest.len(),
                }
            ));
        }
        
        // read lengths
        let name_length = {
            // SAFETY: Length of at least 2 verified in the previous if.
            let bytes = unsafe {[
                *self.rest.get_unchecked(0),
                *self.rest.get_unchecked(1)
            ]};
            if is_big_endian {
                u16::from_be_bytes(bytes) as usize
            } else {
                u16::from_le_bytes(bytes) as usize
            }
        };

        // check length of slice
        let total_size = 2 + name_length;
        if self.rest.len() < total_size {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: Layer::VerboseValue,
                    minimum_size: self.offset + total_size,
                    actual_size: self.offset + self.rest.len(),
                }
            ));
        }

        // read name
        let name = if name_length > 0 {
            // SAFETY: Length of at least 2 + name_length verified in the previous if.
            //         Additionally name_length is guranteed to be at least 1.
            let name_raw = unsafe {
                core::slice::from_raw_parts(
                    self.rest.as_ptr().add(2),
                    // substract 1 to skip the zero termination
                    name_length - 1
                )
            };
            // SAFETY: Length of at least 2 + name_length verified in the previous if.
            //         Additionally name_length is guranteed to be at least 1.
            let last = unsafe {
                *self.rest.as_ptr().add(2 + name_length - 1)
            };

            // check for zero termination
            if last != 0 {
                return Err(VariableNameStringMissingNullTermination);
            }

            core::str::from_utf8(name_raw)?
        } else {
            ""
        };

        // move slice
        // SAFETY: Length of at least total_size verfied in previous if.
        self.rest = unsafe {
            core::slice::from_raw_parts(
                self.rest.as_ptr().add(total_size),
                self.rest.len() - total_size
            )
        };
        self.offset += total_size;
        
        Ok(name)
    }

    pub fn read_var_name_and_unit(&mut self, is_big_endian: bool) -> Result<(&'a str, &'a str), VerboseDecodeError> {
        use VerboseDecodeError::*;
        
        // check length
        if self.rest.len() < 4 {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: Layer::VerboseValue,
                    minimum_size: self.offset + 4,
                    actual_size: self.offset + self.rest.len(),
                }
            ));
        }

        // read lengths
        let name_length = {
            // SAFETY: Length of at least 4 verified in the previous if.
            let bytes = unsafe {[
                *self.rest.get_unchecked(0),
                *self.rest.get_unchecked(1)
            ]};
            if is_big_endian {
                u16::from_be_bytes(bytes) as usize
            } else {
                u16::from_le_bytes(bytes) as usize
            }
        };
        let unit_length = {
            // SAFETY: Length of at least 4 verified in the previous if.
            let bytes = unsafe {[
                *self.rest.get_unchecked(2),
                *self.rest.get_unchecked(3)
            ]};
            if is_big_endian {
                u16::from_be_bytes(bytes) as usize
            } else {
                u16::from_le_bytes(bytes) as usize
            }
        };

        // check length of slice
        let total_size = 4 + name_length + unit_length;
        if self.rest.len() < total_size {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: Layer::VerboseValue,
                    minimum_size: self.offset + total_size,
                    actual_size: self.offset + self.rest.len(),
                }
            ));
        }

        // read name
        let name = if name_length > 0 {
            // SAFETY: Length of at least 4 + name_length verified in the previous if.
            //         Additionally name_length is guranteed to be at least 1.
            let name_raw = unsafe {
                core::slice::from_raw_parts(
                    self.rest.as_ptr().add(4),
                    // substract 1 to skip the zero termination
                    name_length - 1
                )
            };
            // SAFETY: Length of at least 4 + name_length verified in the previous if.
            //         Additionally name_length is guranteed to be at least 1.
            let last = unsafe {
                *self.rest.as_ptr().add(4 + name_length - 1)
            };

            // check for zero termination
            if last != 0 {
                return Err(VariableNameStringMissingNullTermination);
            }

            core::str::from_utf8(name_raw)?
        } else {
            ""
        };

        // read unit
        let unit = if unit_length > 0 {
            // SAFETY: Length of at least 4 + name_length + unit_length verified in the previous if.
            //         Additionally unit_length is guranteed to be at least 1.
            let unit_raw = unsafe {
                core::slice::from_raw_parts(
                    self.rest.as_ptr().add(4 + name_length),
                    // substract 1 to skip the zero termination
                    unit_length - 1
                )
            };
            // SAFETY: Length of at least 4 + name_length + unit_length verified in the previous if.
            //         Additionally unit_length is guranteed to be at least 1.
            let last = unsafe {
                *self.rest.as_ptr().add(4 + name_length + unit_length - 1)
            };

            // check for zero termination
            if last != 0 {
                return Err(VariableUnitStringMissingNullTermination);
            }

            core::str::from_utf8(unit_raw)?
        } else {
            ""
        };

        // move slice
        // SAFETY: Length of at least total_size verfied in previous if.
        self.rest = unsafe {
            core::slice::from_raw_parts(
                self.rest.as_ptr().add(total_size),
                self.rest.len() - total_size
            )
        };
        self.offset += total_size;

        // done
        Ok((name, unit))
    }

    pub fn read_raw(&mut self, len: usize) -> Result<&'a [u8], VerboseDecodeError> {
        use VerboseDecodeError::*;

        // check that the string length is present
        if self.rest.len() < len {
            return Err(UnexpectedEndOfSlice(
                UnexpectedEndOfSliceError{
                    layer: Layer::VerboseValue,
                    minimum_size: self.offset + len,
                    actual_size: self.offset + self.rest.len(),
                }
            ));
        }

        // SAFETY: Slice length checked above to be at least len
        let result = unsafe {
            core::slice::from_raw_parts(
                self.rest.as_ptr(),
                len
            )
        };

        // move rest & offset
        self.rest = unsafe {
            core::slice::from_raw_parts(
                self.rest.as_ptr().add(len),
                self.rest.len() - len
            )
        };
        self.offset += len;

        Ok(result)
    }
}

#[cfg(test)]
mod test_field_slicer {
    use super::*;
    use std::format;
    use proptest::prelude::*;
    use proptest::arbitrary::any;
    use proptest::collection::vec;
    use crate::error::{Layer, UnexpectedEndOfSliceError, VerboseDecodeError};
    use alloc::vec::Vec;

    proptest!{
        #[test]
        fn new(
            data in prop::collection::vec(any::<u8>(), 0..10),
            offset in any::<usize>()
        ) {
            let s = FieldSlicer::new(
                &data,
                offset
            );
            prop_assert_eq!(s.rest(), &data);
            prop_assert_eq!(s.offset, offset);
        }
    }

    proptest!{
        #[test]
        fn read_u8(
            value in any::<u8>(),
            slice_len in 1usize..3,
            offset in 0usize..usize::MAX,
        ) {
            // ok
            {
                let data = [value, 123, 234];
                let mut slicer = FieldSlicer{
                    rest: &data[..slice_len],
                    offset,
                };
                prop_assert_eq!(
                    slicer.read_u8(),
                    Ok(value)
                );
                prop_assert_eq!(slicer.rest, &data[1..slice_len]);
                prop_assert_eq!(slicer.offset, offset + 1);
            }
            // length error
            {
                let mut slicer = FieldSlicer{
                    rest: &[],
                    offset,
                };
                prop_assert_eq!(
                    slicer.read_u8(),
                    Err(VerboseDecodeError::UnexpectedEndOfSlice(
                        UnexpectedEndOfSliceError{
                            layer: Layer::VerboseValue,
                            actual_size: offset,
                            minimum_size: offset + 1,
                        }
                    ))
                );
            }
        }
    }

    proptest!{
        #[test]
        fn read_2bytes(
            value in any::<[u8;2]>(),
            slice_len in 2usize..4,
            offset in 0usize..usize::MAX-1,
            bad_len in 0usize..2,
        ) {
            // ok
            {
                let data = [value[0], value[1], 1, 2];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_2bytes(),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 2);
                prop_assert_eq!(slicer.rest, &[1,2][..slice_len - 2]);
            }

            // length error
            {
                let mut slicer = FieldSlicer::new(&value[..bad_len], offset);
                prop_assert_eq!(
                    slicer.read_2bytes(),
                    Err(VerboseDecodeError::UnexpectedEndOfSlice(
                        UnexpectedEndOfSliceError{
                            layer: Layer::VerboseValue,
                            actual_size: offset + bad_len,
                            minimum_size: offset + 2,
                        }
                    ))
                );
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &value[..bad_len]);
            }
        }
    }

    proptest!{
        #[test]
        fn read_u16(
            value in any::<u16>(),
            slice_len in 2usize..4,
            offset in 0usize..usize::MAX-1,
            bad_len in 0usize..2
        ) {

            // ok big endian
            {
                let value_be = value.to_be_bytes();
                let data = [value_be[0], value_be[1], 1, 2,];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_u16(true),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 2);
                prop_assert_eq!(slicer.rest, &[1,2][..slice_len - 2]);
            }
            // ok little endian
            {
                let value_le = value.to_le_bytes();
                let data = [
                    value_le[0], value_le[1], 1, 2,
                ];
                let mut slicer = FieldSlicer::new(&data[..slice_len], offset);
                prop_assert_eq!(
                    slicer.read_u16(false),
                    Ok(value)
                );
                prop_assert_eq!(slicer.offset, offset + 2);
                prop_assert_eq!(slicer.rest, &[1,2][..slice_len - 2]);
            }

            // length error
            {
                let expected = Err(VerboseDecodeError::UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + 2,
                    }
                ));
                let data = value.to_le_bytes();
                let mut slicer = FieldSlicer::new(&data[..bad_len], offset);

                // little endian
                prop_assert_eq!(slicer.read_u16(false), expected.clone());
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);

                // big endian
                prop_assert_eq!(slicer.read_u16(true), expected);
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &data[..bad_len]);
            }
        }
    }

    proptest!{
        #[test]
        fn read_var_name(
            ref value in "\\PC*",
            offset in 0usize..1024,
            bad_len in 0usize..1024,
            rest in vec(any::<u8>(), 0..4)
        ) {
            use VerboseDecodeError::*;
            // big endian version
            {
                let mut buffer = Vec::with_capacity(2 + value.len() + 1);
                buffer.extend_from_slice(&((value.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(value.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name(true), Ok(value.as_str()));
                prop_assert_eq!(slicer.offset, offset + 2 + value.len() + 1);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // little endian version
            {
                let mut buffer = Vec::with_capacity(2 + value.len() + 1);
                buffer.extend_from_slice(&((value.len() + 1) as u16).to_le_bytes());
                buffer.extend_from_slice(value.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name(false), Ok(value.as_str()));
                prop_assert_eq!(slicer.offset, offset + 2 + value.len() + 1);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // length error (length field)
            for len in 0..2 {
                let expected = Err(UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + len,
                        minimum_size: offset + 2,
                    }
                ));
                {
                    let data = 2u16.to_le_bytes();
                    let mut slicer = FieldSlicer::new(&data[..len], offset);
                    prop_assert_eq!(slicer.read_var_name(false), expected.clone());
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &data[..len]);
                }
                {
                    let data = 2u16.to_be_bytes();
                    let mut slicer = FieldSlicer::new(&data[..len], offset);
                    prop_assert_eq!(slicer.read_var_name(true), expected);
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &data[..len]);
                }
            }

            // length error (string value)
            if value.len() > 0 {

                // make sure the len is actually smaller
                let bad_len = if bad_len >= value.len() {
                    value.len() - 1
                } else {
                    bad_len
                };

                let expected = Err(UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + 2 + bad_len,
                        minimum_size: offset + 2 + value.len(),
                    }
                ));

                // little endian
                {
                    let mut buffer = Vec::with_capacity(2 + value.len());
                    buffer.extend_from_slice(&(value.len() as u16).to_le_bytes());
                    buffer.extend_from_slice(&value.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name(false), expected.clone());
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
                // big endian
                {
                    let mut buffer = Vec::with_capacity(2 + value.len());
                    buffer.extend_from_slice(&(value.len() as u16).to_be_bytes());
                    buffer.extend_from_slice(&value.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name(true), expected);
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
            }

            // zero termination missing
            if value.len() > 0 {
                let mut buffer = Vec::with_capacity(2 + value.len() + rest.len());
                buffer.extend_from_slice(&((value.len()) as u16).to_be_bytes());
                buffer.extend_from_slice(value.as_bytes());

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name(true), Err(VariableNameStringMissingNullTermination));

                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &buffer);
            } else {
                let mut buffer = Vec::with_capacity(2 + value.len() + rest.len());
                buffer.extend_from_slice(&0u16.to_be_bytes());
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name(true), Ok(""));

                prop_assert_eq!(slicer.offset, offset + 2);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // utf8 error
            {
                let mut buffer = Vec::with_capacity(2 + value.len() + 4 + 1 + rest.len());
                buffer.extend_from_slice(&((value.len() + 4 + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(value.as_bytes());
                // some invalid utf8 data
                buffer.extend_from_slice(&[0, 159, 146, 150]);
                buffer.push(0);
                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_var_name(true),
                    Err(Utf8(core::str::from_utf8(&buffer[2..(2 + value.len() + 4)]).unwrap_err()))
                );
            }
        }
    }

    proptest!{
        #[test]
        fn read_var_name_and_unit(
            ref name in "\\PC*",
            ref unit in "\\PC*",
            offset in 0usize..1024,
            bad_len in 0usize..1024,
            rest in vec(any::<u8>(), 0..4)
        ) {
            use VerboseDecodeError::*;

            // big endian version
            {
                let mut buffer = Vec::with_capacity(4 + name.len() + unit.len() + 2);
                buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(unit.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_var_name_and_unit(true),
                    Ok((name.as_str(), unit.as_str()))
                );
                prop_assert_eq!(slicer.offset, offset + 4 + name.len() + unit.len() + 2);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // little endian version
            {
                let mut buffer = Vec::with_capacity(4 + name.len() + unit.len() + 2);
                buffer.extend_from_slice(&((name.len() + 1) as u16).to_le_bytes());
                buffer.extend_from_slice(&((unit.len() + 1) as u16).to_le_bytes());
                buffer.extend_from_slice(name.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(unit.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_var_name_and_unit(false),
                    Ok((name.as_str(), unit.as_str()))
                );
                prop_assert_eq!(slicer.offset, offset + 4 + name.len() + unit.len() + 2);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // length error (length values)
            for len in 0..4 {
                let expected = Err(UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + len,
                        minimum_size: offset + 4,
                    }
                ));
                {
                    let data = [0, 0, 0, 0];
                    let mut slicer = FieldSlicer::new(&data[..len], offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(false), expected.clone());
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &data[..len]);
                }
                {
                    let data = [0, 0, 0, 0];
                    let mut slicer = FieldSlicer::new(&data[..len], offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(true), expected);
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &data[..len]);
                }
            }

            // length error (string name)
            {
                // make sure the len is actually smaller
                let bad_len = if bad_len > name.len() {
                    name.len()
                } else {
                    bad_len
                };

                let expected = Err(UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + 4 + bad_len,
                        minimum_size: offset + 4 + name.len() + 1 + unit.len() + 1,
                    }
                ));

                // little endian
                {
                    let mut buffer = Vec::with_capacity(4 + name.len() + unit.len());
                    buffer.extend_from_slice(&((name.len() + 1) as u16).to_le_bytes());
                    buffer.extend_from_slice(&((unit.len() + 1) as u16).to_le_bytes());
                    buffer.extend_from_slice(&name.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(false), expected.clone());
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
                // big endian
                {
                    let mut buffer = Vec::with_capacity(4 + name.len() + unit.len());
                    buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                    buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                    buffer.extend_from_slice(&name.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(true), expected);
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
            }

            // length error (string unit)
            {
                // make sure the len is actually smaller
                let bad_len = if bad_len > unit.len() {
                    unit.len()
                } else {
                    bad_len
                };

                let expected = Err(UnexpectedEndOfSlice(
                    UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + 4 + name.len() + 1 + bad_len,
                        minimum_size: offset + 4 + name.len() + 1 + unit.len() + 1,
                    }
                ));

                // little endian
                {
                    let mut buffer = Vec::with_capacity(4 + name.len() + unit.len());
                    buffer.extend_from_slice(&((name.len() + 1) as u16).to_le_bytes());
                    buffer.extend_from_slice(&((unit.len() + 1) as u16).to_le_bytes());
                    buffer.extend_from_slice(&name.as_bytes());
                    buffer.push(0);
                    buffer.extend_from_slice(&unit.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(false), expected.clone());
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
                // big endian
                {
                    let mut buffer = Vec::with_capacity(4 + name.len() + unit.len());
                    buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                    buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                    buffer.extend_from_slice(&name.as_bytes());
                    buffer.push(0);
                    buffer.extend_from_slice(&unit.as_bytes()[..bad_len]);

                    let mut slicer = FieldSlicer::new(&buffer, offset);
                    prop_assert_eq!(slicer.read_var_name_and_unit(true), expected);
                    prop_assert_eq!(slicer.offset, offset);
                    prop_assert_eq!(slicer.rest, &buffer[..]);
                }
            }

            // zero termination error (name)
            if name.len() > 0 {
                let mut buffer = Vec::with_capacity(4 + name.len() + unit.len() + 1 + rest.len());
                buffer.extend_from_slice(&(name.len() as u16).to_be_bytes());
                buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                // skip zero termination
                buffer.extend_from_slice(unit.as_bytes());
                buffer.push(0);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name_and_unit(true), Err(VariableNameStringMissingNullTermination));

                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &buffer);
            } else {
                // strings with length 0 are allowed to have no zero termination
                let mut buffer = Vec::with_capacity(4 + unit.len() + 0 + rest.len());
                buffer.extend_from_slice(&(0 as u16).to_be_bytes());
                buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                // skip name as it has len 0,
                buffer.extend_from_slice(unit.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name_and_unit(true), Ok(("", unit.as_str())));

                prop_assert_eq!(slicer.offset, offset + 4 + unit.len() + 1);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // zero termination error (unit)
            if unit.len() > 0 {
                let mut buffer = Vec::with_capacity(4 + name.len() + 1 + unit.len() + rest.len());
                buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(&(unit.len() as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(unit.as_bytes());
                // skip zero termination

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name_and_unit(true), Err(VariableUnitStringMissingNullTermination));

                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &buffer);
            } else {
                // strings with length 0 are allowed to have no zero termination
                let mut buffer = Vec::with_capacity(4 + name.len() + 1 + rest.len());
                buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(&(0 as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                buffer.push(0);
                // skip unit as it has len 0,
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_var_name_and_unit(true), Ok((name.as_str(), "")));

                prop_assert_eq!(slicer.offset, offset + 4 + name.len() + 1);
                prop_assert_eq!(slicer.rest, &rest);
            }

            // utf8 error (name)
            {
                let mut buffer = Vec::with_capacity(4 + name.len() + 4 + 1 + unit.len() + 1 + rest.len());
                buffer.extend_from_slice(&((name.len() + 4 + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(&((unit.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                // some invalid utf8 data
                buffer.extend_from_slice(&[0, 159, 146, 150]);
                buffer.push(0);
                buffer.extend_from_slice(unit.as_bytes());
                buffer.push(0);
                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_var_name_and_unit(true),
                    Err(Utf8(core::str::from_utf8(&buffer[4..(4 + name.len() + 4)]).unwrap_err()))
                );
            }

            // utf8 error (name)
            {
                let mut buffer = Vec::with_capacity(4 + name.len() + 1 + unit.len() + 4 + 1 + rest.len());
                buffer.extend_from_slice(&((name.len() + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(&((unit.len() + 4 + 1) as u16).to_be_bytes());
                buffer.extend_from_slice(name.as_bytes());
                buffer.push(0);
                buffer.extend_from_slice(unit.as_bytes());
                // some invalid utf8 data
                buffer.extend_from_slice(&[0, 159, 146, 150]);
                buffer.push(0);
                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_var_name_and_unit(true),
                    Err(Utf8(core::str::from_utf8(&buffer[(4 + name.len() + 1)..(4 + name.len() + 1 + unit.len() + 4)]).unwrap_err()))
                );
            }
        }
    }

    proptest!{
        #[test]
        fn read_raw(
            data in prop::collection::vec(any::<u8>(), 0..1024),
            offset in 0usize..1024,
            rest in prop::collection::vec(any::<u8>(), 0..10),
            bad_len in 0usize..1024,
        ) {
            // ok case
            {
                let mut buffer = Vec::with_capacity(data.len() + rest.len());
                buffer.extend_from_slice(&data);
                buffer.extend_from_slice(&rest);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(slicer.read_raw(data.len()), Ok(&data[..]));
                prop_assert_eq!(slicer.offset, offset + data.len());
                prop_assert_eq!(slicer.rest, &rest);
            }
    
            // length error
            if data.len() > 0 {

                // make sure the len is actually smaller
                let bad_len = if bad_len >= data.len() {
                    data.len() - 1
                } else {
                    bad_len
                };

                let mut buffer = Vec::with_capacity(data.len());
                buffer.extend_from_slice(&data[..bad_len]);

                let mut slicer = FieldSlicer::new(&buffer, offset);
                prop_assert_eq!(
                    slicer.read_raw(data.len()),
                    Err(VerboseDecodeError::UnexpectedEndOfSlice(UnexpectedEndOfSliceError{
                        layer: Layer::VerboseValue,
                        actual_size: offset + bad_len,
                        minimum_size: offset + data.len(),
                    }))
                );
                prop_assert_eq!(slicer.offset, offset);
                prop_assert_eq!(slicer.rest, &buffer[..]);
            }
        }
    }

}