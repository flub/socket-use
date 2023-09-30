use std::alloc::Layout;
use std::io::{self, IoSlice};
use std::net::{Ipv6Addr, SocketAddr, UdpSocket};
use std::os::fd::AsRawFd;
use std::{iter, mem, ptr};

use anyhow::{anyhow, Result};
use bytes::Bytes;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

const MSG_SIZE: usize = 1200;
const MSG_COUNT: usize = 10_000_000;

fn main() -> Result<()> {
    let dst_sock = UdpSocket::bind("[::1]:0")?;

    sender(dst_sock.local_addr()?)?;

    Ok(())
}

fn sender(dst: SocketAddr) -> Result<()> {
    let payload: Vec<u8> = iter::repeat(1u8).take(MSG_SIZE).collect();
    let payload = Bytes::from(payload);
    let payloads: Vec<Bytes> = iter::repeat_with(|| payload.clone())
        .take(MSG_COUNT)
        .collect();

    let sock = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    let addr = SocketAddr::from((Ipv6Addr::LOCALHOST, 0));
    let addr = SockAddr::from(addr);
    sock.bind(&addr)?;
    let dst = SockAddr::from(dst);

    let gso_batch_size = check_gso()?;

    let mut msg: libc::msghdr = unsafe { mem::zeroed() };
    let mut iovec: Vec<IoSlice> = Vec::with_capacity(gso_batch_size);

    for batch in payloads.chunks(gso_batch_size) {
        iovec.clear();
        iovec.extend(batch.iter().map(|payload| IoSlice::new(payload)));
        msg.msg_name = dst.as_ptr() as *mut _;
        msg.msg_namelen = dst.len();
        msg.msg_iov = iovec.as_ptr() as *mut _;
        msg.msg_iovlen = 1;

        // The value of the auxiliary data to put in the control message.
        // let value: u16 = gso_batch_size.try_into()?;
        // let value: u16 = batch.len().try_into()?;
        let segment_size: u16 = MSG_SIZE.try_into()?;
        // The number of bytes needed for this control message.
        let space = unsafe { libc::CMSG_SPACE(mem::size_of_val(&segment_size) as _) };
        let layout = Layout::from_size_align(space as usize, mem::align_of::<libc::cmsghdr>())?;
        let buf = unsafe { std::alloc::alloc(layout) };
        msg.msg_control = buf as *mut libc::c_void;
        msg.msg_controllen = layout.size();
        let cmsg: &mut libc::cmsghdr = unsafe {
            libc::CMSG_FIRSTHDR(&msg)
                .as_mut()
                .ok_or(anyhow!("No space for cmsg"))?
        };
        cmsg.cmsg_level = libc::SOL_UDP;
        cmsg.cmsg_type = libc::UDP_SEGMENT;
        cmsg.cmsg_len =
            unsafe { libc::CMSG_LEN(mem::size_of_val(&segment_size) as _) } as libc::size_t;
        unsafe { ptr::write(libc::CMSG_DATA(cmsg) as *mut u16, segment_size) };

        let ret = unsafe { libc::sendmsg(sock.as_raw_fd(), &msg, 0) };
        if ret == -1 {
            return Err(io::Error::last_os_error().into());
        }
        assert_eq!(ret as usize, MSG_SIZE * gso_batch_size);
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
