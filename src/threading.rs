use super::handle_connection;
use super::pool_lib::ThreadPool;
use std::net::{TcpListener, ToSocketAddrs};
/// 使用线程池
pub fn pool_main(addr: impl ToSocketAddrs) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let pool = ThreadPool::new(16);
    for stream in listener.incoming() {
        pool.execute(|| handle_connection(stream.unwrap()))
    }
    Ok(())
}
