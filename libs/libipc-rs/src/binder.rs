// Copyright 2026 mcsimon
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

use std::convert::TryInto;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use crate::parcel::Parcel;

pub const IPC_SUCCESS: i32 = 0;
pub const IPC_ERROR: i32 = -1;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IpcHeader {
    pub code: i32,
    pub data_size: u32,
}

/// Connects to a UNIX domain socket at the specified path.
pub fn ipc_connect<P: AsRef<Path>>(path: P) -> std::io::Result<UnixStream> {
    UnixStream::connect(path)
}

/// Sends an IPC transaction over the UnixStream.
/// Consists of writing the header, writing the data parcel payload,
/// reading the reply header, and reading the reply parcel payload.
pub fn ipc_send_transaction(
    stream: &mut UnixStream,
    code: i32,
    data: &Parcel,
) -> std::io::Result<Parcel> {
    // 1. Prepare and write transaction header
    let header = IpcHeader {
        code,
        data_size: data.size() as u32,
    };
    
    let mut header_bytes = [0u8; 8];
    header_bytes[0..4].copy_from_slice(&header.code.to_ne_bytes());
    header_bytes[4..8].copy_from_slice(&header.data_size.to_ne_bytes());
    stream.write_all(&header_bytes)?;
    
    // 2. Write parcel payload if size > 0
    if header.data_size > 0 {
        stream.write_all(data.data())?;
    }
    stream.flush()?;
    
    // 3. Read reply header
    let mut reply_header_bytes = [0u8; 8];
    stream.read_exact(&mut reply_header_bytes)?;
    
    let _reply_code = i32::from_ne_bytes(reply_header_bytes[0..4].try_into().unwrap());
    let reply_data_size = u32::from_ne_bytes(reply_header_bytes[4..8].try_into().unwrap());
    
    // 4. Read reply payload if size > 0
    let mut reply_data = vec![0u8; reply_data_size as usize];
    if reply_data_size > 0 {
        stream.read_exact(&mut reply_data)?;
    }
    
    Ok(Parcel::from_vec(reply_data))
}
