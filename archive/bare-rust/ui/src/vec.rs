// The reason this can not be a generic is:
// 1. to allocate at compile time we need
//     pub const fn new() -> Self
// 2. but a trait can not have a const function

use crate::msg::Msg;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct VecByte<const N: usize> {
    pub data: [u8; N],
    pub len: usize,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct VecMsg<const N: usize> {
    pub data: [Msg; N],
    pub len: usize,
}

use core::ops::Index;

impl<const N: usize> Index<usize> for VecByte<N> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize> VecByte<N> {
    pub const fn new() -> Self {
        VecByte::<N> {
            data: [0; N],
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, value: u8) {
        if self.len < self.data.len() {
            self.data[self.len] = value;
            self.len += 1;
        } else {
            panic!("push: full vector");
        }
    }

    pub fn pop(&mut self) -> u8 {
        if self.len > 0 {
            self.len -= 1;
            self.data[self.len]
        } else {
            panic!("pop: empty vector");
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> VecMsg<N> {
    pub const fn new() -> Self {
        VecMsg::<N> {
            data: [Msg::None; N],
            len: 0,
        }
    }

    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, value: Msg) {
        if self.len < self.data.len() {
            self.data[self.len] = value;
            self.len += 1;
        } else {
            panic!("push: full vector");
        }
    }

    pub fn pop(&mut self) -> Msg {
        if self.len > 0 {
            self.len -= 1;
            self.data[self.len]
        } else {
            panic!("pop: empty vector");
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}
