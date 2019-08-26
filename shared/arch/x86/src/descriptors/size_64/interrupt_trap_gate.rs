//**************************************************************************************************
// interrupt_trap_gate.rs                                                                          *
// Copyright (c) 2019 Todd Berta-Oldham                                                            *
// This code is made available under the MIT License.                                              *
//**************************************************************************************************

use crate::selector::Selector as SegmentSelector;
use crate::ProtectionRing;
use core::convert::TryFrom;
use core::result::Result::Err;
use encapsulation::BitGetterSetters;

#[derive(Copy, Clone, PartialEq, Eq, BitGetterSetters, Default)]
#[repr(C, packed)]
pub struct Descriptor {
    lower: u32,
    #[bit_access(name = "is_present", index = 15, set = true, borrow_self = false)]
    middle: u32,
    upper: u32,
    reserved: u32,
}

impl Descriptor {
    pub const fn new() -> Self {
        Descriptor {
            lower: 0,
            middle: 0,
            upper: 0,
            reserved: 0,
        }
    }

    pub fn set_offset(&mut self, offset: u64) {
        self.lower = (self.lower & !0xFFFF) | ((offset & 0xFFFF) as u32);
        self.middle = (self.middle & 0xFFFF) | ((offset & !0xFFFF) as u32);
        self.upper = (offset >> 32) as u32;
    }

    pub fn offset(self) -> u64 {
        ((self.lower as u64) & 0xFFFF)
            | ((self.middle as u64) & !0xFFFF)
            | ((self.upper as u64) << 32)
    }

    pub fn set_segment_selector(&mut self, selector: SegmentSelector) {
        self.lower = (self.lower & 0xFFFF) | ((u16::from(selector) as u32) << 16);
    }

    pub fn segment_selector(self) -> SegmentSelector {
        SegmentSelector::from((self.lower >> 16) as u16)
    }

    pub fn privilege_level(self) -> ProtectionRing {
        ProtectionRing::try_from(((self.middle & 0x6000) >> 13) as u8).unwrap()
    }

    pub fn set_privilege_level(&mut self, privilege: ProtectionRing) {
        self.middle = (self.middle & !0x6000) | ((privilege as u32) << 13);
    }

    pub fn ist(self) -> IstIndex {
        IstIndex::try_from((self.middle & 0x7) as u8).unwrap()
    }

    pub fn set_ist(&mut self, ist: IstIndex) {
        self.middle = (self.middle & !0x7) | (ist as u32);
    }

    pub fn descriptor_type(self) -> DescriptorType {
        DescriptorType::try_from(self.middle & 0xF00).unwrap()
    }

    pub fn set_descriptor_type(&mut self, descriptor_type: DescriptorType) {
        self.middle = (self.middle & !0xF00) | (descriptor_type as u32);
    }
}

impl From<u128> for Descriptor {
    fn from(value: u128) -> Self {
        Descriptor {
            lower: value as u32,
            middle: (value >> 32) as u32,
            upper: (value >> 64) as u32,
            reserved: (value >> 96) as u32,
        }
    }
}

impl From<Descriptor> for u128 {
    fn from(value: Descriptor) -> Self {
        (value.lower as u128)
            | ((value.middle as u128) << 32)
            | ((value.upper as u128) << 64)
            | ((value.reserved as u128) << 96)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DescriptorType {
    Interrupt = 0xE00,
    Trap = 0xF00,
}

impl TryFrom<u32> for DescriptorType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0xE00 => Ok(DescriptorType::Interrupt),
            0xF00 => Ok(DescriptorType::Trap),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum IstIndex {
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5,
    Seven = 6,
}

impl TryFrom<u8> for IstIndex {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(IstIndex::One),
            1 => Ok(IstIndex::Two),
            2 => Ok(IstIndex::Three),
            3 => Ok(IstIndex::Four),
            4 => Ok(IstIndex::Five),
            5 => Ok(IstIndex::Six),
            6 => Ok(IstIndex::Seven),
            _ => Err(()),
        }
    }
}