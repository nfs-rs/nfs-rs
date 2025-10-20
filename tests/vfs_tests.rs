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

#[tokio::test]
async fn test_vfs_create_and_modify_file() {
    let vfs = MemVfs::new();
    let path = "/testfile";
    vfs.create_file(path, 123).await.unwrap();
    vfs.modify_file(path, 456).await.unwrap();
    let attrs = vfs.get_attr(path).unwrap();
    assert_eq!(attrs.size, 456);
}

#[tokio::test]
async fn test_vfs_create_and_remove_dir() {
    let vfs = MemVfs::new();
    let path = "/testdir";
    vfs.create_dir(path).await.unwrap();
    assert!(vfs.get_attr(path).is_some());
    vfs.remove_entry(path).await.unwrap();
    assert!(vfs.get_attr(path).is_none());
}

#[tokio::test]
async fn test_vfs_concurrent_updates() {
    use std::sync::Arc;
    use tokio::task;
    let vfs = MemVfs::new();
    let path = "/concurrent";
    let vfs = Arc::new(vfs);
    let mut handles = vec![];
    for i in 0..10 {
        let vfs = vfs.clone();
        let path = path.to_string();
        handles.push(task::spawn(async move {
            vfs.create_file(&path, i).await.unwrap();
        }));
    }
    for h in handles { h.await.unwrap(); }
    let attrs = vfs.get_attr(path).unwrap();
    // Last update wins
    assert_eq!(attrs.size, 9);
}
