//**************************************************************************************************
// pml_5.rs                                                                                        *
// Copyright (c) 2020-2021 The Verdure Project                                                     *
// This code is made available under the MIT License.                                              *
//**************************************************************************************************

use crate::paging::size_64::Pml4Table;
use crate::PhysicalAddress52;
use core::convert::TryFrom;
use core::ops::{Index, IndexMut};
use core::slice::{Iter, IterMut};
use memory::{AlignmentError, CheckAlignment, GetBit, SetBitAssign};

#[repr(align(4096))]
pub struct Pml5Table([Pml5Entry; 512]);

impl Pml5Table {
    pub fn get(&self, index: usize) -> Option<&Pml5Entry> {
        self.0.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Pml5Entry> {
        self.0.get_mut(index)
    }

    pub fn iter(&self) -> Iter<'_, Pml5Entry> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Pml5Entry> {
        self.0.iter_mut()
    }
}

impl Index<usize> for Pml5Table {
    type Output = Pml5Entry;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for Pml5Table {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Pml5Value {
    None,
    Pml4Table(PhysicalAddress52),
}

impl Pml5Value {
    pub fn pml4_table(self) -> Option<PhysicalAddress52> {
        match self {
            Pml5Value::Pml4Table(address) => Some(address),
            _ => None,
        }
    }
    pub fn pml4_table_ptr(self) -> Option<*mut Pml4Table> {
        match self {
            Pml5Value::Pml4Table(address) => Some(address.as_mut_ptr()),
            _ => None,
        }
    }
}

u64_paging_entry!(pub struct Pml5Entry);

impl Pml5Entry {
    pub fn value(self) -> Pml5Value {
        if self.0.get_bit(0) {
            let address = self.0.get_bits(12, 12, 40);
            Pml5Value::Pml4Table(PhysicalAddress52::try_from(address).unwrap())
        } else {
            Pml5Value::None
        }
    }

    pub fn set_value(&mut self, value: Pml5Value) -> Result<(), AlignmentError> {
        match value {
            Pml5Value::None => {
                self.0.set_bit_assign(0, false);
            }
            Pml5Value::Pml4Table(address) => {
                if !address.check_alignment(4096) {
                    return Err(AlignmentError);
                }
                self.0.set_bit_assign(0, true);
                self.0.set_bits_assign(address.into(), 12, 12, 40);
            }
        }
        Ok(())
    }
}
