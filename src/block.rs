use super::handle_connection;
use std::net::{TcpListener, ToSocketAddrs};

pub fn block_main(addr: impl ToSocketAddrs) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        handle_connection(stream.unwrap())
    }
    Ok(())
}
