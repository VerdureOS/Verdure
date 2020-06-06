//**************************************************************************************************
// directory_ptr.rs                                                                                *
// Copyright (c) 2019-2020 Aurora Berta-Oldham                                                     *
// This code is made available under the MIT License.                                              *
//**************************************************************************************************

use super::directory::DirectoryTable;
use crate::PhysicalAddress52;
use bits::{ReadBit, WriteBitAssign};
use core::ops::{Index, IndexMut};
use core::slice::{Iter, IterMut};

#[repr(align(4096))]
pub struct DirectoryPtrTable([DirectoryPtrEntry; 512]);

impl DirectoryPtrTable {
    pub fn iter(&self) -> Iter<'_, DirectoryPtrEntry> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, DirectoryPtrEntry> {
        self.0.iter_mut()
    }
}

impl Index<usize> for DirectoryPtrTable {
    type Output = DirectoryPtrEntry;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for DirectoryPtrTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DirectoryPtrValue {
    None,
    DirectoryTable(PhysicalAddress52),
    Page1Gb(PhysicalAddress52),
}

impl DirectoryPtrValue {
    pub fn directory_table(self) -> Option<PhysicalAddress52> {
        match self {
            DirectoryPtrValue::DirectoryTable(address) => Some(address),
            _ => None,
        }
    }
    pub fn directory_table_ptr(self) -> Option<*mut DirectoryTable> {
        match self {
            DirectoryPtrValue::DirectoryTable(address) => Some(address.as_mut_ptr()),
            _ => None,
        }
    }
    pub fn page_1gb_ptr(&self) -> Option<*mut u8> {
        unimplemented!()
    }
}

level_4_paging_entry!(pub struct DirectoryPtrEntry);

impl DirectoryPtrEntry {
    pub fn value(self) -> DirectoryPtrValue {
        if self.0.read_bit(0).unwrap() {
            if self.0.read_bit(7).unwrap() {
            } else {
            }
            unimplemented!()
        } else {
            DirectoryPtrValue::None
        }
    }

    pub fn set_value(&mut self, value: DirectoryPtrValue) {
        match value {
            DirectoryPtrValue::None => {
                self.0.write_bit_assign(0, false).unwrap();
                self.0.write_bit_assign(7, false).unwrap();
            }
            DirectoryPtrValue::DirectoryTable(pointer) => {
                self.0.write_bit_assign(0, true).unwrap();
                self.0.write_bit_assign(7, false).unwrap();
            }
            DirectoryPtrValue::Page1Gb(pointer) => {
                self.0.write_bit_assign(0, true).unwrap();
                self.0.write_bit_assign(7, true).unwrap();
            }
        }
    }
}