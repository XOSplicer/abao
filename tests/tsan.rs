#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]

// TODO: write multi threaded tests

use abao::AbaoVec;
use std::mem::MaybeUninit;
// use std::thread;
// use std::sync::Arc;
// use std::sync::atomic::{AtomicUsize, Ordering};
// use owning_ref::OwningRef;
// use lazy_static::lazy_static;
use scoped_threadpool::Pool;

#[test]
fn scoped_insert() {
    let threads: usize = 8;
    let mut pool = Pool::new(threads as u32);
    let mut buf: [MaybeUninit<usize>; 512] = unsafe {
        MaybeUninit::uninit().assume_init()
    };
    let buf_len = buf.len();
    let v = &AbaoVec::new(&mut buf[..]);

    let values = (0..buf_len)
        .collect::<Vec<usize>>();
    let chunks = values
        .as_slice()
        .chunks(threads)
        .map(Vec::from);

    pool.scoped(|scoped| {
        for chunk in chunks {
            scoped.execute(move || {
                for i in chunk {
                    v.push(i).unwrap();
                }
            });
        }
    });

    for i in values {
        // assert all (unique) elements are inluded
        assert!(v.as_slice().contains(&i))
    }

}
