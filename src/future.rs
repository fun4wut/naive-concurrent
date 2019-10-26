use crate::{ADDR, TIMEOUT};

use async_std::{
    fs, io,
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};
use futures_timer::Delay;
use std::time::Duration;

async fn async_handle(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).await?;
    let contents = fs::read_to_string("index.html").await?;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
        contents
    );
    // 模拟真实环境，加入10ms的延时
    Delay::new(Duration::from_millis(TIMEOUT)).await;
    println!("{:?} handling...", task::current().id());
    stream.write(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}
pub async fn async_main() -> io::Result<()> {
    let listener = TcpListener::bind(ADDR).await?;
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream: TcpStream = stream?;
        task::spawn(async_handle(stream));
    }
    Ok(())
}
