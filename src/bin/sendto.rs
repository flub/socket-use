use std::net::{SocketAddr, UdpSocket};

use anyhow::Result;
use sockets_use::MSG_SIZE;

fn main() -> Result<()> {
    let dst_sock = UdpSocket::bind("[::1]:0")?;

    sender(dst_sock.local_addr()?)?;

    Ok(())
}

fn sender(dst: SocketAddr) -> Result<()> {
    let payloads = sockets_use::payloads();

    let sock = UdpSocket::bind("[::1]:0")?;

    for payload in payloads {
        let n = sock.send_to(&payload, dst)?;
        assert_eq!(n, MSG_SIZE);
    }
    println!("send done");

    Ok(())
}
