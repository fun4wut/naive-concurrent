use crate::{ADDR, TIMEOUT};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub fn block_main() -> std::io::Result<()> {
    let listener = TcpListener::bind(ADDR)?;
    for stream in listener.incoming() {
        handle_connection(stream.unwrap())
    }
    Ok(())
}
// 阻塞版本的处理连接
pub fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();
    // 写入HTML报文
    let contents = std::fs::read_to_string("index.html").unwrap();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
        contents
    );
    // 模拟真实环境，加入10ms的延时
    std::thread::sleep(Duration::from_millis(TIMEOUT));
    //    println!("{:?} handling...", std::thread::current().id());
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
