//! Minimal NFSv4.2 XDR data types and opcodes needed for a skeleton server
use crate::xdr::*;
use num_derive::{FromPrimitive, ToPrimitive};

pub const NFS4_PROGRAM: u32 = 100003;
pub const NFS4_VERSION: u32 = 4;
pub const NFS4_OK: u32 = 0;
pub const NFS4ERR_NOFILEHANDLE: u32 = 10020;

// Minimal file types
pub const NF4REG: u32 = 1;
pub const NF4DIR: u32 = 2;

// Filehandle expire types
pub const FH4_PERSISTENT: u32 = 0x0000_0000;

// Attribute bit numbers (subset)
pub const FATTR4_SUPPORTED_ATTRS: u32 = 0; // not returned
pub const FATTR4_TYPE: u32 = 1;
pub const FATTR4_FH_EXPIRE_TYPE: u32 = 2;
pub const FATTR4_CHANGE: u32 = 3;
pub const FATTR4_SIZE: u32 = 4;
pub const FATTR4_FILEHANDLE: u32 = 19;

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum Nfs4Proc {
    Null = 0,
    // COMPOUND is proc 1 for v4.x
    Compound = 1,
}

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum NfsOp4 {
    OpAccess = 3,
    OpClose = 4,
    OpCommit = 5,
    OpCreate = 6,
    OpDelegpurge = 7,
    OpDelegreturn = 8,
    OpGetattr = 9,
    OpGetfh = 10,
    OpLink = 11,
    OpLock = 12,
    OpLockt = 13,
    OpLocku = 14,
    OpLookup = 15,
    OpLookupp = 16,
    OpNverify = 17,
    OpOpen = 18,
    OpOpenattr = 19,
    OpOpenConfirm = 20,
    OpOpenDowngrade = 21,
    OpPutfh = 22,
    OpPutpubfh = 23,
    OpPutrootfh = 24,
    OpRead = 25,
    OpReaddir = 26,
    OpReadlink = 27,
    OpRemove = 28,
    OpRename = 29,
    OpRenew = 30,
    OpRestorefh = 31,
    OpSavefh = 32,
    OpSecinfo = 33,
    OpSetattr = 34,
    OpSetclientid = 35,
    OpSetclientidConfirm = 36,
    OpVerify = 37,
    OpWrite = 38,
    OpReleaseLockowner = 39,
    // v4.1+ add many ops; v4.2 adds additional ops.
    OpAllocate = 59,
    OpCopy = 60,
    OpCopyNotify = 61,
    OpDeallocate = 62,
    OpIoAdvise = 63,
    OpLayouterror = 64,
    OpLayoutstats = 65,
    OpOffloadCancel = 66,
    OpOffloadStatus = 67,
    OpReadPlus = 68,
    OpSeek = 69,
    OpWriteSame = 70,
    OpClone = 71,
}

#[derive(Debug, Default, Clone)]
pub struct Compound4args {
    pub tag: XdrString,
    pub minorversion: u32,
    pub operations: Vec<Op4>
}

#[derive(Debug, Default, Clone)]
pub struct Compound4res {
    pub status: u32,
    pub tag: XdrString,
}

#[derive(Debug, Clone)]
pub struct Op4 {
    pub opcode: u32,
    pub opdata: Vec<u8>,
}

impl XdrDeserialize for Compound4args {
    fn xdr_deserialize<R: std::io::Read>(r: &mut R) -> std::io::Result<Self> {
        let tag = XdrString::xdr_deserialize(r)?;
        let minorversion = u32::xdr_deserialize(r)?;
        let numops = u32::xdr_deserialize(r)? as usize;
        let mut operations = Vec::with_capacity(numops);
        for _ in 0..numops {
            let opcode = u32::xdr_deserialize(r)?;
            if opcode == NfsOp4::OpGetattr as u32 {
                let bm: Vec<u32> = Vec::<u32>::xdr_deserialize(r)?;
                let opdata = crate::xdr::serialize_to_vec(&bm)?;
                operations.push(Op4 { opcode, opdata });
            } else if opcode == NfsOp4::OpPutfh as u32 {
                let fh: Vec<u8> = Vec::<u8>::xdr_deserialize(r)?;
                let opdata = crate::xdr::serialize_to_vec(&fh)?;
                operations.push(Op4 { opcode, opdata });
            } else if opcode == NfsOp4::OpLookup as u32 {
                let name: XdrString = XdrString::xdr_deserialize(r)?;
                let opdata = crate::xdr::serialize_to_vec(&name)?;
                operations.push(Op4 { opcode, opdata });
            } else if opcode == NfsOp4::OpSetattr as u32 {
                let bm: Vec<u32> = Vec::<u32>::xdr_deserialize(r)?;
                let attrs: Vec<u8> = Vec::<u8>::xdr_deserialize(r)?;
                let mut opdata = crate::xdr::serialize_to_vec(&bm)?;
                opdata.extend(crate::xdr::serialize_to_vec(&attrs)?);
                operations.push(Op4 { opcode, opdata });
            } else {
                operations.push(Op4 { opcode, opdata: vec![] });
            }
        }
        Ok(Compound4args { tag, minorversion, operations })
    }
}

impl XdrSerialize for Compound4res {
    fn xdr_serialize<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.status.xdr_serialize(w)?;
        self.tag.xdr_serialize(w)
    }
}

// Helper to build a simple bitmap4 as Vec<u32>
pub fn bitmap4_with(bits: &[u32]) -> Vec<u32> {
    // Determine number of 32-bit words needed
    let max_bit = bits.iter().copied().max().unwrap_or(0);
    let words = (max_bit as usize) / 32 + 1;
    let mut v = vec![0u32; words];
    for &b in bits {
        let idx = (b / 32) as usize;
        let off = b % 32;
        v[idx] |= 1u32 << off;
    }
    v
}
