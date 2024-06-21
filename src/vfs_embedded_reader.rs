use embedded_io;
use esp_idf_svc::sys::read;
use std::ffi::c_void;
use std::fmt::Debug;
use std::os::fd::RawFd;

pub struct VfsReader {
    file: RawFd,
}

#[derive(Debug)]
pub struct VfsError;

impl embedded_io::Error for VfsError {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl embedded_io::ErrorType for VfsReader {
    type Error = VfsError;
}

impl VfsReader {
    pub fn new(file: RawFd) -> Self {
        Self { file }
    }
}

impl embedded_io::Read for VfsReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let res = unsafe { read(self.file, buf.as_mut_ptr() as *mut c_void, buf.len()) };
        if res >= 0 {
            Ok(res as usize)
        } else {
            Err(VfsError)
        }
    }
}
