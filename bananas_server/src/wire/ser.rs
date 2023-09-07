use byteorder::{LittleEndian, WriteBytesExt};
use serde::{ser, Serialize};

use super::error::{Error, Result};
use super::ServerPacket;

struct Serializer {
    output: Vec<u8>,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ServerPacket,
{
    let mut serializer = Serializer { output: Vec::new() };

    /* Reserve room for the length and add the type. */
    serializer.output.write_u16::<LittleEndian>(0).unwrap();
    serializer.output.write_u8(T::TYPE).unwrap();

    value.serialize(&mut serializer)?;

    /* Add the actual length to the beginning. */
    let len = serializer.output.len();
    serializer.output[0] = len as u8;
    serializer.output[1] = (len >> 8) as u8;

    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output
            .write_u8(if v { 1 } else { 0 })
            .map_err(|_e| Error::WriteFailure)
    }

    fn serialize_i8(self, _v: i8) -> Result<()> {
        Err(Error::NotSupported("i8".to_string()))
    }

    fn serialize_i16(self, _v: i16) -> Result<()> {
        Err(Error::NotSupported("i16".to_string()))
    }

    fn serialize_i32(self, _v: i32) -> Result<()> {
        Err(Error::NotSupported("i32".to_string()))
    }

    fn serialize_i64(self, _v: i64) -> Result<()> {
        Err(Error::NotSupported("i64".to_string()))
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.write_u8(v).map_err(|_e| Error::WriteFailure)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output
            .write_u16::<LittleEndian>(v)
            .map_err(|_e| Error::WriteFailure)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output
            .write_u32::<LittleEndian>(v)
            .map_err(|_e| Error::WriteFailure)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output
            .write_u64::<LittleEndian>(v)
            .map_err(|_e| Error::WriteFailure)
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::NotSupported("f32".to_string()))
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::NotSupported("f64".to_string()))
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::NotSupported("char".to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        /* OpenTTD strings are nul-terminated. */
        self.output.extend_from_slice(v.as_bytes());
        self.output.push(0);
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(Error::NotSupported("bytes".to_string()))
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::NotSupported("none".to_string()))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::NotSupported("some".to_string()))
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::NotSupported("unit".to_string()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::NotSupported("unit_struct".to_string()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Err(Error::NotSupported("unit_variant".to_string()))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::NotSupported("newtype_struct".to_string()))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::NotSupported("newtype_variant".to_string()))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::NotSupported("tuple_struct".to_string()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::NotSupported("tuple_variant".to_string()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::NotSupported("map".to_string()))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::NotSupported("struct_variant".to_string()))
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::NotSupported("key".to_string()))
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::NotSupported("value".to_string()))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
