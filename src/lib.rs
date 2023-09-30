use std::iter;

use bytes::Bytes;

pub const MSG_SIZE: usize = 1200;
pub const MSG_COUNT: usize = 10_000_000;

pub fn payloads() -> Vec<Bytes> {
    let payload: Vec<u8> = iter::repeat(1u8).take(MSG_SIZE).collect();
    let payload = Bytes::from(payload);
    iter::repeat_with(|| payload.clone())
        .take(MSG_COUNT)
        .collect()
}
