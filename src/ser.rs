use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
};
use core::{fmt, num::FpCategory, result};
use serde::ser::{self, Impossible, Serialize};

pub enum Error {
    Message(Box<str>),
    KeyMustBeAString,
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::KeyMustBeAString => f.write_str("key must be a string"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl ser::StdError for Error {}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Message(msg.to_string().into_boxed_str())
    }
}

pub struct Serializer {
    buf: String,
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = SeqSerializer<'a>;
    type SerializeTupleStruct = SeqSerializer<'a>;
    type SerializeTupleVariant = SeqSerializer<'a>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = MapSerializer<'a>;
    type SerializeStructVariant = MapSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        if v {
            self.buf.push_str("!t");
        } else {
            self.buf.push_str("!f");
        }
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        match v.classify() {
            FpCategory::Nan | FpCategory::Infinite => self.serialize_unit(),
            _ => {
                float_to_string(&mut self.buf, v);
                Ok(())
            }
        }
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        match v.classify() {
            FpCategory::Nan | FpCategory::Infinite => self.serialize_unit(),
            _ => {
                float_to_string(&mut self.buf, v);
                Ok(())
            }
        }
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        escaped_str(&mut self.buf, v);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len())).unwrap();
        for b in v {
            seq.serialize_element(b).unwrap();
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.buf.push_str("!n");
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        self.buf.push('(');
        self.serialize_str(variant).unwrap();
        self.buf.push(':');
        value.serialize(&mut *self)?;
        self.buf.push(')');
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.buf.push_str("!(");
        Ok(SeqSerializer::new(self))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.buf.push('(');
        self.serialize_str(variant).unwrap();
        self.buf.push(':');
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer::new(self))
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.buf.push('(');
        self.serialize_str(variant).unwrap();
        self.buf.push(':');
        self.serialize_map(Some(len))
    }
}

impl<'a> ser::SerializeSeq for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.write_sep();
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<()> {
        self.ser.buf.push(')');
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.write_sep();
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        self.ser.buf.push(')');
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.write_sep();
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        self.ser.buf.push(')');
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.write_sep();
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        self.ser.buf.push_str("))");
        Ok(())
    }
}

#[doc(hidden)]
pub struct SeqSerializer<'a> {
    ser: &'a mut Serializer,
    first: bool,
}

impl<'a> SeqSerializer<'a> {
    fn new(ser: &'a mut Serializer) -> SeqSerializer<'a> {
        SeqSerializer { ser, first: true }
    }

    fn write_sep(&mut self) {
        if self.first {
            self.first = false;
        } else {
            self.ser.buf.push(',');
        }
    }
}

#[doc(hidden)]
pub struct MapSerializer<'a> {
    ser: &'a mut Serializer,
    map: BTreeMap<String, String>,
    key: Option<String>,
}

impl<'a> MapSerializer<'a> {
    fn new(ser: &'a mut Serializer) -> MapSerializer<'a> {
        MapSerializer {
            ser,
            map: BTreeMap::new(),
            key: None,
        }
    }

    fn write_object(self, end: &str) {
        self.ser.buf.push('(');
        for (i, (key, value)) in self.map.iter().enumerate() {
            if i != 0 {
                self.ser.buf.push(',');
            }
            self.ser.buf.push_str(key.as_str());
            self.ser.buf.push(':');
            self.ser.buf.push_str(value.as_str());
        }
        self.ser.buf.push_str(end);
    }
}

impl<'a> ser::SerializeMap for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        let mut ser = MapKeySerializer {
            buf: String::with_capacity(4),
        };
        key.serialize(&mut ser)?;
        self.key = Some(ser.buf);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.map.insert(self.key.take().unwrap(), to_string(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.write_object(")");
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let mut buf = String::with_capacity(key.len());
        escaped_str(&mut buf, key);
        self.map.insert(buf, to_string(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.write_object(")");
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let mut buf = String::with_capacity(key.len());
        escaped_str(&mut buf, key);
        self.map.insert(buf, to_string(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.write_object("))");
        Ok(())
    }
}

struct MapKeySerializer {
    buf: String,
}

impl<'a> ser::Serializer for &'a mut MapKeySerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        int_to_string(&mut self.buf, v);
        Ok(())
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        escaped_str(&mut self.buf, v);
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        escaped_str(&mut self.buf, variant);
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::KeyMustBeAString)
    }
}

#[inline]
fn int_to_string<I: itoa::Integer>(s: &mut String, i: I) {
    use itoa::Buffer;
    let mut buf = Buffer::new();
    s.push_str(buf.format(i));
}

#[inline]
fn float_to_string<F: ryu::Float>(s: &mut String, f: F) {
    use ryu::Buffer;
    let mut buf = Buffer::new();
    s.push_str(buf.format(f))
}

fn escaped_str(s: &mut String, value: &str) {
    let bytes = value.as_bytes();

    if bytes.is_empty() {
        s.push_str("''");
        return;
    }

    if !NOT_ID_START[bytes[0] as usize] && !bytes[1..].iter().any(|b| NOT_ID[*b as usize]) {
        s.push_str(value);
        return;
    }

    s.push('\'');
    let mut start = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b != b'!' && b != b'\'' {
            continue;
        }

        if start < i {
            s.push_str(&value[start..i]);
        }
        s.push('!');
        s.push(b.into());

        start = i + 1;
    }
    if start < bytes.len() {
        s.push_str(&value[start..]);
    }
    s.push('\'');
}

const T: bool = true;
const F: bool = false;
// Lookup table: " '!:(),*@$" are true
static NOT_ID: [bool; 256] = [
    // 1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 0
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 1
    T, T, F, F, T, F, F, T, T, T, T, F, T, F, F, F, // 2
    F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, F, // 3
    T, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 4
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 5
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 6
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 7
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 8
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 9
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // a
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // b
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // c
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // d
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // e
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // f
];
// Lookup table: "-0123456789 '!:(),*@$" are true
static NOT_ID_START: [bool; 256] = [
    // 1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 0
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 1
    T, T, F, F, T, F, F, T, T, T, T, F, T, T, F, F, // 2
    T, T, T, T, T, T, T, T, T, T, T, F, F, F, F, F, // 3
    T, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 4
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 5
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 6
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 7
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 8
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // 9
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // a
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // b
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // c
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // d
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // e
    F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, // f
];

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let mut ser = Serializer {
        buf: String::with_capacity(16),
    };
    value.serialize(&mut ser)?;
    Ok(ser.buf)
}
