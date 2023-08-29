use std::io::IoSlice;
use std::iter;
use std::net::{Ipv6Addr, SocketAddr, UdpSocket};

use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use socket2::{Domain, Protocol, SockAddr, Type};

const MSG_SIZE: usize = 1200;
const MSG_COUNT: usize = 10_000_000;

fn main() -> Result<()> {
    let dst_sock = UdpSocket::bind("[::1]:0")?;

    sender(dst_sock.local_addr()?)?;

    Ok(())
}

fn sender(dst: SocketAddr) -> Result<()> {
    // 8 bytes header, 2 596 frames => 1200 bytes payload
    let frame: Vec<u8> = iter::repeat(1u8).take((MSG_SIZE - 8) / 2).collect();
    let frame = Bytes::from(frame);
    debug_assert_eq!(frame.len(), 596);
    let mut frames = iter::repeat(frame).take(MSG_COUNT * 2);

    let sock = socket2::Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    let addr = SocketAddr::from((Ipv6Addr::LOCALHOST, 0));
    let addr = SockAddr::from(addr);
    sock.bind(&addr)?;
    println!("SO_SNDBUF: {}", sock.send_buffer_size()?);
    let dst = SockAddr::from(dst);

    let hdr = b"abcdabcd";

    while let Some(frame0) = frames.next() {
        let frame1 = frames.next().expect("odd number of frames");
        let mut buf = BytesMut::with_capacity(MSG_SIZE);
        buf.put(hdr.as_slice());
        buf.put(frame0);
        buf.put(frame1);
        debug_assert_eq!(buf.len(), MSG_SIZE);

        let buf = IoSlice::new(&buf);
        let n = sock.send_to_vectored([buf].as_slice(), &dst)?;
        assert_eq!(n, MSG_SIZE);
    }
    println!("send done");

    Ok(())
}
