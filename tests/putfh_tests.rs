use nfs_rs::proto::nfs4::*;
use nfs_rs::xdr::*;
use std::io::Cursor;

#[test]
fn test_putfh_parse_roundtrip() {
    // Build COMPOUND args with one PUTFH op carrying a handle
    let tag = XdrString::from("t");
    let minor = 2u32;

    let fh: Vec<u8> = b"fake-handle".to_vec();

    let mut buf = Cursor::new(Vec::new());
    tag.xdr_serialize(&mut buf).unwrap();
    minor.xdr_serialize(&mut buf).unwrap();
    (1u32).xdr_serialize(&mut buf).unwrap(); // one op

    (NfsOp4::OpPutfh as u32).xdr_serialize(&mut buf).unwrap();
    fh.xdr_serialize(&mut buf).unwrap(); // nfs_fh4 opaque<>

    let args = Compound4args::xdr_deserialize(&mut Cursor::new(buf.into_inner())).unwrap();
    assert_eq!(args.operations.len(), 1);
    let op = &args.operations[0];
    assert_eq!(op.opcode, NfsOp4::OpPutfh as u32);

    let parsed_fh = nfs_rs::xdr::deserialize_from_slice::<Vec<u8>>(&op.opdata).unwrap();
    assert_eq!(parsed_fh, fh);
}
