#![feature(box_syntax)]
#![allow(clippy::unused_io_amount)]
mod block;
mod future;
mod pool_lib;
mod threading;

use async_std::task;
use block::block_main;
use future::async_main;
use std::env;
use threading::*;

const TIMEOUT: u64 = 10;
const POOL_SIZE: usize = 16;
const ADDR: &str = "0.0.0.0:8080";

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // 利用命令行参数来决定使用哪种模式
    match &*args[1] {
        "async" => task::block_on(async_main()),
        "block" => block_main(),
        "naive" => get_pool_main!(naive),
        "schedule" => get_pool_main!(schedule),
        "mpmc" => get_pool_main!(mpmc),
        "cvar" => get_pool_main!(cvar),
        _ => panic!("invalid arguments!"),
    }?;
    Ok(())
}
