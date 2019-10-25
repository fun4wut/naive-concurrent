use super::block::handle_connection;
use crate::pool_lib::{MPMCThreadPool, NaiveThreadPool, ScheduledThreadPool, BasicThreadPool};
use crate::{ADDR, POOL_SIZE};
use std::net::TcpListener;

/// 使用线程池
pub fn pool_main(pool: impl BasicThreadPool) -> std::io::Result<()> {
    let listener = TcpListener::bind(ADDR)?;
    for stream in listener.incoming() {
        pool.execute(|| handle_connection(stream.unwrap()))
    }
    Ok(())
}
/// 使用宏来将生成字符串转换成对应的线程池的函数，
/// 不能使用函数，是因为trait object不能有泛型参数
#[macro_export]
macro_rules! make_convert_fn {
    ($s:ident, $t:ty) => {
        #[inline]
        pub fn $s() -> $t {
            <$t>::new(POOL_SIZE)
        }
    };
}
make_convert_fn! {naive, NaiveThreadPool}
make_convert_fn! {schedule, ScheduledThreadPool}
make_convert_fn! {mpmc, MPMCThreadPool}
#[macro_export]
macro_rules! get_pool_main {
    ($s:ident) => {
        pool_main($s())
    };
}
