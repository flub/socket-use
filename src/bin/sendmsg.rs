use std::io::IoSlice;
use std::net::{Ipv6Addr, SocketAddr, UdpSocket};

use anyhow::Result;
use socket2::{Domain, Protocol, SockAddr, Type};
use sockets_use::MSG_SIZE;

fn main() -> Result<()> {
    let dst_sock = UdpSocket::bind("[::1]:0")?;

    sender(dst_sock.local_addr()?)?;

    Ok(())
}

fn sender(dst: SocketAddr) -> Result<()> {
    let payloads = sockets_use::payloads();

    let sock = socket2::Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    let addr = SocketAddr::from((Ipv6Addr::LOCALHOST, 0));
    let addr = SockAddr::from(addr);
    sock.bind(&addr)?;
    let dst = SockAddr::from(dst);

    for payload in payloads {
        let buf = IoSlice::new(&payload);
        let n = sock.send_to_vectored([buf].as_slice(), &dst)?;
        assert_eq!(n, MSG_SIZE);
    }
    println!("send done");

    Ok(())
}
