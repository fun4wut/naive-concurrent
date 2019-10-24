#![feature(box_syntax)]
#![allow(clippy::unused_io_amount)]
mod block;
mod future;
mod pool_lib;
mod threading;
use std::io::prelude::*;
use std::net::TcpStream;

use async_std::task;
use block::block_main;
use future::async_main;
use std::env;
use threading::pool_main;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let addr = "127.0.0.1:8080";
    // 利用命令行参数来决定使用哪种模式
    match &*args[1] {
        "async" => task::block_on(async_main(addr)),
        "pool" => pool_main(addr),
        "block" => block_main(addr),
        _ => panic!("invalid argument!"),
    }?;
    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();
    // 写入HTML报文
    let contents = std::fs::read_to_string("index.html").unwrap();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
