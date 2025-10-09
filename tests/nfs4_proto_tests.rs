use nfs_rs::proto::nfs4::*;
use nfs_rs::xdr::*;
use std::io::Cursor;

#[test]
fn test_compound_getattr_parse() {
    // Build a minimal COMPOUND args with one GETATTR op with bitmap bits [TYPE, FILEHANDLE]
    let tag = XdrString::from("t");
    let minor = 2u32; // e.g., v4.2 minor

    let mut buf = Cursor::new(Vec::new());
    tag.xdr_serialize(&mut buf).unwrap();
    minor.xdr_serialize(&mut buf).unwrap();
    (1u32).xdr_serialize(&mut buf).unwrap(); // one op

    (NfsOp4::OpGetattr as u32).xdr_serialize(&mut buf).unwrap();
    // op args: bitmap4 -> Vec<u32>
    let bm = bitmap4_with(&[FATTR4_TYPE, FATTR4_FILEHANDLE]);
    bm.xdr_serialize(&mut buf).unwrap();

    let args = Compound4args::xdr_deserialize(&mut Cursor::new(buf.into_inner())).unwrap();
    assert_eq!(args.operations.len(), 1);
    let op = &args.operations[0];
    assert_eq!(op.opcode, NfsOp4::OpGetattr as u32);

    let parsed_bm = nfs_rs::xdr::deserialize_from_slice::<Vec<u32>>(&op.opdata).unwrap();
    // Expect our bits set within the first word
    assert!((parsed_bm[0] & (1 << (FATTR4_TYPE % 32))) != 0);
    assert!((parsed_bm[0] & (1 << (FATTR4_FILEHANDLE % 32))) != 0);
}
