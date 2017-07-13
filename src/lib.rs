//! Provides a vector that uses an external slice for storage.

#![no_std]

use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};
use core::mem::replace;
use core::cmp;

/// A Vector using a slice for backing storage (passed in at creation time).
///
/// Changes to the vector are visible in the backing storage after the `SliceVec` is dropped.
///
/// A `SliceVec` can be dereferenced to a truncated slice containing all elements in the `SliceVec`.
/// The returned slice is different from the backing slice in that it only contains the first `n`
/// values, where `n` is the current length of the `SliceVec`. The backing slice may contain unused
/// "dummy" elements after the last element.
///
/// This is essentially a less ergonomic but more flexible version of the `arrayvec` crate's
/// `ArrayVec` type: You have to crate the backing storage yourself, but `SliceVec` works with
/// arrays of *any* length (unlike `ArrayVec`, which works with a fixed set of lengths, since Rust
/// doesn't (yet) have integer generics).
#[derive(Debug)]
pub struct SliceVec<'a, T: 'a> {
    storage: &'a mut [T],
    len: usize,
}

impl<'a, T> SliceVec<'a, T> {
    /// Create a new `SliceVec`, using the given slice as backing storage for elements.
    ///
    /// The capacity of the vector equals the length of the slice, you have to make sure that the
    /// slice is large enough for all elements.
    pub fn new(storage: &'a mut [T]) -> Self {
        SliceVec {
            storage: storage,
            len: 0,
        }
    }

    /// Returns the maximum number of elements that can be stored in this vector. This is equal to
    /// the length of the backing storage passed at creation of this `SliceVec`.
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Returns the number of elements stored in this `SliceVec`.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the length of this vector is 0, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Tries to append an element to the end of this vector.
    ///
    /// If the backing storage is already full, returns `Err(elem)`.
    pub fn push(&mut self, elem: T) -> Result<(), T> {
        if self.len < self.capacity() {
            self.storage[self.len] = elem;
            self.len += 1;
            Ok(())
        } else {
            Err(elem)
        }
    }

    /// Removes and returns the last elements stored inside the vector, replacing it with `elem`.
    ///
    /// If the vector is empty, returns `None` and drops `elem`.
    pub fn pop_and_replace(&mut self, elem: T) -> Option<T> {
        // FIXME should this return a `Result<T, T>` instead?
        if self.len > 0 {
            self.len -= 1;
            let elem = replace(&mut self.storage[self.len], elem);
            Some(elem)
        } else {
            None
        }
    }

    /// Shortens the vector to `len` elements.
    ///
    /// Excess elements are not dropped. They are kept in the backing slice.
    pub fn truncate(&mut self, len: usize) {
        self.len = cmp::min(self.len, len);
    }

    /// Extract a slice containing the entire vector.
    ///
    /// The returned slice will be shorter than the backing slice if the vector hasn't yet exceeded
    /// its capacity.
    pub fn as_slice(&self) -> &[T] {
        &self.storage[..self.len]
    }

    /// Extract a mutable slice containing the entire vector.
    ///
    /// The returned slice will be shorter than the backing slice if the vector hasn't yet exceeded
    /// its capacity.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.storage[..self.len]
    }
}

impl<'a, T: 'a + Default> SliceVec<'a, T> {
    /// Removes and returns the last element in this vector.
    ///
    /// Returns `None` if the vector is empty.
    ///
    /// This operation is restricted to element types that implement `Default`, since the element's
    /// spot in the backing storage is replaced by a default value.
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            let elem = replace(&mut self.storage[self.len], T::default());
            Some(elem)
        } else {
            None
        }
    }

    /// Removes and returns the element at `index` and replaces it with the last element.
    ///
    /// Panics if `index` is out of bounds.
    pub fn swap_remove(&mut self, index: usize) -> T {
        let len = self.len();
        self.as_mut_slice().swap(index, len - 1);
        // the unwrap should never fail since we already touched the slice, causing a bounds check
        self.pop().expect("swap_remove failed pop")
    }
}

impl<'a, T> Deref for SliceVec<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T> DerefMut for SliceVec<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

// Forward useful `[T]` impls so `SliceVec` is useful in generic contexts.
// TODO: There are a lot more we can forward. Is this the right way? `Vec<T>` also forwards dozens.

impl<'a, T> AsRef<[T]> for SliceVec<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T> AsMut<[T]> for SliceVec<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<'a, T> Borrow<[T]> for SliceVec<'a, T> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T> BorrowMut<[T]> for SliceVec<'a, T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

#[test]
fn basic() {
    const CAP: usize = 1;
    let mut storage = [0; CAP];

    {
        let mut s = SliceVec::new(&mut storage);
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert_eq!(s.capacity(), CAP);

        assert_eq!(s.push(123), Ok(()));
        assert_eq!(s.len(), 1);
        assert!(!s.is_empty());
        assert_eq!(s.as_slice(), &[123]);
        assert_eq!(s.push(42), Err(42));
        assert!(!s.is_empty());
        assert_eq!(s.as_slice(), &[123]);
        assert_eq!(s.pop(), Some(123));
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
        assert_eq!(s.as_slice(), &[]);
        assert_eq!(&*s, &[]);
    }
}

#[test]
fn swap_remove() {
    let mut storage = [0; 3];

    {
        let mut s = SliceVec::new(&mut storage);
        assert_eq!(s.len(), 0);
        assert_eq!(s.capacity(), 3);

        assert!(s.is_empty());
        assert_eq!(s.push(0), Ok(()));
        assert!(!s.is_empty());
        assert_eq!(s.push(1), Ok(()));
        assert_eq!(s.push(2), Ok(()));
        assert_eq!(s.push(3), Err(3));
        assert_eq!(s.len(), 3);
        assert_eq!(s.swap_remove(0), 0);
        assert!(!s.is_empty());
        assert_eq!(s.len(), 2);
        assert_eq!(s[0], 2);
        assert_eq!(s[1], 1);
        assert_eq!(s.as_slice().len(), 2);
    }
}
