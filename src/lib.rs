#![feature(core)]

extern crate core;

use std::mem;
use core::slice;
use std::ops;

use core::array::FixedSizeArray;
use std::fmt::{Debug,Formatter};
use std::iter::FromIterator;

pub struct ArrayVec<T, A: FixedSizeArray<T>> {
    array: A,
    idx: usize, // FIXME: isize?
    phantom: core::marker::PhantomData<T>
}

impl<T, A: FixedSizeArray<T>> ArrayVec<T, A> {
    unsafe fn base_ptr_mut(&mut self) -> *mut T {
        let ptr_arr: *mut A = &mut self.array;
        mem::transmute(ptr_arr)
    }

    unsafe fn base_ptr(&self) -> *const T {
        let ptr_arr: *const A = &self.array;
        mem::transmute(ptr_arr)
    }

    pub fn new() -> Self {
        ArrayVec {
            array: unsafe { mem::uninitialized() },
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

    struct Droppings(u32);

    impl Droppings {
        fn new() -> Self { Droppings(0xDEFEC8ED) }
    }

    impl ops::Drop for Droppings {
        fn drop(&mut self) {
            assert!(self.0 == 0xDEFEC8ED); // check for magic value from new()
            self.0 = 0xDEADBEEF; // set to another magic value
        }
    }

    // FIXME: re-enable and implement proper dropping
    // #[test]
    fn uninitialized_drop() {
        let mut a: ArrayVec<Droppings, [_; 3]> = ArrayVec::new();
        a.push(Droppings::new());
        a.push(Droppings::new());
        a.pop();

        // check whether the destructor ran
        unsafe {
            let ptr: *const u32 = mem::transmute(a.base_ptr());
            assert_eq!(0xDEFEC8ED, *ptr);
            assert_eq!(0xDEADBEEF, *ptr.offset(1));
            assert!(0xDEFEC8ED != *ptr.offset(2));
            assert!(0xDEADBEEF != *ptr.offset(2));
        }
    }
}
