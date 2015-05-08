#![feature(core)]

extern crate core;

use std::mem;
use core::slice;
use std::ops;

use core::array::FixedSizeArray;
use std::fmt::{Debug,Formatter};
use std::iter::FromIterator;

// Assumptions made:
//    1. [T; n] has a representation of n consecutive elements of size_of::<T>
//    2. Only instances of FixedSizeArray<T> are of form [T; n]
//
// TODO: drop and destructors

struct ArrayVec<T, A: FixedSizeArray<T>> {
    array: A,
    idx: usize, // FIXME: isize?
    phantom: core::marker::PhantomData<T>
}

impl<T, A: FixedSizeArray<T>> ArrayVec<T, A> {
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
                let ptr: *mut T = mem::transmute(&mut self.array);
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
                let ptr: *mut T = mem::transmute(&mut self.array);
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
            let ptr: *const T = mem::transmute(&self.array);
            slice::from_raw_parts(ptr, self.length())
        }
    }
}

impl<T, A: FixedSizeArray<T>> ops::DerefMut for ArrayVec<T, A> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            let ptr: *mut T = mem::transmute(&self.array);
            slice::from_raw_parts_mut(ptr, self.length())
        }
    }
}

impl<T: Debug, A: FixedSizeArray<T>> Debug for ArrayVec<T, A> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let as_slice: &[T] = &**self;
        Debug::fmt(as_slice, f)
    }
}

#[test]
fn push_pop() {
    let mut a: ArrayVec<i32, [_; 10]> = ArrayVec::new();
    a.push(5).unwrap();
    assert_eq!(a.pop(), Some(5));
}
