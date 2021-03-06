//**************************************************************************************************
// cr4.rs                                                                                          *
// Copyright (c) 2021 The Verdure Project                                                          *
// This code is made available under the MIT License.                                              *
//**************************************************************************************************

use memory::{GetBit, SetBitAssign};

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Value(u64);

impl Value {
    pub const fn new() -> Self {
        Value(0)
    }

    pub fn la57(self) -> bool {
        self.0.get_bit(12)
    }

    pub fn set_la57(&mut self, value: bool) {
        self.0.set_bit_assign(12, value)
    }

    pub fn tsd(self) -> bool {
        self.0.get_bit(2)
    }

    pub fn set_tsd(&mut self, value: bool) {
        self.0.set_bit_assign(2, value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Value {
        Value(value)
    }
}

impl From<Value> for u64 {
    fn from(value: Value) -> u64 {
        value.0
    }
}

pub fn read() -> Value {
    let value: Value;
    unsafe {
        llvm_asm!("mov %cr4, $0" : "=r"(value) ::: "volatile");
    }
    value
}

pub unsafe fn write(value: Value) {
    llvm_asm!("mov $0, %cr4" :: "r"(value) :: "volatile")
}
