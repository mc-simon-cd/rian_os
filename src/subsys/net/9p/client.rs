// -----------------------------------------------------------------------------
// Copyright 2026 simon_projec
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -----------------------------------------------------------------------------
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;


use core::convert::TryInto;
use crate::libkern::dmesg::kernel_log;

// Zero-Panic System Error Handling Model for 9P RPC Operations
#[derive(Debug)]
pub enum NinePError {
    BufferTooSmall,
    InvalidMessageCode(u8),
    MalformedString,
    ProtocolViolation(&'static str),
    NetworkSimulationFault,
}

#[derive(Debug, Clone)]
pub struct Qid {
    pub vtype: u8,
    pub version: u32,
    pub path: u64,
}

// Memory-Safe Native Rust Enums depicting network transactions explicitly
#[derive(Debug)]
pub enum NinePMessage {
    Tversion { tag: u16, msize: u32, version: String },
    Rversion { tag: u16, msize: u32, version: String },
    Tattach { tag: u16, fid: u32, afid: u32, uname: String, aname: String },
    Rattach { tag: u16, qid: Qid },
    Tread { tag: u16, fid: u32, offset: u64, count: u32 },
    Rread { tag: u16, data: Vec<u8> },
    Twrite { tag: u16, fid: u32, offset: u64, data: Vec<u8> },
    Rwrite { tag: u16, count: u32 },
}

pub struct NinePCodec;

impl NinePCodec {
    // -----------------------------------------------------------------------------
    // Codec Decoding Block (Bounds-Checked, Unconditional Error Recovery)
    // -----------------------------------------------------------------------------
    pub fn read_u8(buf: &[u8], offset: &mut usize) -> Result<u8, NinePError> {
        if *offset + 1 > buf.len() { return Err(NinePError::BufferTooSmall); }
        let val = buf[*offset];
        *offset += 1;
        Ok(val)
    }

    pub fn read_u16(buf: &[u8], offset: &mut usize) -> Result<u16, NinePError> {
        if *offset + 2 > buf.len() { return Err(NinePError::BufferTooSmall); }
        let val = u16::from_le_bytes(buf[*offset..*offset+2].try_into().unwrap());
        *offset += 2;
        Ok(val)
    }

    pub fn read_u32(buf: &[u8], offset: &mut usize) -> Result<u32, NinePError> {
        if *offset + 4 > buf.len() { return Err(NinePError::BufferTooSmall); }
        let val = u32::from_le_bytes(buf[*offset..*offset+4].try_into().unwrap());
        *offset += 4;
        Ok(val)
    }

    pub fn read_string(buf: &[u8], offset: &mut usize) -> Result<String, NinePError> {
        let len = Self::read_u16(buf, offset)? as usize;
        if *offset + len > buf.len() { return Err(NinePError::BufferTooSmall); }
        
        let string_bytes = &buf[*offset..*offset+len];
        match core::str::from_utf8(string_bytes) {
            Ok(s) => {
                *offset += len;
                Ok(s.to_string())
            }
            Err(_) => Err(NinePError::MalformedString),
        }
    }

    // Main Decoder Hook
    pub fn deserialize(buf: &[u8]) -> Result<NinePMessage, NinePError> {
        let mut offset = 0;
        let _size = Self::read_u32(buf, &mut offset)?; 
        let mtype = Self::read_u8(buf, &mut offset)?;
        
        match mtype {
            101 /* Rversion */ => {
                let tag = Self::read_u16(buf, &mut offset)?;
                let msize = Self::read_u32(buf, &mut offset)?;
                let version = Self::read_string(buf, &mut offset)?;
                Ok(NinePMessage::Rversion { tag, msize, version })
            }
            117 /* Rread */ => {
                let tag = Self::read_u16(buf, &mut offset)?;
                let count = Self::read_u32(buf, &mut offset)? as usize;
                if offset + count > buf.len() { return Err(NinePError::BufferTooSmall); }
                let data = buf[offset..offset+count].to_vec();
                Ok(NinePMessage::Rread { tag, data })
            }
            119 /* Rwrite */ => {
                let tag = Self::read_u16(buf, &mut offset)?;
                let count = Self::read_u32(buf, &mut offset)?;
                Ok(NinePMessage::Rwrite { tag, count })
            }
            _ => {
                kernel_log("9P_CODEC", &format!("Unknown inbound message code: {}", mtype));
                Err(NinePError::InvalidMessageCode(mtype))
            }
        }
    }

    // -----------------------------------------------------------------------------
    // Codec Encoding Block bounds-checked buffer limits natively mapped without serde
    // -----------------------------------------------------------------------------
    pub fn serialize(msg: &NinePMessage, buf: &mut [u8]) -> Result<usize, NinePError> {
        let mut offset = 4; // Reserve space for 4-byte size header at the start

        match msg {
            NinePMessage::Tversion { tag, msize, version } => {
                Self::write_u8(buf, &mut offset, 100)?; // Tversion type code
                Self::write_u16(buf, &mut offset, *tag)?;
                Self::write_u32(buf, &mut offset, *msize)?;
                Self::write_string(buf, &mut offset, version)?;
            }
            NinePMessage::Tread { tag, fid, offset: seek_off, count } => {
                Self::write_u8(buf, &mut offset, 116)?;
                Self::write_u16(buf, &mut offset, *tag)?;
                Self::write_u32(buf, &mut offset, *fid)?;
                Self::write_u64(buf, &mut offset, *seek_off)?;
                Self::write_u32(buf, &mut offset, *count)?;
            }
            _ => return Err(NinePError::ProtocolViolation("Serializer implementation pending for message type")),
        }

        // Back-patch the actual calculated Total-Size struct length to offset 0
        let total_size = offset as u32;
        let mut z = 0;
        Self::write_u32(buf, &mut z, total_size)?;

        Ok(offset)
    }

    fn write_u8(buf: &mut [u8], offset: &mut usize, val: u8) -> Result<(), NinePError> {
        if *offset + 1 > buf.len() { return Err(NinePError::BufferTooSmall); }
        buf[*offset] = val;
        *offset += 1;
        Ok(())
    }

    fn write_u16(buf: &mut [u8], offset: &mut usize, val: u16) -> Result<(), NinePError> {
        if *offset + 2 > buf.len() { return Err(NinePError::BufferTooSmall); }
        buf[*offset..*offset+2].copy_from_slice(&val.to_le_bytes());
        *offset += 2;
        Ok(())
    }

    fn write_u32(buf: &mut [u8], offset: &mut usize, val: u32) -> Result<(), NinePError> {
        if *offset + 4 > buf.len() { return Err(NinePError::BufferTooSmall); }
        buf[*offset..*offset+4].copy_from_slice(&val.to_le_bytes());
        *offset += 4;
        Ok(())
    }

    fn write_u64(buf: &mut [u8], offset: &mut usize, val: u64) -> Result<(), NinePError> {
        if *offset + 8 > buf.len() { return Err(NinePError::BufferTooSmall); }
        buf[*offset..*offset+8].copy_from_slice(&val.to_le_bytes());
        *offset += 8;
        Ok(())
    }

    fn write_string(buf: &mut [u8], offset: &mut usize, s: &str) -> Result<(), NinePError> {
        let bytes = s.as_bytes();
        let len = bytes.len() as u16;
        Self::write_u16(buf, offset, len)?;
        
        if *offset + (len as usize) > buf.len() { return Err(NinePError::BufferTooSmall); }
        buf[*offset..*offset+(len as usize)].copy_from_slice(bytes);
        *offset += len as usize;
        Ok(())
    }
}
