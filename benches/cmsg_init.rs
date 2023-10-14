//! Compare constructing an array vs Vec.

use std::alloc::{self, Layout};
use std::mem::{self};
use std::ptr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn zeroed_init(c: &mut Criterion) {
    let layout = Layout::from_size_align(32, 8).unwrap();
    let buf = unsafe { alloc::alloc(layout) };
    c.bench_function("zeroed", |b| {
        b.iter(|| {
            let cmsg: *mut libc::cmsghdr = buf.cast();
            let cmsg: &mut libc::cmsghdr = unsafe {
                let cmsg_zeroed: libc::cmsghdr = mem::zeroed();
                ptr::copy_nonoverlapping(&cmsg_zeroed, cmsg, 1);
                cmsg.as_mut().unwrap()
            };
            cmsg.cmsg_level = libc::SOL_UDP;
            cmsg.cmsg_type = libc::UDP_SEGMENT;
            cmsg.cmsg_len = unsafe { libc::CMSG_LEN(mem::size_of::<u16>() as _) } as libc::size_t;
            unsafe { ptr::write(libc::CMSG_DATA(cmsg) as *mut u16, 1200u16) };
            let _ = black_box(cmsg);
        })
    });
}

fn ptr_init(c: &mut Criterion) {
    let layout = Layout::from_size_align(32, 8).unwrap();
    let buf = unsafe { alloc::alloc(layout) };
    c.bench_function("ptr", |b| {
        b.iter(|| {
            let cmsg: *mut libc::cmsghdr = buf.cast();
            let cmsg: &mut libc::cmsghdr = unsafe {
                let cmsg_level = ptr::addr_of_mut!((*cmsg).cmsg_level);
                cmsg_level.write(libc::SOL_UDP);
                let cmsg_type = ptr::addr_of_mut!((*cmsg).cmsg_type);
                cmsg_type.write(libc::UDP_SEGMENT);
                let cmsg_len = ptr::addr_of_mut!((*cmsg).cmsg_len);
                cmsg_len.write(libc::CMSG_LEN(mem::size_of::<u16>() as _) as libc::size_t);
                ptr::write(libc::CMSG_DATA(cmsg) as *mut u16, 1200u16);
                cmsg.as_mut().unwrap()
            };
            let _ = black_box(cmsg);
        })
    });
}

criterion_group!(benches, zeroed_init, ptr_init);
criterion_main!(benches);
