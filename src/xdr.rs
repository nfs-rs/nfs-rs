use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

pub trait XdrSerialize {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
}

pub trait XdrDeserialize: Sized {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self>;
}

impl XdrSerialize for u32 {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_u32::<BigEndian>(*self)
    }
}
impl XdrDeserialize for u32 {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self> {
        r.read_u32::<BigEndian>()
    }
}

impl XdrSerialize for i32 {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_i32::<BigEndian>(*self)
    }
}
impl XdrDeserialize for i32 {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self> {
        r.read_i32::<BigEndian>()
    }
}

impl XdrSerialize for u64 {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_u64::<BigEndian>(*self)
    }
}
impl XdrDeserialize for u64 {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self> {
        r.read_u64::<BigEndian>()
    }
}

impl XdrSerialize for bool {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let v: u32 = if *self { 1 } else { 0 };
        v.xdr_serialize(w)
    }
}
impl XdrDeserialize for bool {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self> {
        let v = u32::xdr_deserialize(r)?;
        Ok(v != 0)
    }
}

impl XdrSerialize for Vec<u8> {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let len = self.len() as u32;
        len.xdr_serialize(w)?;
        w.write_all(self)?;
        // pad to 4-byte boundary
        let pad = (4 - (self.len() % 4)) % 4;
        if pad > 0 {
            let zeros = [0u8; 3];
            w.write_all(&zeros[..pad])?;
        }
        Ok(())
    }
}
impl XdrDeserialize for Vec<u8> {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self> {
        let len = u32::xdr_deserialize(r)? as usize;
        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf)?;
        // read and discard padding
        let pad = (4 - (len % 4)) % 4;
        if pad > 0 {
            let mut tmp = [0u8; 3];
            r.read_exact(&mut tmp[..pad])?;
        }
        Ok(buf)
    }
}

impl XdrSerialize for Vec<u32> {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let len = self.len() as u32;
        len.xdr_serialize(w)?;
        for v in self {
            v.xdr_serialize(w)?;
        }
        Ok(())
    }
}
impl XdrDeserialize for Vec<u32> {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self> {
        let len = u32::xdr_deserialize(r)? as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(u32::xdr_deserialize(r)?);
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Default)]
pub struct XdrString(pub Vec<u8>);

impl From<&str> for XdrString {
    fn from(value: &str) -> Self {
        XdrString(value.as_bytes().to_vec())
    }
}
impl From<String> for XdrString {
    fn from(value: String) -> Self {
        XdrString(value.into_bytes())
    }
}

impl XdrSerialize for XdrString {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.0.xdr_serialize(w)
    }
}
impl XdrDeserialize for XdrString {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self> {
        let v = Vec::<u8>::xdr_deserialize(r)?;
        Ok(XdrString(v))
    }
}

pub fn serialize_to_vec<T: XdrSerialize>(v: &T) -> std::io::Result<Vec<u8>> {
    let mut cur = Cursor::new(Vec::new());
    v.xdr_serialize(&mut cur)?;
    Ok(cur.into_inner())
}

pub fn deserialize_from_slice<T: XdrDeserialize>(buf: &[u8]) -> std::io::Result<T> {
    let mut cur = Cursor::new(buf);
    T::xdr_deserialize(&mut cur)
}

// Minimal encoder for fattr4: takes a bitmap4 (Vec<u32>) and raw attrlist bytes.
// Expect caller to pre-encode attr_vals in canonical order.
pub fn encode_fattr4<W: Write>(w: &mut W, attrmask: &Vec<u32>, attr_vals: &[u8]) -> std::io::Result<()> {
    attrmask.xdr_serialize(w)?; // bitmap4
    // attrlist4 is opaque<>: length + bytes + padding
    let len = attr_vals.len() as u32;
    len.xdr_serialize(w)?;
    w.write_all(attr_vals)?;
    let pad = (4 - (attr_vals.len() % 4)) % 4;
    if pad > 0 {
        let zeros = [0u8; 3];
        w.write_all(&zeros[..pad])?;
    }
    Ok(())
}
