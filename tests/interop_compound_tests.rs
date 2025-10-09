use nfs_rs::rpc::*;
use nfs_rs::proto::nfs4::*;
use nfs_rs::xdr::*;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Compose a COMPOUND: PUTROOTFH -> GETFH -> GETATTR(TYPE) -> LOOKUP("foo") -> SETATTR
fn build_compound_putrootfh_getfh_getattr_lookup_setattr() -> Vec<u8> {
    let mut cur = std::io::Cursor::new(Vec::new());
    // RPC Call header
    let call = RpcCallHeader { xid: 1, msg_type: RpcMessageType::Call, rpcvers: 2, prog: NFS4_PROGRAM, vers: NFS4_VERSION, proc: Nfs4Proc::Compound as u32 };
    call.xdr_serialize(&mut cur).unwrap();

    // COMPOUND args
    let tag: XdrString = "t".into();
    let minor = 2u32;
    tag.xdr_serialize(&mut cur).unwrap();
    minor.xdr_serialize(&mut cur).unwrap();
    (5u32).xdr_serialize(&mut cur).unwrap();

    // PUTROOTFH (no args)
    (NfsOp4::OpPutrootfh as u32).xdr_serialize(&mut cur).unwrap();

    // GETFH (no args)
    (NfsOp4::OpGetfh as u32).xdr_serialize(&mut cur).unwrap();

    // GETATTR with bitmap TYPE only
    (NfsOp4::OpGetattr as u32).xdr_serialize(&mut cur).unwrap();
    let bm = bitmap4_with(&[FATTR4_TYPE]);
    bm.xdr_serialize(&mut cur).unwrap();

    // LOOKUP("foo")
    (NfsOp4::OpLookup as u32).xdr_serialize(&mut cur).unwrap();
    let name: XdrString = "foo".into();
    name.xdr_serialize(&mut cur).unwrap();

    // SETATTR (empty bitmap, no attrs)
    (NfsOp4::OpSetattr as u32).xdr_serialize(&mut cur).unwrap();
    let empty_bm: Vec<u32> = vec![];
    empty_bm.xdr_serialize(&mut cur).unwrap();
    let empty_attrs: Vec<u8> = vec![];
    empty_attrs.xdr_serialize(&mut cur).unwrap();

    cur.into_inner()
}

#[tokio::test]
async fn test_compound_roundtrip() {
    // Bind an ephemeral port and run server accept loop on it
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_task = {
        let vfs_server = nfs_rs::vfs::MemVfs::new();
        // Use the accept loop directly
        tokio::spawn(async move {
            nfs_rs::server::run_on_listener(listener, vfs_server).await.unwrap();
        })
    };

    // Connect as a tiny client and send our record-marked RPC
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let payload = build_compound_putrootfh_getfh_getattr_lookup_setattr();
    let mut framed = Vec::new();
    write_record_marked(&mut framed, &payload).unwrap();
    stream.write_all(&framed).await.unwrap();

    // Read one record-marked reply
    let mut hdr = [0u8;4];
    stream.read_exact(&mut hdr).await.unwrap();
    let len_hdr = u32::from_be_bytes(hdr);
    let len = (len_hdr & 0x7fff_ffff) as usize;
    let mut buf = vec![0u8; len];
    let n = stream.read_exact(&mut buf).await;
    assert!(n.is_ok(), "Failed to read reply payload");

    // Parse reply header and compound result, but tolerate short reads
    let mut cur = std::io::Cursor::new(&buf);
    let _reply_hdr = RpcReplyHeader::xdr_deserialize(&mut cur).unwrap();
    let _status = u32::xdr_deserialize(&mut cur).unwrap();
    let _tag = Vec::<u8>::xdr_deserialize(&mut cur).unwrap();
    let count = u32::xdr_deserialize(&mut cur).unwrap();
    assert_eq!(count, 5);
    let expected = [
        (NfsOp4::OpPutrootfh as u32, NFS4_OK),
        (NfsOp4::OpGetfh as u32, NFS4_OK),
        (NfsOp4::OpGetattr as u32, NFS4_OK),
        (NfsOp4::OpLookup as u32, 10004u32),
        (NfsOp4::OpSetattr as u32, 10004u32),
    ];
    for (i, (exp_op, exp_st)) in expected.iter().enumerate() {
        let op = u32::xdr_deserialize(&mut cur).unwrap();
        assert_eq!(op, *exp_op, "op {}: expected {}, got {}", i, exp_op, op);
        let st = u32::xdr_deserialize(&mut cur).unwrap();
        assert_eq!(st, *exp_st, "status {}: expected {}, got {}", i, exp_st, st);
    }
    // Cleanup server task
    server_task.abort();
}
