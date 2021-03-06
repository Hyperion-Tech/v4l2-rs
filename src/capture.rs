use std::fs::File;
use std::io;
use std::mem;
use std::os::unix::io::{AsRawFd, FromRawFd};

use memmap::{MmapMut, MmapOptions};

use crate::sys::uapi::*;
use crate::sys::V4l2Device;

pub struct Capture {
    device: V4l2Device,
    buffers: Vec<MmapMut>,
}

impl Capture {
    fn new(device: V4l2Device) -> Capture {
        Capture {
            device,
            buffers: Vec::new(),
        }
    }

    /// Return current pixel format of capture device.
    pub fn pix_format(&self) -> io::Result<v4l2_pix_format> {
        self.device.capture_format()
    }

    pub fn prepare_mmapped(&mut self, count: usize) -> io::Result<()> {
        // Request buffers
        let n = self.device.request_buffers(
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE,
            v4l2_memory::V4L2_MEMORY_MMAP,
            count,
        )?;

        self.buffers.clear();

        let f = unsafe { File::from_raw_fd(self.device.as_raw_fd()) };

        for buf in self.device.buffers(
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE,
            v4l2_memory::V4L2_MEMORY_MMAP,
        ) {
            let mmap = unsafe {
                MmapOptions::new()
                    .len(buf.length as usize)
                    .offset(buf.m.offset as u64)
                    .map_mut(&f)
            };
            if let Ok(mmap) = mmap {
                self.buffers.push(mmap);
            }
        }
        mem::forget(f);

        if self.buffers.len() != n {}

        Ok(())
    }

    pub fn unprepare(&mut self) {
        self.buffers.clear();
    }

    pub fn start(&self) -> io::Result<()> {
        let mut buf: v4l2_buffer = unsafe { mem::zeroed() };

        buf.typ = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE;
        buf.memory = v4l2_memory::V4L2_MEMORY_MMAP;

        // Queue buffers
        for i in 0..self.buffers.len() {
            buf.index = i as u32;

            self.device.queue_buffer(&buf)?;
        }

        self.device
            .stream_on(v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE)
    }

    pub fn stop(&self) -> io::Result<()> {
        self.device
            .stream_off(v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE)
    }

    pub fn take_frame(&mut self) -> io::Result<(v4l2_buffer, &mut MmapMut)> {
        let buf = self.device.dequeue_buffer(
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE,
            v4l2_memory::V4L2_MEMORY_MMAP,
        )?;

        let mmap = &mut self.buffers[buf.index as usize];

        Ok((buf, mmap))
    }

    pub fn return_frame(&self, buf: &v4l2_buffer) -> io::Result<()> {
        self.device.queue_buffer(buf)
    }

    pub fn with_default<'a>() -> Builder<'a> {
        Builder::default()
    }

    pub fn with_device<'a>(path: &'a str) -> Builder<'a> {
        Builder::with_device(path)
    }
}

pub struct Builder<'a> {
    path: &'a str,
    input: Option<i32>,
    capturemode: u32,
    timeperframe: v4l2_fract,
    format: v4l2_pix_format,
    #[cfg(feature = "sunxi-vfe")]
    _subch: Option<v4l2_pix_format>,
}

impl<'a> Builder<'a> {
    pub fn with_device(path: &'a str) -> Self {
        #[cfg(feature = "sunxi-vfe")]
        use std::ptr;

        Builder {
            path,
            input: None,
            capturemode: 0,
            timeperframe: v4l2_fract {
                numerator: 1,
                denominator: 30,
            },
            #[cfg(not(feature = "sunxi-vfe"))]
            format: v4l2_pix_format {
                width: 0,
                height: 0,
                pixelformat: 0,
                sizeimage: 0,
                field: v4l2_field::V4L2_FIELD_ANY,
                bytesperline: 0,
                colorspace: v4l2_colorspace::V4L2_COLORSPACE_JPEG,
                private: 0,
            },
            #[cfg(feature = "sunxi-vfe")]
            format: v4l2_pix_format {
                width: 0,
                height: 0,
                pixelformat: 0,
                sizeimage: 0,
                field: v4l2_field::V4L2_FIELD_ANY,
                bytesperline: 0,
                colorspace: v4l2_colorspace::V4L2_COLORSPACE_JPEG,
                private: 0,
                rot_angle: 0,
                subchannel: ptr::null_mut(),
            },
            #[cfg(feature = "sunxi-vfe")]
            _subch: None,
        }
    }

    pub fn device(mut self, path: &'a str) -> Self {
        self.path = path;
        self
    }

    pub fn input(mut self, input: i32) -> Self {
        self.input = Some(input);
        self
    }

    pub fn high_quality(mut self) -> Self {
        self.capturemode = V4L2_MODE_HIGHQUALITY;
        self
    }

    pub fn video_size(mut self, width: u32, height: u32) -> Self {
        self.format.width = width;
        self.format.height = height;
        self
    }

    pub fn pixel_format(mut self, fmt: u32) -> Self {
        self.format.pixelformat = fmt;
        self
    }

    pub fn progressive(mut self) -> Self {
        self.format.field = v4l2_field::V4L2_FIELD_NONE;
        self
    }

    pub fn time_per_frame(mut self, num: u32, den: u32) -> Self {
        self.timeperframe.numerator = num;
        self.timeperframe.denominator = den;
        self
    }

    pub fn open(self) -> io::Result<Capture> {
        let video = V4l2Device::open(self.path)?;

        // Ensure pixel format supported for safety.
        // VFE driver crashes if pixel format is not specified.
        if video
            .supported_formats(v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE)
            .find(|fmtdesc| fmtdesc.pixelformat == self.format.pixelformat)
            .is_none()
        {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "unsupported pixel format",
            ));
        }

        if let Some(input) = self.input {
            video.set_input(input)?;
        }

        let mut param = video.capture_parm()?;
        println!("expected read bufs {}", param.readbuffers);

        param.capturemode = self.capturemode;

        if (param.capability & V4L2_CAP_TIMEPERFRAME) != 0 {
            param.timeperframe = self.timeperframe;
        }

        let param = video.set_capture_parm(&param)?;
        println!("capture mode {}", param.capturemode);
        println!(
            "time/frame {}/{}",
            param.timeperframe.numerator, param.timeperframe.denominator
        );

        let _pixfmt = video.set_capture_format(&self.format)?;

        Ok(Capture::new(video))
    }
}

#[cfg(feature = "sunxi-vfe")]
impl<'a> Builder<'a> {
    pub fn video_mode(mut self) -> Self {
        self.capturemode = V4L2_MODE_VIDEO;
        self
    }

    pub fn image_mode(mut self) -> Self {
        self.capturemode = V4L2_MODE_IMAGE;
        self
    }

    pub fn preview_mode(mut self) -> Self {
        self.capturemode = V4L2_MODE_PREVIEW;
        self
    }

    pub fn rotate(&mut self, degree: u32) -> &mut Self {
        self.format.rot_angle = degree;
        self
    }
}

impl<'a> Default for Builder<'a> {
    fn default() -> Builder<'a> {
        Builder::with_device("/dev/video0")
    }
}
