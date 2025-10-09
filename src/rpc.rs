use crate::xdr::*;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

// ONC RPC over TCP record marking standard
pub fn write_record_marked(mut w: impl Write, payload: &[u8]) -> std::io::Result<()> {
    let len = payload.len() as u32;
    let last = 1u32 << 31;
    let header = last | len;
    w.write_u32::<BigEndian>(header)?;
    w.write_all(payload)?;
    Ok(())
}

pub fn read_record_marked(mut r: impl Read) -> std::io::Result<Vec<u8>> {
    let hdr = r.read_u32::<BigEndian>()?;
    let _last = (hdr & (1u32 << 31)) != 0;
    let len = hdr & 0x7fff_ffff;
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

#[derive(Debug, Clone, Copy)]
pub enum RpcMessageType {
    Call = 0,
    Reply = 1,
}

impl XdrSerialize for RpcMessageType {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let v = match self { RpcMessageType::Call => 0u32, RpcMessageType::Reply => 1u32 };
        v.xdr_serialize(w)
    }
}
impl XdrDeserialize for RpcMessageType {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self> {
        let v = u32::xdr_deserialize(r)?;
        match v {
            0 => Ok(RpcMessageType::Call),
            1 => Ok(RpcMessageType::Reply),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid rpc msg type")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RpcCallHeader {
    pub xid: u32,
    pub msg_type: RpcMessageType,
    pub rpcvers: u32,
    pub prog: u32,
    pub vers: u32,
    pub proc: u32,
}

impl XdrSerialize for RpcCallHeader {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.xid.xdr_serialize(w)?;
        self.msg_type.xdr_serialize(w)?;
        self.rpcvers.xdr_serialize(w)?;
        self.prog.xdr_serialize(w)?;
        self.vers.xdr_serialize(w)?;
        self.proc.xdr_serialize(w)?;
        // auth flavor: AUTH_NULL
        0u32.xdr_serialize(w)?; // auth flavor
        0u32.xdr_serialize(w)?; // auth length
        // verf: AUTH_NULL
        0u32.xdr_serialize(w)?;
        0u32.xdr_serialize(w)?;
        Ok(())
    }
}
impl XdrDeserialize for RpcCallHeader {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self> {
        let xid = u32::xdr_deserialize(r)?;
        let msg_type = RpcMessageType::xdr_deserialize(r)?;
        let rpcvers = u32::xdr_deserialize(r)?;
        let prog = u32::xdr_deserialize(r)?;
        let vers = u32::xdr_deserialize(r)?;
        let proc = u32::xdr_deserialize(r)?;
        // credential
        let _auth_flavor = u32::xdr_deserialize(r)?;
        let auth_len = u32::xdr_deserialize(r)? as usize;
        if auth_len > 0 {
            let mut tmp = vec![0u8; auth_len];
            r.read_exact(&mut tmp)?;
            let pad = (4 - (auth_len % 4)) % 4;
            if pad > 0 {
                let mut tmp2 = [0u8; 3];
                r.read_exact(&mut tmp2[..pad])?;
            }
        }
        // verifier
        let _verf_flavor = u32::xdr_deserialize(r)?;
        let verf_len = u32::xdr_deserialize(r)? as usize;
        if verf_len > 0 {
            let mut tmp = vec![0u8; verf_len];
            r.read_exact(&mut tmp)?;
            let pad = (4 - (verf_len % 4)) % 4;
            if pad > 0 {
                let mut tmp2 = [0u8; 3];
                r.read_exact(&mut tmp2[..pad])?;
            }
        }
        Ok(RpcCallHeader { xid, msg_type, rpcvers, prog, vers, proc })
    }
}

#[derive(Debug, Clone)]
pub struct RpcReplyHeader {
    pub xid: u32,
    pub msg_type: RpcMessageType,
    pub reply_state: u32, // MSG_ACCEPTED = 0
    pub verf_flavor: u32, // AUTH_NULL = 0
    pub verf_len: u32,
    pub accept_state: u32, // SUCCESS = 0
}

impl RpcReplyHeader {
    pub fn success(xid: u32) -> Self {
        RpcReplyHeader {
            xid,
            msg_type: RpcMessageType::Reply,
            reply_state: 0,
            verf_flavor: 0,
            verf_len: 0,
            accept_state: 0,
        }
    }
}

impl XdrSerialize for RpcReplyHeader {
    fn xdr_serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.xid.xdr_serialize(w)?;
        self.msg_type.xdr_serialize(w)?;
        self.reply_state.xdr_serialize(w)?;
        self.verf_flavor.xdr_serialize(w)?;
        self.verf_len.xdr_serialize(w)?;
        self.accept_state.xdr_serialize(w)?;
        Ok(())
    }
}

impl XdrDeserialize for RpcReplyHeader {
    fn xdr_deserialize<R: Read>(r: &mut R) -> std::io::Result<Self> {
        let xid = u32::xdr_deserialize(r)?;
        let msg_type = RpcMessageType::xdr_deserialize(r)?;
        let reply_state = u32::xdr_deserialize(r)?;
        let verf_flavor = u32::xdr_deserialize(r)?;
        let verf_len = u32::xdr_deserialize(r)?;
        if verf_len > 0 {
            let mut tmp = vec![0u8; verf_len as usize];
            r.read_exact(&mut tmp)?;
            let pad = (4 - ((verf_len as usize) % 4)) % 4;
            if pad > 0 {
                let mut tmp2 = [0u8; 3];
                r.read_exact(&mut tmp2[..pad])?;
            }
        }
        let accept_state = u32::xdr_deserialize(r)?;
        Ok(RpcReplyHeader { xid, msg_type, reply_state, verf_flavor, verf_len, accept_state })
    }
}
