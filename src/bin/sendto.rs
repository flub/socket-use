use std::iter;
use std::net::{SocketAddr, UdpSocket};

use anyhow::Result;
use bytes::Bytes;

const MSG_SIZE: usize = 1200;
const MSG_COUNT: usize = 1_000_000;

// Executed in   18.19 secs    fish           external
//    usr time    2.00 secs  550.00 micros    2.00 secs
//    sys time   16.20 secs  166.00 micros   16.20 secs
//
// % time     seconds  usecs/call     calls    errors syscall
// ------ ----------- ----------- --------- --------- ----------------
// 100.00   47.676005           4  10000000           sendto
//   0.00    0.000026          13         2           socket
//   0.00    0.000020          10         2           bind
//   0.00    0.000006           6         1           getsockname
// ------ ----------- ----------- --------- --------- ----------------
// 100.00   47.676057           4  10000005           total
fn main() -> Result<()> {
    let dst_sock = UdpSocket::bind("[::1]:0")?;

    sender(dst_sock.local_addr()?)?;

    Ok(())
}

fn sender(dst: SocketAddr) -> Result<()> {
    let payload: Vec<u8> = iter::repeat(1u8).take(MSG_SIZE).collect();
    let payload = Bytes::from(payload);
    let payloads = iter::repeat(payload).take(MSG_COUNT);

    let sock = UdpSocket::bind("[::1]:0")?;

    for payload in payloads {
        let n = sock.send_to(&payload, dst)?;
        assert_eq!(n, MSG_SIZE);
    }
    println!("send done");

    Ok(())
}
