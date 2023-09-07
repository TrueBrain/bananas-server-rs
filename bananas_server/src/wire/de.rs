use byteorder::{LittleEndian, ReadBytesExt};
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, SeqAccess, VariantAccess, Visitor,
};
use serde::Deserialize;

use super::error::{Error, Result};

struct Deserializer<'de> {
    input: &'de [u8],
    seq_expected: bool,
    seq_length: usize,
}

impl<'de> Deserializer<'de> {
    fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input,
            seq_expected: false,
            seq_length: 0,
        }
    }
}

pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::PacketTooLong)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("any".to_string()))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.input
            .read_u8()
            .map_err(|_e| Error::PacketTooShort)
            .and_then(|x| match x {
                0 => visitor.visit_bool(false),
                1 => visitor.visit_bool(true),
                _ => Err(Error::ValueOutOfRange),
            })
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("i8".to_string()))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("i16".to_string()))
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("i32".to_string()))
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("i64".to_string()))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.input.read_u8().map_err(|_e| Error::PacketTooShort)?;
        if self.seq_expected {
            self.seq_length = value as usize;
        }
        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self
            .input
            .read_u16::<LittleEndian>()
            .map_err(|_e| Error::PacketTooShort)?;
        if self.seq_expected {
            self.seq_length = value as usize;
        }
        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self
            .input
            .read_u32::<LittleEndian>()
            .map_err(|_e| Error::PacketTooShort)?;
        if self.seq_expected {
            self.seq_length = value as usize;
        }
        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self
            .input
            .read_u64::<LittleEndian>()
            .map_err(|_e| Error::PacketTooShort)?;
        if self.seq_expected {
            self.seq_length = value as usize;
        }
        visitor.visit_u64(value)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("f32".to_string()))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("f64".to_string()))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("char".to_string()))
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("str".to_string()))
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        /* OpenTTD's strings are nul-terminated; find the first zero. */
        let mut len = 0;
        for i in 0..self.input.len() {
            if self.input[i] == 0 {
                len = i;
                break;
            }
        }
        /* If we didn't find any, the packet is broken. */
        if len == 0 {
            return Err(Error::PacketTooShort);
        }

        /* Convert to UTF-8. */
        let s = std::str::from_utf8(&self.input[..len]).map_err(|_e| Error::InvalidString)?;

        /* Remove string from buffer. */
        self.input = &self.input[len + 1..];

        _visitor.visit_string(s.to_string())
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("bytes".to_string()))
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("byte_buf".to_string()))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        /* Option elements can only be at the end of a struct. */
        if self.input.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("unit_struct".to_string()))
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if !self.seq_expected || self.seq_length == 0 {
            return Err(Error::InvalidSeq);
        }
        self.seq_expected = false;

        let len = self.seq_length;
        self.seq_length = 0;

        visitor.visit_seq(ProtocolSeqAccess { de: &mut self, len })
    }

    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(ProtocolSeqAccess { de: &mut self, len })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("tuple_struct".to_string()))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("map".to_string()))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if name == "DeVecLen" {
            self.seq_expected = true;
        }

        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        mut self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if name == "ClientPacket" {
            visitor.visit_enum(PacketEnum { de: &mut self })
        } else {
            Err(Error::NotSupported("enum".to_string()))
        }
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("identifier".to_string()))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("ignored_any".to_string()))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

struct ProtocolSeqAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}

impl<'de, 'a> SeqAccess<'de> for ProtocolSeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.len > 0 {
            self.len -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct PacketEnum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> EnumAccess<'de> for PacketEnum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        /* This is called on ClientPacket enums. OpenTTD's protocol always
         * start with a single u8, indicating the type. The enum has all the
         * fields named for the number they represent. */
        let idx = u8::deserialize(&mut *self.de)?.to_string();
        let val = seed.deserialize(idx.into_deserializer())?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for PacketEnum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(self.de, "", fields, visitor)
    }
}
