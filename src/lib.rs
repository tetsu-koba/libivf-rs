use byteorder::{ReadBytesExt, WriteBytesExt};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};

const IVF_SIGNATURE: &[u8; 4] = b"DKIF";
pub const IVF_HEADER_SIZE: usize = 32;
pub const IVF_FRAME_HEADER_SIZE: usize = 12;

#[repr(C)]
#[derive(Debug)]
pub struct IvfHeader {
    pub signature: [u8; 4],
    pub version: u16,
    pub header_size: u16,
    pub fourcc: [u8; 4],
    pub width: u16,
    pub height: u16,
    pub framerate_num: u32,
    pub framerate_den: u32,
    pub num_frames: u32,
    unused: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct IvfFrameHeader {
    pub frame_size: u32,
    pub timestamp: u64,
}

pub struct IvfReader {
    pub header: IvfHeader,
    file: File,
}

impl IvfReader {
    pub fn init(mut file: File) -> Result<Self, Box<dyn Error>> {
        let mut header = IvfHeader {
            signature: [0; 4],
            version: 0,
            header_size: 0,
            fourcc: [0; 4],
            width: 0,
            height: 0,
            framerate_num: 0,
            framerate_den: 0,
            num_frames: 0,
            unused: 0,
        };
        file.read_exact(unsafe {
            std::slice::from_raw_parts_mut(
                &mut header as *mut _ as *mut u8,
                std::mem::size_of::<IvfHeader>(),
            )
        })?;
        if header.signature != *IVF_SIGNATURE {
            return Err("Invalid IVF format".into());
        }
        if header.version != 0 || header.header_size != 32 {
            return Err("Unsupported IVF version or header size".into());
        }
        Ok(Self { header, file })
    }

    pub fn read_ivf_frame_header(&mut self) -> io::Result<IvfFrameHeader> {
        let frame_header = IvfFrameHeader {
            frame_size: self.file.read_u32::<byteorder::LittleEndian>()?,
            timestamp: self.file.read_u64::<byteorder::LittleEndian>()?,
        };
        Ok(frame_header)
    }

    pub fn read_frame(&mut self, frame: &mut [u8]) -> io::Result<usize> {
        let size = self.file.read(frame)?;
        Ok(size)
    }

    pub fn skip_frame(&mut self, frame_size: u32) -> io::Result<()> {
        self.file.seek(SeekFrom::Current(frame_size as i64))?;
        Ok(())
    }
}

pub struct IvfWriter {
    file: File,
    frame_count: u32,
}

impl IvfWriter {
    pub fn init(mut file: File, header: &IvfHeader) -> Result<Self, Box<dyn Error>> {
        if header.signature != *IVF_SIGNATURE || header.version != 0 || header.header_size != 32 {
            return Err("Invalid or unsupported IVF header".into());
        }
        file.write_all(unsafe {
            std::slice::from_raw_parts(
                header as *const _ as *const u8,
                std::mem::size_of::<IvfHeader>(),
            )
        })?;
        Ok(Self {
            file,
            frame_count: 0,
        })
    }

    pub fn write_ivf_frame(&mut self, frame: &[u8], timestamp: u64) -> io::Result<()> {
        self.file
            .write_u32::<byteorder::LittleEndian>(frame.len() as u32)?;
        self.file.write_u64::<byteorder::LittleEndian>(timestamp)?;
        self.file.write_all(frame)?;
        self.frame_count += 1;
        Ok(())
    }
}

impl Drop for IvfWriter {
    fn drop(&mut self) {
        let _ = self.file.seek(SeekFrom::Start(24));
        let _ = self
            .file
            .write_u32::<byteorder::LittleEndian>(self.frame_count);
    }
}
