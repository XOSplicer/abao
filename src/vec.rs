use std::cell::Cell;
use std::mem::MaybeUninit;
use std::sync::atomic::{self, AtomicUsize, Ordering};

use crate::errors::OomError;
use crate::utils::{cell_as_slice_of_cells, cell_from_mut};

/// An array backed apend only vector.
///
///
/// # Examples
///
/// ```
/// use abao::AbaoVec;
/// use std::mem::MaybeUninit;
///
/// let mut buf: [MaybeUninit<u8>; 128] = unsafe {
///     MaybeUninit::uninit().assume_init()
/// };
/// let v = AbaoVec::new(&mut buf[..]);
///
/// v.push(0).unwrap();
/// v.push(1).unwrap();
/// v.push(2).unwrap();
///
/// assert_eq!(v.len(), 3);
/// assert_eq!(v.get(0), Some(&0));
/// assert_eq!(v.get(1), Some(&1));
/// assert_eq!(v.get(2), Some(&2));
/// ```
pub struct AbaoVec<'a, T> {
    /// the next index to write to
    next_idx: AtomicUsize,
    /// length of continous initialized elements
    confirmed_len: AtomicUsize,
    /// backing buffer
    buf: &'a [Cell<MaybeUninit<T>>],
}

impl<'a, T> AbaoVec<'a, T> {
    /// Creates a new empty vector with the given buffer as backing memory.
    ///
    /// The buffer is used to store the elements of the vector in.
    /// The buffer is assumed to be uninitialized when creating the vector.
    /// When the vector is dropped, all items contained
    /// in the vector are dropped in place.
    /// After dropping the vector, the buffer has to be
    /// treated as uninitialized again.
    /// Reading it may rusult in undefined behavior.
    ///
    /// # Exmaples
    ///
    /// ```
    /// use abao::AbaoVec;
    /// use std::mem::MaybeUninit;
    ///
    /// let mut buf: [MaybeUninit<u8>; 128] = unsafe {
    ///     MaybeUninit::uninit().assume_init()
    /// };
    /// let v = AbaoVec::new(&mut buf[..]);
    ///
    /// assert_eq!(v.len(), 0);
    /// ```
    pub fn new(buf: &'a mut [MaybeUninit<T>]) -> Self {
        Self {
            next_idx: AtomicUsize::new(0),
            confirmed_len: AtomicUsize::new(0),
            buf: cell_as_slice_of_cells(cell_from_mut(buf)),
        }
    }

    /// Get the current length of the vector.
    ///
    /// Actually the vector may already contain more elements currently,
    /// which have not finished to be inserted.
    /// However this is the guaranteed minimal length of the vector.
    ///
    /// # Exmaples
    ///
    /// ```
    /// use abao::AbaoVec;
    /// use std::mem::MaybeUninit;
    ///
    /// let mut buf: [MaybeUninit<u8>; 128] = unsafe {
    ///     MaybeUninit::uninit().assume_init()
    /// };
    /// let v = AbaoVec::new(&mut buf[..]);
    ///
    /// assert_eq!(v.len(), 0);
    /// v.push(1).unwrap();
    /// assert_eq!(v.len(), 1);
    /// v.push(2).unwrap();
    /// assert_eq!(v.len(), 2);
    /// v.push(3).unwrap();
    /// assert_eq!(v.len(), 3);
    ///
    /// ```
    pub fn len(&self) -> usize {
        let len = self.confirmed_len.load(Ordering::Relaxed);
        debug_assert!(
            len <= self.buf.len(),
            "Invariant violation: Vector longer than buffer"
        );
        debug_assert!(
            len <= self.next_idx.load(Ordering::Relaxed),
            "Invarian violation: Vector has more confirmed writes than total writes"
        );
        len
    }


    /// Get the value at index `idx`.
    ///
    /// Returns `None` if the index is out of bounds of the vector.
    ///
    /// Only compleated `push` operations can increase the readable length
    /// of the vector. Therfore only `get` operations are consistent,
    /// even while `push` operations may be performed conrurrently.
    ///
    /// # Examples
    ///
    /// ```
    /// use abao::AbaoVec;
    /// use std::mem::MaybeUninit;
    ///
    /// let mut buf: [MaybeUninit<u8>; 128] = unsafe {
    ///     MaybeUninit::uninit().assume_init()
    /// };
    /// let v = AbaoVec::new(&mut buf[..]);
    ///
    /// v.push(0).unwrap();
    /// v.push(1).unwrap();
    /// v.push(2).unwrap();
    ///
    /// assert_eq!(v.get(0), Some(&0));
    /// assert_eq!(v.get(1), Some(&1));
    /// assert_eq!(v.get(2), Some(&2));
    /// assert_eq!(v.get(3), None);
    /// assert_eq!(v.get(128), None);
    /// ```
    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx >= self.len() {
            return None;
        }
        unsafe {
            // NOTE(unsafe):
            // since all elements up to at least the current len
            // have been initialized
            // and idx is not out of bounds, this is safe to do
            return Some(self.get_unchecked(idx));
        }
    }

    /// Get the value at index `idx` without checking bounds.
    ///
    /// # Safety
    /// An index that is out of bounds of this vector can cause creating
    /// a reference to uninitialized memory within the underlaying buffer
    /// or even outside of the underlaying buffer.
    /// This is generally undefined behavior.
    pub unsafe fn get_unchecked(&self, idx: usize) -> &T {
        // NOTE(unsafe):
        // only safe when idx is not out of bounds of initialized elements
        let cell_ptr = self.buf.get_unchecked(idx).as_ptr() as *const MaybeUninit<T>;
        &*(*cell_ptr).as_ptr()
    }

    /// TODO: write doc
    ///
    /// # Eaxmples
    /// ```
    /// use abao::AbaoVec;
    /// use abao::OomError;
    /// use std::mem::MaybeUninit;
    ///
    /// let mut buf: [MaybeUninit<u8>; 4] = unsafe {
    ///     MaybeUninit::uninit().assume_init()
    /// };
    /// let v = AbaoVec::new(&mut buf[..]);
    ///
    /// assert_eq!(v.push(0), Ok(0));
    /// assert_eq!(v.push(1), Ok(1));
    /// assert_eq!(v.push(2), Ok(2));
    /// assert_eq!(v.push(3), Ok(3));
    /// assert_eq!(v.push(4), Err(OomError));
    ///
    /// assert_eq!(v.as_slice(), &[0, 1, 2, 3])
    ///
    /// ```
    pub fn push(&self, t: T) -> Result<usize, OomError> {
        // 1. claim the next index to write to by increasing it
        // this ensures that only the current push
        // can access the memory at the claimed location

        let idx = self.next_idx.fetch_add(1, Ordering::SeqCst); // can this be weaker?

        if idx >= self.buf.len() {
            return Err(OomError);
        }

        // 2. write to the claimed index

        unsafe {
            // NOTE(unsafe):
            // TODO: write safty note
            let cell_ptr = self.buf.get_unchecked(idx).as_ptr();
            let ptr: *mut T = (&mut *cell_ptr).as_mut_ptr();
            std::ptr::write(ptr, t);
        }

        // 3. increase the confirmed length to be the next index after this,
        // but only if all previous writes have finished.
        // it may be only increased by one.
        // this ensures that read calls can only access
        // completely initialized memory.

        let expected_current = idx;
        let new_confirmed = idx + 1;

        // NOTE(spinlock):
        // TODO: Write spinlock note
        while self
            .confirmed_len
            .compare_exchange(
                expected_current,
                new_confirmed,
                Ordering::SeqCst,
                Ordering::SeqCst, // can this be weaker?
            )
            .is_err()
        {
            atomic::spin_loop_hint()
        }

        Ok(idx)
    }

    /// Extracts a slice containing the entire vector up to the current length.
    ///
    /// This slice does not include elements that are currently being inserted.
    /// However it contains only fully inserted elements.
    ///
    /// # Examples
    /// ```
    /// use abao::AbaoVec;
    /// use std::mem::MaybeUninit;
    ///
    /// let mut buf: [MaybeUninit<u8>; 128] = unsafe {
    ///     MaybeUninit::uninit().assume_init()
    /// };
    /// let v = AbaoVec::new(&mut buf[..]);
    ///
    /// assert_eq!(v.as_slice(), &[]);
    ///
    /// v.push(0).unwrap();
    /// v.push(1).unwrap();
    /// v.push(2).unwrap();
    ///
    /// assert_eq!(v.as_slice(), &[0, 1, 2]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        // NOTE(unsafe):
        // TODO: write safety note
        // NOTE(index):
        // self.len() should never be out of bound,
        // so checking the index is actually not necessary
        unsafe { &*(&self.buf[0..self.len()] as *const [Cell<MaybeUninit<T>>] as *const [T]) }
    }
}

impl<'a, T> Drop for AbaoVec<'a, T> {
    fn drop(&mut self) {
        for cell in &self.buf[0..self.len()] {
            // NOTE(unsafe):
            unsafe {
                let cell_ptr = cell.as_ptr();
                let ptr: *mut T = (&mut *cell_ptr).as_mut_ptr();
                std::ptr::drop_in_place(ptr);
            }
        }
    }
}

// TODO: add drop test
