use std::iter;
// use std::net::{Ipv6Addr, SocketAddr};

// use anyhow::Result;
use bytes::Bytes;
// use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub const MSG_SIZE: usize = 1200;
pub const MSG_COUNT: usize = 10_000_000;
// pub const MSG_COUNT: usize = 100;

pub fn payloads() -> Vec<Bytes> {
    let payload: Vec<u8> = iter::repeat(1u8).take(MSG_SIZE).collect();
    let payload = Bytes::from(payload);
    iter::repeat_with(|| payload.clone())
        .take(MSG_COUNT)
        .collect()
}

// pub fn bound_sock() -> Result<Socket> {
//     let sock = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
//     let addr = SocketAddr::from((Ipv6Addr::LOCALHOST, 0));
//     let addr = SockAddr::from(addr);
//     sock.bind(&addr);
//     Ok(sock)
// }
