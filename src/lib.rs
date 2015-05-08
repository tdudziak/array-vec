#![feature(core)]

extern crate core;

use std::mem;
use core::slice;
use std::ops;

use core::array::FixedSizeArray;
use std::fmt::{Debug,Formatter};
use std::iter::FromIterator;

pub struct ArrayVec<T, A: FixedSizeArray<T>> {
    array: Option<A>,
    idx: usize, // FIXME: isize?
    phantom: core::marker::PhantomData<T>
}

impl<T, A: FixedSizeArray<T>> ArrayVec<T, A> {
    unsafe fn base_ptr_mut(&mut self) -> *mut T {
        if let &mut Some(ref mut ref_arr) = &mut self.array {
            return mem::transmute(ref_arr as *mut A)
        }
        unreachable!();
    }

    unsafe fn base_ptr(&self) -> *const T {
        if let &Some(ref ref_arr) = &self.array {
            return mem::transmute(ref_arr as *const A)
        }
        unreachable!();
    }

    pub fn new() -> Self {
        ArrayVec {
            array: Some(unsafe { mem::uninitialized() }),
            idx: 0,
            phantom: core::marker::PhantomData
        }
    }

    pub fn capacity(&self) -> usize {
        mem::size_of::<A>() / mem::size_of::<T>()
    }

    pub fn length(&self) -> usize { self.idx }

    pub fn push(&mut self, x: T) -> Result<(), &'static str> {
        if self.idx < self.capacity() {
            unsafe {
                let ptr = self.base_ptr_mut();
                let mut cell = x;
                mem::swap(&mut *ptr.offset(self.idx as isize), &mut cell);
                mem::forget(cell);
                self.idx = self.idx + 1;
            }
            Ok(())
        } else {
            Err("cannot push: this ArrayVec is full")
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.idx <= 0 {
            None
        } else {
            unsafe {
                let ptr = self.base_ptr_mut();
                let mut cell = mem::uninitialized();
                mem::swap(&mut *ptr.offset(self.idx as isize - 1), &mut cell);
                self.idx = self.idx - 1;
                Some(cell)
            }
        }
    }
}

impl<T, A: FixedSizeArray<T>> ops::Drop for ArrayVec<T, A> {
    fn drop(&mut self) {
        while self.length() > 0 {
            self.pop();
            // The popped element goes out of scope here and its destructor is
            // run (if present).
        }

        // The array now contains garbage and we have to prevent its destructor
        // from running but we cannot mem::forget() out of borrowed context. To
        // work around this, self.array is an Option type and we swap it with
        // None.
        let mut to_be_forgotten: Option<A> = None;
        mem::swap(&mut self.array, &mut to_be_forgotten);
        unsafe { mem::forget(to_be_forgotten) };
    }
}

impl<T, A: FixedSizeArray<T>> FromIterator<T> for ArrayVec<T, A> {
    fn from_iter<I: IntoIterator<Item=T>>(iterable: I) -> ArrayVec<T, A> {
        let mut result = ArrayVec::new();
        for element in iterable {
            result.push(element).unwrap();
        }
        result
    }
}

impl<T, A: FixedSizeArray<T>> ops::Index<usize> for ArrayVec<T, A> {
    type Output = T;

    fn index<'a>(&'a self, index: usize) -> &'a T {
        &(**self)[index]
    }
}

impl<T, A: FixedSizeArray<T>> ops::Deref for ArrayVec<T, A> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.base_ptr(), self.length())
        }
    }
}

impl<T, A: FixedSizeArray<T>> ops::DerefMut for ArrayVec<T, A> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.base_ptr_mut(), self.length())
        }
    }
}

impl<T: Debug, A: FixedSizeArray<T>> Debug for ArrayVec<T, A> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let as_slice: &[T] = &**self;
        Debug::fmt(as_slice, f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::ops;
    use std::mem;

    #[test]
    fn push_pop() {
        let mut a: ArrayVec<i32, [_; 10]> = ArrayVec::new();
        assert_eq!(0, a.length());
        assert_eq!(10, a.capacity());
        a.push(5).unwrap();
        assert_eq!(1, a.length());
        assert_eq!(Some(5), a.pop());
        assert_eq!(0, a.length());
    }

    #[test]
    fn failures() {
        let mut a: ArrayVec<i32, [_; 1]> = ArrayVec::new();
        assert_eq!(0, a.length());
        assert_eq!(None, a.pop());
        assert_eq!(0, a.length());
        assert_eq!(Ok(()), a.push(7));
        assert_eq!(1, a.length());
        assert!(a.push(13).is_err());
        assert_eq!(1, a.length());
    }

    #[test]
    fn zero_len() {
        let mut useless: ArrayVec<i32, [_; 0]> = ArrayVec::new();
        assert_eq!(0, useless.length());
        assert_eq!(0, useless.capacity());
    }

    static mut DROPPINGS_DROPPED: bool = false;

    struct Droppings(u32);

    impl Droppings {
        fn new() -> Self { Droppings(0xDEFEC8ED) }
    }

    impl ops::Drop for Droppings {
        fn drop(&mut self) {
            // Check for the magic value from new(). The magic is used to
            // distinguish properly-initialized values from random garbage.
            assert!(self.0 == 0xDEFEC8ED);
            unsafe { DROPPINGS_DROPPED = true };
        }
    }

    #[test]
    fn uninitialized_drop() {
        let mut a: ArrayVec<Droppings, [_; 3]> = ArrayVec::new();
        a.push(Droppings::new()).unwrap();
        a.push(Droppings::new()).unwrap();
        a.pop();

        // check whether dropping the vector executes elements' destructors
        unsafe {
            DROPPINGS_DROPPED = false;
            mem::drop(a);
            assert!(DROPPINGS_DROPPED);
        }
    }
}
