#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]

use abao::AbaoVec;
use scoped_threadpool::Pool;
use std::mem::MaybeUninit;

#[test]
fn scoped_insert() {
    let threads: usize = 8;
    let mut pool = Pool::new(threads as u32);
    let mut buf: [MaybeUninit<usize>; 512] = unsafe { MaybeUninit::uninit().assume_init() };
    let buf_len = buf.len();
    let v = &AbaoVec::new(&mut buf[..]);

    let values = (0..buf_len).collect::<Vec<usize>>();
    let chunks = values.as_slice().chunks(threads).map(Vec::from);

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
