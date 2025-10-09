use nfs_rs::vfs::{MemVfs, Vfs};
use nfs_rs::proto::nfs4::*;
use nfs_rs::xdr::XdrDeserialize;

#[tokio::test]
async fn test_vfs_getattr_bitmap_respected() {
    let vfs = MemVfs::new();
    // Request TYPE and SIZE only
    let bm = bitmap4_with(&[FATTR4_TYPE, FATTR4_SIZE]);
    let fattr = vfs.getattr_root(&bm).await.unwrap();

    // fattr4 encoding: bitmap4, then attrlist4 opaque
    // Decode bitmap4 length (u32) and words to see which bits were returned
    let mut cur = std::io::Cursor::new(&fattr);
    let words = u32::xdr_deserialize(&mut cur).unwrap() as usize;
    let mut got = Vec::with_capacity(words);
    for _ in 0..words { got.push(u32::xdr_deserialize(&mut cur).unwrap()); }
    // attrlist length
    let _len = u32::xdr_deserialize(&mut cur).unwrap();

    assert_eq!(words, bm.len());
    assert_eq!(got, bm);
}
