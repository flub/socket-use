use std::io::{self, IoSlice};
use std::mem;
use std::net::{Ipv6Addr, SocketAddr, UdpSocket};
use std::os::fd::AsRawFd;

use anyhow::Result;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use sockets_use::MSG_SIZE;

const BATCH_SIZE: usize = 64;

fn main() -> Result<()> {
    let dst_sock = UdpSocket::bind("[::1]:0")?;
    let dst_addr = dst_sock.local_addr()?.into();

    sender(dst_addr)?;

    Ok(())
}

fn sender(dst: SockAddr) -> Result<()> {
    let payloads = sockets_use::payloads();

    let sock = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    let addr = SocketAddr::from((Ipv6Addr::LOCALHOST, 0));
    let addr = SockAddr::from(addr);
    sock.bind(&addr)?;

    let mut mmsgs: [libc::mmsghdr; BATCH_SIZE] = unsafe { mem::zeroed() };

    for batch in payloads.chunks(BATCH_SIZE) {
        for (i, payload) in batch.iter().enumerate() {
            let buf = IoSlice::new(payload);
            let bufs = [buf];

            let mmsg = &mut mmsgs[i].msg_hdr;
            mmsg.msg_name = dst.as_ptr() as *mut _;
            mmsg.msg_namelen = dst.len();
            mmsg.msg_iov = bufs.as_ptr() as *mut _;
            mmsg.msg_iovlen = bufs.len();
        }
        let ret = unsafe {
            libc::sendmmsg(
                sock.as_raw_fd(),
                mmsgs.as_mut_ptr(),
                batch.len().try_into()?,
                0,
            )
        };
        if ret == -1 {
            return Err(io::Error::last_os_error().into());
        }
        assert_eq!(ret, batch.len().try_into()?); // Number of messages sent
        for mmsg in mmsgs {
            assert_eq!(mmsg.msg_len as usize, MSG_SIZE); // Number of bytes sent.
        }
    }
    println!("send done");
    Ok(())
}
