//! Error types for the NFS server

use thiserror::Error;

pub type NfsResult<T> = Result<T, NfsError>;

#[derive(Error, Debug)]
pub enum NfsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("XDR encoding/decoding error: {0}")]
    Xdr(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Not found")]
    NotFound,

    #[error("Already exists")]
    AlreadyExists,

    #[error("Operation not supported")]
    NotSupported,

    #[error("No space left on device")]
    NoSpace,

    #[error("Read-only file system")]
    ReadOnlyFs,

    #[error("Stale file handle")]
    StaleHandle,

    #[error("Bad stateid")]
    BadStateid,

    #[error("Grace period")]
    Grace,

    #[error("Server fault")]
    ServerFault,

    #[error("Network error: {0}")]
    Network(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),
}

/// NFS v4.2 status codes (from RFC 7862)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Nfs4Status {
    Ok = 0,
    Perm = 1,
    Noent = 2,
    Io = 5,
    Nxio = 6,
    Access = 13,
    Exist = 17,
    Xdev = 18,
    Nodev = 19,
    Notdir = 20,
    Isdir = 21,
    Inval = 22,
    Fbig = 27,
    Nospc = 28,
    Rofs = 30,
    Mlink = 31,
    Nametoolong = 63,
    Notempty = 66,
    Dquot = 69,
    Stale = 70,
    Badhandle = 10001,
    BadCookie = 10003,
    Notsupp = 10004,
    Toosmall = 10005,
    Serverfault = 10006,
    Badtype = 10007,
    Delay = 10008,
    SameSession = 10018,

    // NFSv4.2 specific errors
    BadLabel = 10093,
    OffloadDenied = 10091,
    OffloadNoReqs = 10094,
    PartnerNoAuth = 10089,
    PartnerNotsupp = 10088,
    UnionNotsupp = 10090,
    WrongLfs = 10092,
}

impl From<NfsError> for Nfs4Status {
    fn from(error: NfsError) -> Self {
        match error {
            NfsError::Io(_) => Nfs4Status::Io,
            NfsError::PermissionDenied => Nfs4Status::Access,
            NfsError::NotFound => Nfs4Status::Noent,
            NfsError::AlreadyExists => Nfs4Status::Exist,
            NfsError::NotSupported => Nfs4Status::Notsupp,
            NfsError::NoSpace => Nfs4Status::Nospc,
            NfsError::ReadOnlyFs => Nfs4Status::Rofs,
            NfsError::StaleHandle => Nfs4Status::Stale,
            NfsError::BadStateid => Nfs4Status::Badhandle,
            NfsError::InvalidArgument(_) => Nfs4Status::Inval,
            _ => Nfs4Status::Serverfault,
        }
    }
}
