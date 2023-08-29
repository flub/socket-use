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
    let batch_size = check_gso()?;
    assert_eq!(batch_size, BATCH_SIZE);
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

/// Checks Generic Segmentation Offload (GSO) using `UDP_SEGMENT` socket option.
///
/// This checks GSO support by trying to enable it on a socket.
///
/// Returns the maximum number of segments to use.
fn check_gso() -> Result<usize> {
    let sock = UdpSocket::bind("[::1]:0")?;

    // As defined in `udp(7)` and in linux/udp.h
    // #define UDP_MAX_SEGMENTS        (1 << 6UL)
    set_socket_option(&sock, libc::SOL_UDP, libc::UDP_SEGMENT, 1500)
        .map(|_| 64)
        .map_err(|e| e.into())
}

fn set_socket_option(
    socket: &impl AsRawFd,
    level: libc::c_int,
    name: libc::c_int,
    value: libc::c_int,
) -> Result<(), io::Error> {
    let rc = unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            level,
            name,
            &value as *const _ as _,
            mem::size_of_val(&value) as _,
        )
    };

    match rc == 0 {
        true => Ok(()),
        false => Err(io::Error::last_os_error()),
    }
}
