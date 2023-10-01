use std::io::IoSlice;
use std::net::{Ipv6Addr, SocketAddr, UdpSocket};
use std::os::fd::AsRawFd;
use std::thread;
use std::{mem, panic};

use anyhow::{Context, Result};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use sockets_use::{MSG_COUNT, MSG_SIZE};

const BATCH_SIZE: usize = 64;

fn main() -> Result<()> {
    // let dst_sock = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    // let bind_addr = SocketAddr::from((Ipv6Addr::LOCALHOST, 0));
    // let bind_addr = SockAddr::from(bind_addr);
    // dst_sock.bind(&bind_addr)?;
    // let dst_addr = dst_sock.local_addr()?;
    let dst_sock = UdpSocket::bind("[::1]:0")?;
    let dst_addr = dst_sock.local_addr()?.into();

    // let handle = thread::Builder::new()
    //     .name("receiver".into())
    //     .spawn(|| receiver(dst_sock))?;

    sender(dst_addr)?;

    // match handle.join() {
    //     Ok(res) => res.context("receiver error")?,
    //     Err(e) => panic::resume_unwind(e),
    // }

    Ok(())
}

fn receiver(sock: UdpSocket) -> Result<()> {
    const BUF_SIZE: usize = 1500;
    let mut datagrams_received = 0;
    let mut buf = [0u8; BUF_SIZE];
    while datagrams_received < MSG_COUNT {
        let (n, _addr) = sock.recv_from(&mut buf)?;
        datagrams_received += 1;
        assert_eq!(n, MSG_SIZE);
    }
    println!("receive done");
    Ok(())
}

// fn receiver(sock: Socket) -> Result<()> {
//     const BUF_SIZE: usize = 1500;
//     let mut bufs: Vec<Box<[u8]>> = iter::repeat_with(|| -> Box<[u8]> { Box::new([0u8; BUF_SIZE]) })
//         .take(BATCH_SIZE)
//         .collect();
//     let mut iovec: Vec<IoSliceMut> = bufs
//         .iter_mut()
//         .map(|buf| IoSliceMut::new(&mut buf[..]))
//         .collect();
//     let mut msgvec: [libc::mmsghdr; BATCH_SIZE] = unsafe { mem::zeroed() };
//     for (mmsghdr, ioslice) in msgvec.iter_mut().zip(iovec.iter_mut()) {
//         mmsghdr.msg_hdr.msg_iov = ioslice.as_mut_ptr() as *mut libc::iovec;
//         mmsghdr.msg_hdr.msg_iovlen = 1;
//     }
//     let mut timeout = libc::timespec {
//         tv_sec: 3,
//         tv_nsec: 0,
//     };
//     let mut datagrams_received = 0usize;
//     loop {
//         let ret = unsafe {
//             libc::recvmmsg(
//                 sock.as_raw_fd(),
//                 msgvec.as_mut_ptr(),
//                 msgvec.len().try_into().expect("vlen overflow"),
//                 0,
//                 &mut timeout,
//             )
//         };
//         if ret == -1 {
//             return Err(io::Error::last_os_error().into());
//         }
//         datagrams_received += ret as usize;
//         println!("received {datagrams_received} datagrams");
//         for mmsghdr in msgvec.iter().take(ret as usize) {
//             println!("tick {}", mmsghdr.msg_len);
//             // assert_eq!(
//             //     mmsghdr.msg_len,
//             //     MSG_SIZE
//             //         .try_into()
//             //         .expect("MSG_SIZE exceeds mmsghdr.msg_len")
//             // );
//         }
//         if datagrams_received >= MSG_COUNT {
//             break;
//         }
//     }
//     println!("receive done");
//     Ok(())
// }

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
        assert_eq!(ret, batch.len().try_into()?); // Number of messages sent
        for mmsg in mmsgs {
            assert_eq!(mmsg.msg_len as usize, MSG_SIZE); // Number of bytes sent.
        }
    }
    println!("send done");
    Ok(())
}
