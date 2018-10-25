use std::io;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::slice;

use crate::sys::ioctl::v4l2_buffer;

pub struct MappedBuffer {
    buf: *mut libc::c_void,
    len: usize,
}

impl MappedBuffer {
    pub fn new<D: AsRawFd>(fd: &D, buffer: &v4l2_buffer) -> io::Result<MappedBuffer> {
        unsafe {
            let map = libc::mmap(
                ptr::null_mut(),
                buffer.length as usize,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd.as_raw_fd(),
                buffer.m.offset as libc::off_t,
            );
            if map != libc::MAP_FAILED {
                Ok(MappedBuffer {
                    buf: map,
                    len: buffer.length as usize,
                })
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buf as *const u8, self.len) }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.buf as *mut u8, self.len) }
    }
}

impl Drop for MappedBuffer {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.buf, self.len);
        }
    }
}
