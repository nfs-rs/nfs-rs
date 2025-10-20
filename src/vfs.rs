
use async_trait::async_trait;
use crate::error::NfsResult;
use crate::proto::nfs4::*;
use crate::xdr::*;
use std::sync::Arc;
use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH};


#[derive(Clone, Debug)]
pub struct FileAttr {
    pub changeid: u64,
    pub size: u64,
    pub mtime: u64,
    pub ctime: u64,
}

#[derive(Clone)]
pub struct MemVfs {
    root_fh: Vec<u8>,
    attrs: Arc<DashMap<String, FileAttr>>,
}

#[async_trait]
pub trait Vfs: Send + Sync {
    async fn root_fh(&self) -> NfsResult<Vec<u8>>;
    async fn getattr_root(&self, attr_request: &Vec<u32>) -> NfsResult<Vec<u8>>;
    async fn create_file(&self, path: &str, size: u64) -> NfsResult<()>;
    async fn modify_file(&self, path: &str, new_size: u64) -> NfsResult<()>;
    async fn create_dir(&self, path: &str) -> NfsResult<()>;
    async fn remove_entry(&self, path: &str) -> NfsResult<()>;
}


impl MemVfs {
    /// Public accessor for tests to get file attributes
    pub fn get_attr(&self, path: &str) -> Option<FileAttr> {
        self.attrs.get(path).map(|entry| entry.clone())
    }
    pub fn new() -> Arc<Self> {
        let mut fh = Vec::new();
        fh.extend_from_slice(b"nfsrs-root-fh");
        let attrs = DashMap::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        attrs.insert(
            "/".to_string(),
            FileAttr {
                changeid: now,
                size: 0,
                mtime: now,
                ctime: now,
            },
        );
        Arc::new(Self {
            root_fh: fh,
            attrs: Arc::new(attrs),
        })
    }

    // Batch/coalesce: only update if size or mtime would change
    fn update_attr(&self, path: &str, size: Option<u64>) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.attrs.entry(path.to_string()).and_modify(|entry| {
            let mut changed = false;
            if let Some(s) = size {
                if entry.size != s {
                    entry.size = s;
                    changed = true;
                }
            }
            if entry.mtime != now {
                entry.mtime = now;
                changed = true;
            }
            if changed {
                entry.changeid = now;
            }
        }).or_insert_with(|| FileAttr {
            changeid: now,
            size: size.unwrap_or(0),
            mtime: now,
            ctime: now,
        });
    }

    fn remove_attr(&self, path: &str) {
        self.attrs.remove(path);
    }
}


#[async_trait]
impl Vfs for MemVfs {
    async fn root_fh(&self) -> NfsResult<Vec<u8>> {
        Ok(self.root_fh.clone())
    }

    async fn getattr_root(&self, attr_request: &Vec<u32>) -> NfsResult<Vec<u8>> {
        // DashMap read lock is very fast; no blocking for other ops
        let root_attr = self.attrs.get("/").expect("root attr");
        let mut mask_bits: Vec<u32> = Vec::new();
        let mut w = std::io::Cursor::new(Vec::new());

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
            root_attr.changeid.xdr_serialize(&mut w)?;
        }
        if req_has(FATTR4_SIZE) {
            mask_bits.push(FATTR4_SIZE);
            root_attr.size.xdr_serialize(&mut w)?;
        }
        // Note: time attributes not implemented in minimal proto set
        if req_has(FATTR4_FILEHANDLE) {
            mask_bits.push(FATTR4_FILEHANDLE);
            let fh = self.root_fh.clone();
            let len = fh.len() as u32;
            len.xdr_serialize(&mut w)?;
            w.get_mut().extend_from_slice(&fh);
            let pad = (4 - (fh.len() % 4)) % 4;
            if pad > 0 { w.get_mut().extend_from_slice(&[0u8;3][..pad]); }
        }

        let vals = w.into_inner();
        let mut out = std::io::Cursor::new(Vec::new());
        let bitmap = bitmap4_with(&mask_bits);
        encode_fattr4(&mut out, &bitmap, &vals)?;
        Ok(out.into_inner())
    }

    async fn create_file(&self, path: &str, size: u64) -> NfsResult<()> {
        // Only update if needed (batch/coalesce)
        self.update_attr(path, Some(size));
        Ok(())
    }

    async fn modify_file(&self, path: &str, new_size: u64) -> NfsResult<()> {
        self.update_attr(path, Some(new_size));
        Ok(())
    }

    async fn create_dir(&self, path: &str) -> NfsResult<()> {
        self.update_attr(path, Some(0));
        Ok(())
    }

    async fn remove_entry(&self, path: &str) -> NfsResult<()> {
        self.remove_attr(path);
        Ok(())
    }

    // --- Performance tips for users ---
    // 1. For best performance, use a persistent backend (not just in-memory) for large trees.
    // 2. If your workflow allows, mount with actimeo=1 or higher to enable some attribute caching.
    // 3. For Android builds, actimeo=0 is safest but slowest; try actimeo=1 for a speed/consistency tradeoff.
    // 4. Use DashMap for concurrent access to attributes, reducing lock contention.
    // 5. Batch/coalesce updates to avoid unnecessary attribute changes.
    // 6. Profile with `perf` or similar to find further bottlenecks.
    // 7. Consider using a real filesystem backend for production workloads.
    // 8. For advanced users: implement smarter client-side attribute caching with custom NFS mount options.
}
