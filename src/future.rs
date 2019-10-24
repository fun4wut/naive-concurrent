use async_std::net::ToSocketAddrs;
use async_std::{
    fs, io,
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};

async fn async_handle(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).await?;
    println!("received!");
    let contents = fs::read_to_string("index.html").await?;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
        contents
    );
    stream.write(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}
pub async fn async_main(addr: impl ToSocketAddrs) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream: TcpStream = stream?;
        task::spawn(async_handle(stream));
    }
    Ok(())
}
