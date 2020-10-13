use super::Server;
use crate::store;
use crossbeam_utils::sync::WaitGroup;
use std::{
    os::unix::net::UnixDatagram,
    path::PathBuf,
    sync::{
        atomic::AtomicBool,
        Arc,
    },
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("can not open entries database: {0}")]
    OpenEntriesDb(sled::Error),

    #[error("no parent directory for socket path")]
    NoSocketPathParent,

    #[error("can not create socket parent directory: {0}")]
    CreateSocketPathParent(std::io::Error),

    #[error("can not bind to socket: {0}")]
    BindSocket(std::io::Error),
}

pub struct Builder {
    pub(super) cache_dir: PathBuf,
    pub(super) data_dir: PathBuf,
    pub(super) socket: PathBuf,
}

impl Builder {
    pub fn build(self) -> Result<Server, Error> {
        let entries = sled::open(self.cache_dir.join("entries")).map_err(Error::OpenEntriesDb)?;

        let socket_path_parent = self.socket.parent().ok_or(Error::NoSocketPathParent)?;
        std::fs::create_dir_all(socket_path_parent).map_err(Error::CreateSocketPathParent)?;
        let socket = UnixDatagram::bind(&self.socket).map_err(Error::BindSocket)?;

        let store = store::new(self.data_dir);

        let stopping = Arc::new(AtomicBool::new(false));
        let wait_group = WaitGroup::new();

        Ok(Server {
            entries,
            socket,
            socket_path: self.socket,
            store,
            stopping,
            wait_group,
        })
    }
}
