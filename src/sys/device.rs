use std::ffi::CString;
use std::io;
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
use std::path::Path;

use libc;

use super::ioctl::*;

fn cvt(i: libc::c_int) -> io::Result<libc::c_int> {
    if i == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(i)
    }
}

/// Video4Linux 장치 파일 디스크립터를 나타내는 구조체.
///
#[derive(Debug)]
pub struct V4l2Device {
    fd: libc::c_int,
}

impl V4l2Device {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<V4l2Device> {
        use libc::{EINVAL, O_RDWR};

        let cstr = match CString::new(path.as_ref().as_os_str().as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(io::Error::from_raw_os_error(EINVAL)),
        };

        Ok(V4l2Device {
            fd: cvt(unsafe { libc::open(cstr.as_ptr(), O_RDWR) })?,
        })
    }

    pub fn capability(&self) -> io::Result<v4l2_capability> {
        unsafe {
            let mut caps = mem::zeroed::<v4l2_capability>();
            cvt(libc::ioctl(self.fd, VIDIOC_QUERYCAP, &mut caps)).map(|_| caps)
        }
    }

    fn enum_format(&self, buf_type: v4l2_buf_type, index: u32) -> io::Result<v4l2_fmtdesc> {
        unsafe {
            let mut fmtdesc = mem::zeroed::<v4l2_fmtdesc>();
            fmtdesc.index = index;
            fmtdesc.typ = buf_type;

            cvt(libc::ioctl(self.fd, VIDIOC_ENUM_FMT, &mut fmtdesc)).map(|_| fmtdesc)
        }
    }

    pub fn supported_formats<'a>(&'a self, buf_type: v4l2_buf_type) -> SupportedFormats<'a> {
        SupportedFormats {
            dev: self,
            buf_type,
            index: 0,
        }
    }

    fn enum_frame_size(&self, pixel_format: u32, index: u32) -> io::Result<v4l2_frmsizeenum> {
        unsafe {
            let mut frmsize = mem::zeroed::<v4l2_frmsizeenum>();
            frmsize.index = index;
            frmsize.pixel_format = pixel_format;

            cvt(libc::ioctl(self.fd, VIDIOC_ENUM_FRAMESIZES, &mut frmsize)).map(|_| frmsize)
        }
    }

    pub fn supported_frame_sizes<'a>(&'a self, pixel_format: u32) -> SupportedFrameSizes<'a> {
        SupportedFrameSizes {
            dev: self,
            pixel_format: pixel_format,
            index: 0,
        }
    }

    /// Returns current `v4l2_format` for the specified `v4l2_buf_type`.
    ///
    fn format(&self, buf_type: v4l2_buf_type) -> io::Result<v4l2_format> {
        unsafe {
            let mut fmt = v4l2_format {
                typ: buf_type,
                fmt: mem::zeroed(),
            };
            cvt(libc::ioctl(self.fd, VIDIOC_G_FMT, &mut fmt)).map(|_| fmt)
        }
    }

    fn set_format(&self, fmt: &mut v4l2_format) -> io::Result<()> {
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_S_FMT, fmt)).map(|_| ()) }
    }

    /// Returns current `v4l2_format` for the specified `v4l2_buf_type` assuming
    /// it is in `v4l2_pix_format`.
    ///
    fn pix_format(&self, buf_type: v4l2_buf_type) -> io::Result<v4l2_pix_format> {
        self.format(buf_type).map(|fmt| unsafe { fmt.fmt.pix })
    }

    /// Sets `v4l2_pix_format` for the specified `v4l2_buf_type`.
    ///
    fn set_pix_format(
        &self,
        buf_type: v4l2_buf_type,
        fmt: &v4l2_pix_format,
    ) -> io::Result<v4l2_pix_format> {
        let mut fmt = v4l2_format {
            typ: buf_type,
            fmt: v4l2_format_fmt { pix: *fmt },
        };
        self.set_format(&mut fmt).map(|_| unsafe { fmt.fmt.pix })
    }

    pub fn capture_format(&self) -> io::Result<v4l2_pix_format> {
        self.pix_format(v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE)
    }

    pub fn set_capture_format(&self, fmt: &v4l2_pix_format) -> io::Result<v4l2_pix_format> {
        self.set_pix_format(v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE, fmt)
    }

    pub fn output_format(&self) -> io::Result<v4l2_pix_format> {
        self.pix_format(v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT)
    }

    pub fn set_output_format(&self, fmt: &v4l2_pix_format) -> io::Result<v4l2_pix_format> {
        self.set_pix_format(v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT, fmt)
    }

    fn stream_parm(&self, buf_type: v4l2_buf_type) -> io::Result<v4l2_streamparm> {
        unsafe {
            let mut parm = v4l2_streamparm {
                typ: buf_type,
                parm: mem::zeroed(),
            };
            cvt(libc::ioctl(self.fd, VIDIOC_G_PARM, &mut parm)).map(|_| parm)
        }
    }

    fn set_stream_parm(&self, parm: &mut v4l2_streamparm) -> io::Result<()> {
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_S_PARM, parm)).map(|_| ()) }
    }

    pub fn capture_parm(&self) -> io::Result<v4l2_captureparm> {
        self.stream_parm(v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE)
            .map(|parm| unsafe { parm.parm.capture })
    }

    pub fn set_capture_parm(&self, parm: &v4l2_captureparm) -> io::Result<v4l2_captureparm> {
        let mut parm = v4l2_streamparm {
            typ: v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE,
            parm: _v4l2_streamparm_parm { capture: *parm },
        };
        self.set_stream_parm(&mut parm)
            .map(|_| unsafe { parm.parm.capture })
    }

    pub fn output_parm(&self) -> io::Result<v4l2_outputparm> {
        self.stream_parm(v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT)
            .map(|parm| unsafe { parm.parm.output })
    }

    pub fn set_output_parm(&self, parm: &v4l2_outputparm) -> io::Result<v4l2_outputparm> {
        let mut parm = v4l2_streamparm {
            typ: v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT,
            parm: _v4l2_streamparm_parm { output: *parm },
        };
        self.set_stream_parm(&mut parm)
            .map(|_| unsafe { parm.parm.output })
    }

    pub fn input(&self) -> io::Result<i32> {
        let mut input = -1;
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_G_INPUT, &mut input)).map(|_| input) }
    }

    pub fn set_input(&self, input: i32) -> io::Result<()> {
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_S_INPUT, &input)).map(|_| ()) }
    }

    pub fn request_buffers(
        &self,
        buf_type: v4l2_buf_type,
        memory: v4l2_memory,
        count: usize,
    ) -> io::Result<usize> {
        let mut reqbufs = v4l2_requestbuffers {
            typ: buf_type,
            count: count as u32,
            memory: memory,
            reserved: [0; 2],
        };
        unsafe {
            cvt(libc::ioctl(self.fd, VIDIOC_REQBUFS, &mut reqbufs)).map(|_| reqbufs.count as usize)
        }
    }

    pub fn buffer(
        &self,
        buf_type: v4l2_buf_type,
        memory: v4l2_memory,
        index: usize,
    ) -> io::Result<v4l2_buffer> {
        unsafe {
            let mut buf = mem::zeroed::<v4l2_buffer>();
            buf.typ = buf_type;
            buf.memory = memory;
            buf.index = index as u32;

            cvt(libc::ioctl(self.fd, VIDIOC_QUERYBUF, &mut buf)).map(|_| buf)
        }
    }

    pub fn buffers<'a>(&'a self, buf_type: v4l2_buf_type, memory: v4l2_memory) -> Buffers<'a> {
        Buffers {
            dev: self,
            typ: buf_type,
            memory: memory,
            index: 0,
        }
    }

    pub fn queue_buffer(&self, buf: &v4l2_buffer) -> io::Result<()> {
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_QBUF, buf)).map(|_| ()) }
    }

    pub fn dequeue_buffer(
        &self,
        buf_type: v4l2_buf_type,
        memory: v4l2_memory,
    ) -> io::Result<v4l2_buffer> {
        let mut buf: v4l2_buffer = unsafe { mem::zeroed() };
        buf.typ = buf_type;
        buf.memory = memory;
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_DQBUF, &mut buf)).map(|_| buf) }
    }

    pub fn stream_on(&self, buf_type: v4l2_buf_type) -> io::Result<()> {
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_STREAMON, &buf_type)).map(|_| ()) }
    }

    pub fn stream_off(&self, buf_type: v4l2_buf_type) -> io::Result<()> {
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_STREAMOFF, &buf_type)).map(|_| ()) }
    }

    pub fn subscribe_event(&self, event: u32) -> io::Result<()> {
        let sub = v4l2_event_subscription {
            typ: event,
            id: 0,
            flags: 0,
            reserved: [0; 5],
        };
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_SUBSCRIBE_EVENT, &sub)).map(|_| ()) }
    }

    pub fn dequeue_event(&self) -> io::Result<v4l2_event> {
        unsafe {
            let mut evt: v4l2_event = mem::uninitialized();
            cvt(libc::ioctl(self.fd, VIDIOC_DQEVENT, &mut evt)).map(|_| evt)
        }
    }

    // pub fn events(&self) -> Events {
    //     Events { dev: self }
    // }
}

impl V4l2Device {
    pub fn get_register(&self, reg: &mut v4l2_dbg_register) -> io::Result<()> {
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_DBG_G_REGISTER, reg)).map(|_| ()) }
    }

    pub fn set_register(&self, reg: &v4l2_dbg_register) -> io::Result<()> {
        unsafe { cvt(libc::ioctl(self.fd, VIDIOC_DBG_S_REGISTER, reg)).map(|_| ()) }
    }
}

impl Drop for V4l2Device {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let _ = libc::close(self.fd);
        }
    }
}

impl AsRawFd for V4l2Device {
    #[inline]
    fn as_raw_fd(&self) -> i32 {
        self.fd
    }
}

impl IntoRawFd for V4l2Device {
    #[inline]
    fn into_raw_fd(self) -> i32 {
        self.fd
    }
}

impl FromRawFd for V4l2Device {
    #[inline]
    unsafe fn from_raw_fd(fd: i32) -> V4l2Device {
        V4l2Device { fd }
    }
}

pub struct SupportedFormats<'a> {
    dev: &'a V4l2Device,
    buf_type: v4l2_buf_type,
    index: u32,
}

impl<'a> Iterator for SupportedFormats<'a> {
    type Item = v4l2_fmtdesc;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(fmtdesc) = self.dev.enum_format(self.buf_type, self.index) {
            self.index += 1;
            Some(fmtdesc)
        } else {
            None
        }
    }
}

pub struct SupportedFrameSizes<'a> {
    dev: &'a V4l2Device,
    pixel_format: u32,
    index: u32,
}

impl<'a> Iterator for SupportedFrameSizes<'a> {
    type Item = v4l2_frmsizeenum;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(frmsize) = self.dev.enum_frame_size(self.pixel_format, self.index) {
            self.index += 1;
            Some(frmsize)
        } else {
            None
        }
    }
}

pub struct Buffers<'a> {
    dev: &'a V4l2Device,
    typ: v4l2_buf_type,
    memory: v4l2_memory,
    index: usize,
}

impl<'a> Iterator for Buffers<'a> {
    type Item = v4l2_buffer;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(buffer) = self.dev.buffer(self.typ, self.memory, self.index) {
            self.index += 1;
            Some(buffer)
        } else {
            None
        }
    }
}

// pub struct Events<'a> {
//     dev: &'a V4l2Device,
// }

// impl<'a> Iterator for Events<'a> {
//     type Item = v4l2_event;

//     fn next(&mut self) -> Option<v4l2_event> {
//         self.dev.dequeue_event().ok()
//     }
// }
