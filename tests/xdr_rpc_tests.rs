use nfs_rs::xdr::*;
use nfs_rs::rpc::*;
use std::io::Cursor;

#[test]
fn test_xdr_vec_u8_roundtrip() {
    let data = vec![1u8, 2, 3, 4, 5];
    let mut cur = Cursor::new(Vec::new());
    data.xdr_serialize(&mut cur).unwrap();
    let buf = cur.into_inner();

    let mut rd = Cursor::new(&buf);
    let back = Vec::<u8>::xdr_deserialize(&mut rd).unwrap();
    assert_eq!(back, data);
}

#[test]
fn test_xdr_string_roundtrip() {
    let s = XdrString::from("hello");
    let mut cur = Cursor::new(Vec::new());
    s.xdr_serialize(&mut cur).unwrap();

    let mut rd = Cursor::new(cur.into_inner());
    let back = XdrString::xdr_deserialize(&mut rd).unwrap();
    assert_eq!(back.0, b"hello".to_vec());
}

#[test]
fn test_record_mark_roundtrip() {
    let payload = b"abcdefg".to_vec();
    let mut cur = Cursor::new(Vec::new());
    write_record_marked(&mut cur, &payload).unwrap();

    let mut rd = Cursor::new(cur.into_inner());
    let out = read_record_marked(&mut rd).unwrap();
    assert_eq!(out, payload);
}

#[test]
fn test_rpc_headers_roundtrip() {
    let call = RpcCallHeader { xid: 42, msg_type: RpcMessageType::Call, rpcvers: 2, prog: 100003, vers: 4, proc: 1 };
    let mut cur = Cursor::new(Vec::new());
    call.xdr_serialize(&mut cur).unwrap();

    let mut rd = Cursor::new(cur.into_inner());
    let back = RpcCallHeader::xdr_deserialize(&mut rd).unwrap();
    assert_eq!(back.xid, 42);
    assert_eq!(matches!(back.msg_type, RpcMessageType::Call), true);
    assert_eq!(back.rpcvers, 2);
    assert_eq!(back.prog, 100003);
    assert_eq!(back.vers, 4);
    assert_eq!(back.proc, 1);

    let reply = RpcReplyHeader::success(99);
    let mut cur2 = Cursor::new(Vec::new());
    reply.xdr_serialize(&mut cur2).unwrap();
    // No deserialize for reply; just ensures serialize path works
    assert!(!cur2.into_inner().is_empty());
}
