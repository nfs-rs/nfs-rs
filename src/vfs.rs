use async_trait::async_trait;
use crate::error::NfsResult;
use crate::proto::nfs4::*;
use crate::xdr::*;
use std::sync::Arc;

#[async_trait]
pub trait Vfs: Send + Sync {
    // Return a stable root filehandle and simple attributes
    async fn root_fh(&self) -> NfsResult<Vec<u8>>;
    async fn getattr_root(&self, attr_request: &Vec<u32>) -> NfsResult<Vec<u8>>;
}
#[derive(Clone)]
pub struct MemVfs {
    root_fh: Vec<u8>,
}

impl MemVfs {
    pub fn new() -> Arc<Self> {
        // Minimal persistent-looking handle
        let mut fh = Vec::new();
        fh.extend_from_slice(b"nfsrs-root-fh");
        Arc::new(Self { root_fh: fh })
    }
}

#[async_trait]
impl Vfs for MemVfs {
    async fn root_fh(&self) -> NfsResult<Vec<u8>> {
        Ok(self.root_fh.clone())
    }

    async fn getattr_root(&self, attr_request: &Vec<u32>) -> NfsResult<Vec<u8>> {
        // Build fattr4 for a directory with minimal fields
        // Supported: TYPE, FH_EXPIRE_TYPE, CHANGE, SIZE, FILEHANDLE
        let mut mask_bits: Vec<u32> = Vec::new();
    let vals;
        let mut w = std::io::Cursor::new(Vec::new());

        // Helper to check if requested
        let req_has = |bit: u32| -> bool {
            let idx = (bit / 32) as usize;
            let off = bit % 32;
            if idx >= attr_request.len() { return false; }
            (attr_request[idx] & (1u32 << off)) != 0
        };

        if req_has(FATTR4_TYPE) {
            mask_bits.push(FATTR4_TYPE);
            (NF4DIR as u32).xdr_serialize(&mut w)?;
        }
        if req_has(FATTR4_FH_EXPIRE_TYPE) {
            mask_bits.push(FATTR4_FH_EXPIRE_TYPE);
            (FH4_PERSISTENT as u32).xdr_serialize(&mut w)?;
        }
        if req_has(FATTR4_CHANGE) {
            mask_bits.push(FATTR4_CHANGE);
            (0u64).xdr_serialize(&mut w)?; // changeid4
        }
        if req_has(FATTR4_SIZE) {
            mask_bits.push(FATTR4_SIZE);
            (0u64).xdr_serialize(&mut w)?; // size
        }
        if req_has(FATTR4_FILEHANDLE) {
            mask_bits.push(FATTR4_FILEHANDLE);
            // fattr4_filehandle is nfs_fh4 opaque<NFS4_FHSIZE>
            let fh = self.root_fh.clone();
            // opaque<> encoding
            let len = fh.len() as u32;
            len.xdr_serialize(&mut w)?;
            w.get_mut().extend_from_slice(&fh);
            let pad = (4 - (fh.len() % 4)) % 4;
            if pad > 0 { w.get_mut().extend_from_slice(&[0u8;3][..pad]); }
        }

        vals = w.into_inner();
        let mut out = std::io::Cursor::new(Vec::new());
        let bitmap = bitmap4_with(&mask_bits);
        encode_fattr4(&mut out, &bitmap, &vals)?;
        Ok(out.into_inner())
    }
}
