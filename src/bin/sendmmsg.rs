use std::io::{self, IoSlice};
use std::net::{Ipv6Addr, SocketAddr, UdpSocket};
use std::os::fd::AsRawFd;
use std::{iter, mem};

use anyhow::Result;
use bytes::Bytes;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

const MSG_SIZE: usize = 1200;
const MSG_COUNT: usize = 10_000_000;
const BATCH_SIZE: usize = 64;

fn main() -> Result<()> {
    let dst_sock = UdpSocket::bind("[::1]:0")?;

    sender(dst_sock.local_addr()?)?;

    Ok(())
}

fn sender(dst: SocketAddr) -> Result<()> {
    let payload: Vec<u8> = iter::repeat(1u8).take(MSG_SIZE).collect();
    let payload = Bytes::from(payload);
    let mut payloads = iter::repeat(payload).take(MSG_COUNT);

    let sock = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    let addr = SocketAddr::from((Ipv6Addr::LOCALHOST, 0));
    let addr = SockAddr::from(addr);
    sock.bind(&addr)?;
    let dst = SockAddr::from(dst);

    let mut mmsgs: [libc::mmsghdr; BATCH_SIZE] = unsafe { mem::zeroed() };

    let mut i = 0;
    loop {
        if let Some(payload) = payloads.next() {
            let buf = IoSlice::new(&payload);
            let bufs = [buf];

            let msg = &mut mmsgs[i].msg_hdr;
            msg.msg_name = dst.as_ptr() as *mut _;
            msg.msg_namelen = dst.len();
            msg.msg_iov = bufs.as_ptr() as *mut _;
            msg.msg_iovlen = bufs.len();

            i += 1;
            if i < BATCH_SIZE {
                continue;
            }
        }
        if i == 0 {
            break; // No more payloads, batch empty
        }
        let ret = unsafe { libc::sendmmsg(sock.as_raw_fd(), mmsgs.as_mut_ptr(), i.try_into()?, 0) };
        if ret == -1 {
            return Err(io::Error::last_os_error().into());
        }
        assert_eq!(ret, i.try_into()?); // Number of messages sent.
        for mmsg in mmsgs {
            assert_eq!(mmsg.msg_len as usize, MSG_SIZE); // Number of bytes sent.
        }
        i = 0;
    }
    println!("send done");
    Ok(())
}
