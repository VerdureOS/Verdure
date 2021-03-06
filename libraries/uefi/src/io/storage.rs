//**************************************************************************************************
// storage.rs                                                                                      *
// Copyright (c) 2019-2020 Aurora Berta-Oldham                                                     *
// This code is made available under the MIT License.                                              *
//**************************************************************************************************

use crate::error::Error;
use crate::ffi::simple_file_system;
use crate::ffi::Status;
use crate::ffi::{file, loaded_image};
use crate::{protocol, Handle};
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::iter::FusedIterator;
use core::mem;
use core::ptr;
use io::{Read, Write};
use ucs2;

#[derive(Debug)]
pub struct VolumeBuffer(protocol::HandleBuffer);

impl VolumeBuffer {
    pub fn locate() -> Result<Self, Error> {
        let handle_buffer = protocol::HandleBuffer::locate(simple_file_system::Protocol::GUID)?;
        Ok(VolumeBuffer(handle_buffer))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn open(&self, index: usize) -> Result<Volume, Error> {
        let protocol = self.0.open(index)?;
        Ok(Volume(protocol))
    }

    pub fn iter(&self) -> VolumeIterator {
        VolumeIterator(self.0.iter())
    }
}

impl<'a> IntoIterator for &'a VolumeBuffer {
    type Item = Result<Volume, Error>;
    type IntoIter = VolumeIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct VolumeIterator<'a>(protocol::InterfaceIterator<'a>);

impl<'a> Iterator for VolumeIterator<'a> {
    type Item = Result<Volume, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|result| result.map(|interface| Volume(interface)))
    }
}

impl<'a> FusedIterator for VolumeIterator<'a> {}

#[derive(Debug)]
pub struct Volume(protocol::Interface);

impl Volume {
    pub fn new(interface: protocol::Interface) -> Result<Self, Error> {
        if interface.protocol_guid() != simple_file_system::Protocol::GUID {
            return Err(Error::InvalidArgument("interface"));
        }
        Ok(Volume(interface))
    }

    pub unsafe fn new_unchecked(interface: protocol::Interface) -> Self {
        Volume(interface)
    }

    pub fn containing_image(image_handle: Handle) -> Result<Self, Error> {
        unsafe {
            let loaded_image_interface =
                protocol::Interface::open(loaded_image::Protocol::GUID, image_handle)?;
            let loaded_image_protocol = &*loaded_image_interface.get::<loaded_image::Protocol>();
            let volume_interface = protocol::Interface::open(
                simple_file_system::Protocol::GUID,
                loaded_image_protocol.device_handle,
            )?;
            Ok(Self(volume_interface))
        }
    }

    pub fn containing_current_image() -> Result<Self, Error> {
        Self::containing_image(crate::system::handle()?)
    }

    pub fn open_node(&self, path: &str, read: bool, write: bool) -> Result<Node, Error> {
        let root = self.root_node()?;
        root.open_child_node(path, read, write)
    }

    pub fn root_node(&self) -> Result<Node, Error> {
        unsafe {
            let interface = self.0.get::<simple_file_system::Protocol>();
            let sfs = &*interface;
            let mut file_protocol = ptr::null_mut();

            let status = (sfs.open_volume)(interface, &mut file_protocol);

            match status {
                Status::SUCCESS => Node::new(file_protocol),
                Status::UNSUPPORTED => Err(Error::UnsupportedFileSystem),
                Status::NO_MEDIA => Err(Error::NoMedia),
                Status::DEVICE_ERROR => Err(Error::DeviceError),
                Status::VOLUME_CORRUPTED => Err(Error::VolumeCorrupted),
                Status::ACCESS_DENIED => Err(Error::OperationDenied),
                Status::OUT_OF_RESOURCES => Err(Error::OutOfMemory),
                Status::MEDIA_CHANGED => Err(Error::MediaInvalidated),
                _ => Err(Error::UnexpectedStatus(status)),
            }
        }
    }
}

pub struct Node(*mut file::Protocol);

impl Node {
    pub unsafe fn new(file_protocol: *mut file::Protocol) -> Result<Node, Error> {
        Ok(Node(file_protocol))
    }

    pub fn open_child_node(&self, path: &str, read: bool, write: bool) -> Result<Node, Error> {
        let mut open_mode = file::OpenModes::empty();

        if read {
            open_mode |= file::OpenModes::READ;
        }

        if write {
            open_mode |= file::OpenModes::WRITE;
        }

        self.open_child_node_internal(path, open_mode, file::Attributes::empty())
    }

    pub fn create_child_node(&self, path: &str, node_type: NodeType) -> Result<Node, Error> {
        let open_mode = file::OpenModes::WRITE | file::OpenModes::CREATE | file::OpenModes::READ;
        let mut attributes = file::Attributes::empty();

        if node_type.is_directory() {
            attributes |= file::Attributes::DIRECTORY;
        }

        self.open_child_node_internal(path, open_mode, attributes)
    }

    fn open_child_node_internal(
        &self,
        path: &str,
        open_mode: file::OpenModes,
        attributes: file::Attributes,
    ) -> Result<Node, Error> {
        unsafe {
            let mut path_buffer =
                ucs2::encode_string_with_null(path).map_err(|_| Error::InvalidArgument("path"))?;
            let path_pointer = path_buffer.as_mut_ptr();

            let protocol = &*self.0;
            let mut new_protocol = ptr::null_mut();

            let status = (protocol.open)(
                self.0,
                &mut new_protocol,
                path_pointer,
                open_mode,
                attributes,
            );

            match status {
                Status::SUCCESS => Node::new(new_protocol),
                Status::NOT_FOUND => Err(Error::PathNonExistent(String::from(path))),
                Status::NO_MEDIA => Err(Error::NoMedia),
                Status::MEDIA_CHANGED => Err(Error::MediaInvalidated),
                Status::DEVICE_ERROR => Err(Error::DeviceError),
                Status::VOLUME_CORRUPTED => Err(Error::VolumeCorrupted),
                Status::WRITE_PROTECTED => Err(Error::ReadOnlyViolation),
                Status::ACCESS_DENIED => Err(Error::OperationDenied),
                Status::OUT_OF_RESOURCES => Err(Error::OutOfMemory),
                Status::VOLUME_FULL => Err(Error::VolumeFull),
                _ => Err(Error::UnexpectedStatus(status)),
            }
        }
    }

    pub fn read_to_end(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        let info = self.get_info()?;

        if info.node_type() == NodeType::Directory {
            return Err(Error::FileOnlyOperation);
        }

        let position = self.get_position()? as usize;
        let length = buffer.len();
        let size = info.size().unwrap() as usize;
        let additional = (size + length) - (buffer.capacity() + position);

        if additional > 0 {
            buffer.reserve_exact(additional);
        }

        for _ in 0..additional {
            buffer.push(0);
        }

        self.read_internal(&mut buffer[length..])
    }

    fn read_internal(&self, buffer: &mut [u8]) -> Result<(), Error> {
        unsafe {
            let data = buffer.as_ptr() as *mut c_void;
            let mut data_size = buffer.len();

            let protocol = &*self.0;

            let status = (protocol.read)(self.0, &mut data_size, data);

            match status {
                Status::SUCCESS => Ok(()),
                Status::NO_MEDIA => Err(Error::NoMedia),
                Status::DEVICE_ERROR => Err(Error::DeviceError),
                Status::VOLUME_CORRUPTED => Err(Error::VolumeCorrupted),
                _ => Err(Error::UnexpectedStatus(status)),
            }
        }
    }

    pub fn set_position(&self, position: u64) -> Result<(), Error> {
        unsafe {
            let protocol = &*self.0;

            let status = (protocol.set_position)(self.0, position);

            match status {
                Status::SUCCESS => Ok(()),
                Status::UNSUPPORTED => Err(Error::FileOnlyOperation),
                Status::DEVICE_ERROR => Err(Error::DeviceError),
                _ => Err(Error::UnexpectedStatus(status)),
            }
        }
    }

    pub fn get_position(&self) -> Result<u64, Error> {
        unsafe {
            let protocol = &*self.0;
            let mut position = 0;

            let status = (protocol.get_position)(self.0, &mut position);

            match status {
                Status::SUCCESS => Ok(position),
                Status::UNSUPPORTED => Err(Error::FileOnlyOperation),
                Status::DEVICE_ERROR => Err(Error::DeviceError),
                _ => Err(Error::UnexpectedStatus(status)),
            }
        }
    }

    pub fn get_info(&self) -> Result<NodeInfo, Error> {
        unsafe {
            let protocol = &*self.0;
            let mut id = file::Info::ID;
            let mut buffer_size = 0;

            // Get size first. This should give a buffer too small error.

            let mut status =
                (protocol.get_info)(self.0, &mut id, &mut buffer_size, ptr::null_mut());

            match status {
                Status::NO_MEDIA => return Err(Error::NoMedia),
                Status::DEVICE_ERROR => return Err(Error::DeviceError),
                Status::VOLUME_CORRUPTED => return Err(Error::VolumeCorrupted),
                Status::BUFFER_TOO_SMALL => {}
                // SUCCESS and UNSUPPORTED are handled by this.
                _ => return Err(Error::UnexpectedStatus(status)),
            }

            // Get actual info.

            let mut buffer = memory_pool!(buffer_size);

            status = (protocol.get_info)(
                self.0,
                &mut id,
                &mut buffer_size,
                buffer.as_mut_ptr() as *mut c_void,
            );

            let info = &*(buffer.as_mut_ptr() as *mut file::Info);

            match status {
                Status::SUCCESS => {
                    if info.attribute.contains(file::Attributes::DIRECTORY) {
                        Ok(NodeInfo::Directory)
                    } else {
                        Ok(NodeInfo::File(info.file_size))
                    }
                }
                _ => Err(Error::UnexpectedStatus(status)),
            }
        }
    }

    pub fn flush(&self) -> Result<(), Error> {
        unsafe {
            let protocol = &*self.0;

            let status = (protocol.flush)(self.0);

            match status {
                Status::SUCCESS => Ok(()),
                Status::NO_MEDIA => Err(Error::NoMedia),
                Status::DEVICE_ERROR => Err(Error::DeviceError),
                Status::VOLUME_CORRUPTED => Err(Error::VolumeCorrupted),
                Status::WRITE_PROTECTED => Err(Error::ReadOnlyViolation),
                Status::ACCESS_DENIED => Err(Error::NoWriteAccess),
                Status::VOLUME_FULL => Err(Error::VolumeFull),
                _ => Err(Error::UnexpectedStatus(status)),
            }
        }
    }

    pub fn delete(self) -> Result<(), Error> {
        unsafe {
            let protocol = &*self.0;

            let status = (protocol.delete)(self.0);

            mem::forget(self);

            match status {
                Status::SUCCESS => Ok(()),
                Status::WARN_DELETE_FAILURE => Err(Error::DeleteFailed),
                _ => Err(Error::UnexpectedStatus(status)),
            }
        }
    }
}

impl Read for Node {
    type Error = Error;

    fn read_exact(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let info = self.get_info()?;

        if info.node_type() == NodeType::Directory {
            return Err(Error::FileOnlyOperation);
        }

        self.read_internal(buffer)
    }
}

impl Write for Node {
    type Error = Error;

    fn write(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        unsafe {
            let data = buffer.as_ptr() as *mut c_void;
            let mut data_size = buffer.len();
            let protocol = &*self.0;

            let status = (protocol.write)(self.0, &mut data_size, data);

            match status {
                Status::SUCCESS => Ok(()),
                Status::UNSUPPORTED => Err(Error::FileOnlyOperation),
                Status::NO_MEDIA => Err(Error::NoMedia),
                Status::DEVICE_ERROR => Err(Error::DeviceError),
                Status::VOLUME_CORRUPTED => Err(Error::VolumeCorrupted),
                Status::WRITE_PROTECTED => Err(Error::ReadOnlyViolation),
                Status::ACCESS_DENIED => Err(Error::NoWriteAccess),
                Status::VOLUME_FULL => Err(Error::VolumeFull),
                _ => Err(Error::UnexpectedStatus(status)),
            }
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            let protocol = &*self.0;
            (protocol.close)(self.0);
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NodeType {
    File,
    Directory,
}

impl NodeType {
    pub fn is_file(self) -> bool {
        match self {
            NodeType::File => true,
            _ => false,
        }
    }
    pub fn is_directory(self) -> bool {
        match self {
            NodeType::Directory => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum NodeInfo {
    File(u64),
    Directory,
}

impl NodeInfo {
    pub fn node_type(&self) -> NodeType {
        match self {
            NodeInfo::File(_) => NodeType::File,
            NodeInfo::Directory => NodeType::Directory,
        }
    }

    pub fn size(&self) -> Option<u64> {
        match self {
            NodeInfo::File(size) => Some(*size),
            _ => None,
        }
    }
}
