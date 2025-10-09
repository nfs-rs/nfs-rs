use crate::error::{NfsResult};
use crate::proto::nfs4::*;
use crate::rpc::*;
use crate::xdr::*;
use crate::vfs::{MemVfs, Vfs};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info};
use std::sync::Arc;

pub struct NfsServer {
    cfg: crate::config::NfsConfig,
    vfs: Arc<dyn Vfs>,
}

impl NfsServer {
    pub async fn new(cfg: crate::config::NfsConfig) -> NfsResult<Self> {
        Ok(Self { cfg, vfs: MemVfs::new() })
    }

    pub async fn run(self) -> NfsResult<()> {
        let addr = format!("{}:{}", self.cfg.bind_addr, self.cfg.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("NFSv4.2 server listening on {}", addr);
        run_on_listener(listener, self.vfs.clone()).await
    }
}

// Expose accept loop for tests/integration to run on a pre-bound listener
pub async fn run_on_listener(listener: TcpListener, vfs: Arc<dyn Vfs>) -> NfsResult<()> {
    loop {
        let (mut sock, peer) = listener.accept().await?;
        info!("connection from {}", peer);
        let vfs = vfs.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_conn(&mut sock, vfs).await {
                error!("conn error: {:?}", e);
            }
        });
    }
}

async fn handle_conn(sock: &mut tokio::net::TcpStream, vfs: Arc<dyn Vfs>) -> NfsResult<()> {
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        // Read a record-marked RPC message
        sock.read_exact(&mut buf[..4]).await?;
        let len_hdr = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let last = (len_hdr & (1u32 << 31)) != 0;
        let len = (len_hdr & 0x7fff_ffff) as usize;
        if !last {
            // For simplicity, expect single fragment
        }
        if buf.len() < len { buf.resize(len, 0); }
        sock.read_exact(&mut buf[..len]).await?;

        let mut cur = std::io::Cursor::new(&buf[..len]);
        let call = RpcCallHeader::xdr_deserialize(&mut cur)?;
        debug!("rpc call: {:?}", call);
    let mut reply_cur = std::io::Cursor::new(Vec::new());
        let reply_hdr = RpcReplyHeader::success(call.xid);
        reply_hdr.xdr_serialize(&mut reply_cur)?;

        if call.prog != NFS4_PROGRAM || call.vers != NFS4_VERSION {
            // RPC PROG_MISMATCH is not encoded here; just accept failure
            10007u32.xdr_serialize(&mut reply_cur)?; // NFS4ERR_BADTYPE-ish placeholder
        } else if call.proc == Nfs4Proc::Null as u32 {
            // NULL: no body, success
            // Some stacks expect empty body beyond header
        } else if call.proc == Nfs4Proc::Compound as u32 {
            // Parse COMPOUND args
            let args = Compound4args::xdr_deserialize(&mut cur)?;
            debug!("compound minor={} ops={} tag={:?}", args.minorversion, args.operations.len(), args.tag);

            // Evaluate minimal ops with current FH tracking
            let mut current_fh: Option<Vec<u8>> = None;
            // Result array header: we'll serialize after building entries
            let mut comp_res = std::io::Cursor::new(Vec::new());
            let mut overall_status = NFS4_OK;

            // Helper: write a resop header (opcode and status) and payload
            let write_resop = |w: &mut std::io::Cursor<Vec<u8>>, opcode: u32, status: u32, payload: &[u8]| -> std::io::Result<()> {
                opcode.xdr_serialize(w)?; // resop opcode
                status.xdr_serialize(w)?; // nfsstat4
                w.get_mut().extend_from_slice(payload);
                Ok(())
            };

            // Count results we'll produce
            let mut res_count: u32 = 0;
            for op in &args.operations {
                match op.opcode {
                    x if x == NfsOp4::OpPutrootfh as u32 => {
                        current_fh = Some(vfs.root_fh().await?);
                        write_resop(&mut comp_res, NfsOp4::OpPutrootfh as u32, NFS4_OK, &[])?;
                        res_count += 1;
                    }
                    x if x == NfsOp4::OpPutfh as u32 => {
                        let fh: Vec<u8> = if !op.opdata.is_empty() {
                            crate::xdr::deserialize_from_slice::<Vec<u8>>(&op.opdata).unwrap_or_default()
                        } else { vec![] };
                        current_fh = Some(fh);
                        write_resop(&mut comp_res, NfsOp4::OpPutfh as u32, NFS4_OK, &[])?;
                        res_count += 1;
                    }
                    x if x == NfsOp4::OpGetfh as u32 => {
                        if let Some(fh) = &current_fh {
                            let mut payload = std::io::Cursor::new(Vec::new());
                            let len = fh.len() as u32;
                            len.xdr_serialize(&mut payload)?;
                            payload.get_mut().extend_from_slice(fh);
                            let pad = (4 - (fh.len() % 4)) % 4;
                            if pad > 0 { payload.get_mut().extend_from_slice(&[0u8;3][..pad]); }
                            write_resop(&mut comp_res, NfsOp4::OpGetfh as u32, NFS4_OK, &payload.into_inner())?;
                            res_count += 1;
                        } else {
                            overall_status = NFS4ERR_NOFILEHANDLE;
                            write_resop(&mut comp_res, NfsOp4::OpGetfh as u32, NFS4ERR_NOFILEHANDLE, &[])?;
                            res_count += 1;
                        }
                    }
                    x if x == NfsOp4::OpGetattr as u32 => {
                        if let Some(_fh) = &current_fh {
                            let req_bitmap: Vec<u32> = if !op.opdata.is_empty() {
                                crate::xdr::deserialize_from_slice::<Vec<u32>>(&op.opdata).unwrap_or_else(|_| vec![])
                            } else { vec![] };
                            let fattr = vfs.getattr_root(&req_bitmap).await?;
                            write_resop(&mut comp_res, NfsOp4::OpGetattr as u32, NFS4_OK, &fattr)?;
                            res_count += 1;
                        } else {
                            overall_status = NFS4ERR_NOFILEHANDLE;
                            write_resop(&mut comp_res, NfsOp4::OpGetattr as u32, NFS4ERR_NOFILEHANDLE, &[])?;
                            res_count += 1;
                        }
                    }
                    _ => {
                        let sts = 10004u32; // NFS4ERR_NOTSUPP
                        overall_status = sts;
                        write_resop(&mut comp_res, op.opcode, sts, &[])?;
                        res_count += 1;
                        // Continue to next op instead of break
                    }
                }
            }

            // Now write Compound4res header and resarray
            let mut compound = std::io::Cursor::new(Vec::new());
            let cres = Compound4res { status: overall_status, tag: args.tag };
            cres.xdr_serialize(&mut compound)?;
            // resarray<>: count followed by entries (already encoded as opcode+status+payload)
            res_count.xdr_serialize(&mut compound)?;
            compound.get_mut().extend_from_slice(&comp_res.into_inner());
            reply_cur.get_mut().extend_from_slice(&compound.into_inner());
        } else {
            // Unknown proc
            10004u32.xdr_serialize(&mut reply_cur)?; // NOTSUPP
        }
        let reply_payload = reply_cur.into_inner();
        let mut framed = Vec::with_capacity(4 + reply_payload.len());
        let last = 1u32 << 31;
        let header = last | (reply_payload.len() as u32);
        framed.extend_from_slice(&header.to_be_bytes());
        framed.extend_from_slice(&reply_payload);
        sock.write_all(&framed).await?;
    }
}
