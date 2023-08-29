use std::io::{self, IoSlice};
use std::net::{Ipv6Addr, SocketAddr, UdpSocket};
use std::os::fd::AsRawFd;
use std::{iter, mem};

use anyhow::Result;
use bytes::Bytes;
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
    let mut msg: libc::msghdr = unsafe { mem::zeroed() };

    while let Some(frame0) = frames.next() {
        let frame1 = frames.next().expect("odd number of frames");

        let hdr_buf = IoSlice::new(hdr);
        let frame0_buf = IoSlice::new(&frame0);
        let frame1_buf = IoSlice::new(&frame1);
        let bufs = [hdr_buf, frame0_buf, frame1_buf];

        // Casting these pointers to mut is fine as we only send.  The types are mut only
        // for recvmsg.
        msg.msg_name = dst.as_ptr() as *mut _;
        msg.msg_namelen = dst.len();
        msg.msg_iov = bufs.as_ptr() as *mut _;
        msg.msg_iovlen = bufs.len();

        let n = unsafe { libc::sendmsg(sock.as_raw_fd(), &msg, 0) };
        if n == -1 {
            return Err(io::Error::last_os_error().into());
        }
        assert_eq!(n as usize, MSG_SIZE);
    }
    println!("send done");

    Ok(())
}
