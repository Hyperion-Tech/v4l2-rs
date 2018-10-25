#![allow(non_camel_case_types)]

use core::mem;

use libc::{c_char, c_int, c_ulong, c_void, timespec, timeval};
use nix::sys::ioctl::ioctl_num_type;

// videodev2.h

pub const VIDEO_MAX_PLANES: usize = 8;

macro_rules! v4l2_fourcc {
    ( $a:expr, $b:expr, $c:expr, $d:expr ) => {
        ($a as u32) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24)
    };
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum v4l2_field {
    V4L2_FIELD_ANY = 0,
    V4L2_FIELD_NONE = 1,
    V4L2_FIELD_TOP = 2,
    V4L2_FIELD_BOTTOM = 3,
    V4L2_FIELD_INTERLACED = 4,
    V4L2_FIELD_SEQ_TB = 5,
    V4L2_FIELD_SEQ_BT = 6,
    V4L2_FIELD_ALTERNATE = 7,
    V4L2_FIELD_INTERLACED_TB = 8,
    V4L2_FIELD_INTERLACED_BT = 9,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum v4l2_memory {
    V4L2_MEMORY_MMAP = 1,
    V4L2_MEMORY_USERPTR = 2,
    V4L2_MEMORY_OVERLAY = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum v4l2_colorspace {
    V4L2_COLORSPACE_SMPTE170M = 1,
    V4L2_COLORSPACE_SMPTE240M = 2,
    V4L2_COLORSPACE_REC709 = 3,
    V4L2_COLORSPACE_BT878 = 4,
    V4L2_COLORSPACE_470_SYSTEM_M = 5,
    V4L2_COLORSPACE_470_SYSTEM_BG = 6,
    V4L2_COLORSPACE_JPEG = 7,
    V4L2_COLORSPACE_SRGB = 8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum v4l2_buf_type {
    V4L2_BUF_TYPE_VIDEO_CAPTURE = 1,
    V4L2_BUF_TYPE_VIDEO_OUTPUT = 2,
    V4L2_BUF_TYPE_VIDEO_OVERLAY = 3,
    V4L2_BUF_TYPE_VBI_CAPTURE = 4,
    V4L2_BUF_TYPE_VBI_OUTPUT = 5,
    V4L2_BUF_TYPE_SLICED_VBI_CAPTURE = 6,
    V4L2_BUF_TYPE_SLICED_VBI_OUTPUT = 7,
    /* Experimental */
    V4L2_BUF_TYPE_VIDEO_OUTPUT_OVERLAY = 8,
    V4L2_BUF_TYPE_VIDEO_CAPTURE_MPLANE = 9,
    V4L2_BUF_TYPE_VIDEO_OUTPUT_MPLANE = 10,
    V4L2_BUF_TYPE_PRIVATE = 0x80,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_rect {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_fract {
    pub numerator: u32,
    pub denominator: u32,
}

#[repr(C)]
pub struct v4l2_capability {
    driver: [u8; 16],
    card: [u8; 32],
    bus_info: [u8; 32],
    version: u32,
    capabilities: u32,
    device_caps: u32,
    reserved: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_pix_format {
    pub width: u32,
    pub height: u32,
    pub pixelformat: u32,
    pub field: v4l2_field,
    pub bytesperline: u32,
    pub sizeimage: u32,
    pub colorspace: v4l2_colorspace,
    pub private: u32,
    pub rot_angle: u32,
    pub subchannel: *mut v4l2_pix_format,
}

pub mod pix_fmt {
    pub const V4L2_PIX_FMT_YVU420: u32 = v4l2_fourcc!('Y', 'V', '1', '2'); /* 12  YVU 4:2:0     */
    pub const V4L2_PIX_FMT_YUV420: u32 = v4l2_fourcc!('Y', 'U', '1', '2'); /* 12  YUV 4:2:0     */
    pub const V4L2_PIX_FMT_YUYV: u32 = v4l2_fourcc!('Y', 'U', 'Y', 'V'); /* 16  YUV 4:2:2     */
    pub const V4L2_PIX_FMT_NV12: u32 = v4l2_fourcc!('N', 'V', '1', '2'); /* 12  Y/CbCr 4:2:0  */
    pub const V4L2_PIX_FMT_NV21: u32 = v4l2_fourcc!('N', 'V', '2', '1'); /* 12  Y/CrCb 4:2:0  */

    // /* compressed formats */
    pub const V4L2_PIX_FMT_MJPEG: u32 = v4l2_fourcc!('M', 'J', 'P', 'G'); /* Motion-JPEG   */
    pub const V4L2_PIX_FMT_JPEG: u32 = v4l2_fourcc!('J', 'P', 'E', 'G'); /* JFIF JPEG     */
    // #define V4L2_PIX_FMT_DV       v4l2_fourcc('d', 'v', 's', 'd') /* 1394          */
    // #define V4L2_PIX_FMT_MPEG     v4l2_fourcc('M', 'P', 'E', 'G') /* MPEG-1/2/4 Multiplexed */
    pub const V4L2_PIX_FMT_H264: u32 = v4l2_fourcc!('H', '2', '6', '4'); /* H264 with start codes */
    pub const V4L2_PIX_FMT_H264_NO_SC: u32 = v4l2_fourcc!('A', 'V', 'C', '1'); /* H264 without start codes */
    // #define V4L2_PIX_FMT_H264_MVC v4l2_fourcc('M', '2', '6', '4') /* H264 MVC */
    // #define V4L2_PIX_FMT_H263     v4l2_fourcc('H', '2', '6', '3') /* H263          */
    // #define V4L2_PIX_FMT_MPEG1    v4l2_fourcc('M', 'P', 'G', '1') /* MPEG-1 ES     */
    pub const V4L2_PIX_FMT_MPEG2: u32 = v4l2_fourcc!('M', 'P', 'G', '2'); /* MPEG-2 ES     */
    // #define V4L2_PIX_FMT_MPEG4    v4l2_fourcc('M', 'P', 'G', '4') /* MPEG-4 part 2 ES */
    // #define V4L2_PIX_FMT_XVID     v4l2_fourcc('X', 'V', 'I', 'D') /* Xvid           */
    // #define V4L2_PIX_FMT_VC1_ANNEX_G v4l2_fourcc('V', 'C', '1', 'G') /* SMPTE 421M Annex G compliant stream */
    // #define V4L2_PIX_FMT_VC1_ANNEX_L v4l2_fourcc('V', 'C', '1', 'L') /* SMPTE 421M Annex L compliant stream */
    pub const V4L2_PIX_FMT_VP8: u32 = v4l2_fourcc!('V', 'P', '8', '0'); /* VP8 */
}

#[repr(C)]
#[derive(Clone)]
pub struct v4l2_fmtdesc {
    pub index: u32,
    pub typ: v4l2_buf_type,
    pub flags: u32,
    pub description: [u8; 32],
    pub pixelformat: u32,
    pub reserved: [u32; 4],
}

#[repr(u32)]
pub enum v4l2_frmsizetypes {
    V4L2_FRMSIZE_TYPE_DISCRETE = 1,
    V4L2_FRMSIZE_TYPE_CONTINUOUS = 2,
    V4L2_FRMSIZE_TYPE_STEPWISE = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_frmsize_discrete {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_frmsize_stepwise {
    pub min_width: u32,
    pub max_width: u32,
    pub step_width: u32,
    pub min_height: u32,
    pub max_height: u32,
    pub step_height: u32,
}

#[repr(C)]
pub union _v4l2_frmsizeenum_u {
    pub discrete: v4l2_frmsize_discrete,
    pub stepwise: v4l2_frmsize_stepwise,
}

#[repr(C)]
pub struct v4l2_frmsizeenum {
    pub index: u32,
    pub pixel_format: u32,
    pub typ: v4l2_frmsizetypes,

    pub u: _v4l2_frmsizeenum_u,

    pub reserved: [u32; 2],
}

#[repr(C)]
#[derive(Clone)]
pub struct v4l2_timecode {
    typ: u32,
    flags: u32,
    frames: u8,
    seconds: u8,
    minutes: u8,
    hours: u8,
    userbits: [u8; 4],
}

#[repr(C)]
pub struct v4l2_requestbuffers {
    pub count: u32,
    pub typ: v4l2_buf_type,
    pub memory: v4l2_memory,
    pub reserved: [u32; 2],
}

#[repr(C)]
pub union _v4l2_plane_m {
    mem_offset: u32,
    userptr: c_ulong,
}

#[repr(C)]
pub struct v4l2_plane {
    bytesused: u32,
    length: u32,
    m: _v4l2_plane_m,
    data_offset: u32,
    reserved: [u32; 11],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union _v4l2_buffer_m {
    pub offset: u32,
    pub userptr: c_ulong,
    pub planes: *mut v4l2_plane,
}

#[repr(C)]
#[derive(Clone)]
pub struct v4l2_buffer {
    pub index: u32,
    pub typ: v4l2_buf_type,
    pub bytesused: u32,
    pub flags: u32,
    pub field: v4l2_field,
    pub timestamp: timeval,
    pub timecode: v4l2_timecode,
    pub sequence: u32,

    /* memory location */
    pub memory: v4l2_memory,
    pub m: _v4l2_buffer_m,
    pub length: u32,
    pub input: u32,
    pub reserved: u32,
}

#[repr(C)]
pub struct v4l2_clip {
    c: v4l2_rect,
    next: *mut v4l2_clip,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_window {
    w: v4l2_rect,
    field: v4l2_field,
    chromakey: u32,
    clips: *mut v4l2_clip,
    clipcount: u32,
    bitmap: *mut c_void,
    global_alpha: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_captureparm {
    pub capability: u32,
    pub capturemode: u32,
    pub timeperframe: v4l2_fract,
    pub extendedmode: u32,
    pub readbuffers: u32,
    pub reserved: [u32; 4],
}

pub const V4L2_MODE_HIGHQUALITY: u32 = 0x0001; /*  High quality imaging mode */

pub const V4L2_CAP_TIMEPERFRAME: u32 = 0x1000; /*  timeperframe field is supported */

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_outputparm {
    pub capability: u32,
    pub outputmode: u32,
    pub timeperframe: v4l2_fract,
    pub extendedmode: u32,
    pub writebuffers: u32,
    pub reserved: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_vbi_format {
    sampling_rate: u32,
    offset: u32,
    samples_per_line: u32,
    sample_format: u32,
    start: [i32; 2],
    count: [u32; 2],
    flags: u32,
    reserved: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_sliced_vbi_format {
    service_set: u16,
    // __u16   service_lines[2][24];
    io_size: u32,
    reserved: [u32; 2],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct v4l2_plane_pix_format {
    sizeimage: u32,
    bytesperline: u16,
    reserved: [u16; 7],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct v4l2_pix_format_mplane {
    width: u32,
    height: u32,
    pixelformat: u32,
    field: v4l2_field,
    colorspace: v4l2_colorspace,

    plane_fmt: [v4l2_plane_pix_format; VIDEO_MAX_PLANES],
    num_planes: u8,
    reserved: [u8; 11],
}

#[repr(C)]
pub union v4l2_format_fmt {
    pub pix: v4l2_pix_format,
    pub pix_mp: v4l2_pix_format_mplane,
    pub win: v4l2_window,
    pub vbi: v4l2_vbi_format,
    pub sliced: v4l2_sliced_vbi_format,
    pub raw_data: [u8; 200],
}

#[repr(C)]
pub struct v4l2_format {
    pub typ: v4l2_buf_type,
    pub fmt: v4l2_format_fmt,
}

#[repr(C)]
pub union _v4l2_streamparm_parm {
    pub capture: v4l2_captureparm,
    pub output: v4l2_outputparm,
    pub raw_data: [u8; 200],
}

#[repr(C)]
pub struct v4l2_streamparm {
    pub typ: v4l2_buf_type,
    pub parm: _v4l2_streamparm_parm,
}

pub const V4L2_EVENT_PRIVATE_START: u32 = 0x08000000;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct v4l2_event_vsync {
    field: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union _v4l2_event_ctrl_value {
    value: i32,
    value64: i64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_event_ctrl {
    changes: u32,
    typ: u32,
    v: _v4l2_event_ctrl_value,
    flags: u32,
    minimum: i32,
    maximum: i32,
    step: i32,
    default_value: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_event_frame_sync {
    frame_sequence: u32,
}

#[repr(C)]
pub union _v4l2_event_u {
    pub vsync: v4l2_event_vsync,
    pub ctrl: v4l2_event_ctrl,
    pub frame_sync: v4l2_event_frame_sync,
    pub data: [u8; 64],
}

#[repr(C)]
pub struct v4l2_event {
    pub typ: u32,
    pub u: _v4l2_event_u,
    pub pending: u32,
    pub sequence: u32,
    pub timestamp: timespec,
    pub id: u32,
    pub reserved: [u32; 8],
}

#[repr(C)]
pub struct v4l2_event_subscription {
    pub typ: u32,
    pub id: u32,
    pub flags: u32,
    pub reserved: [u32; 5],
}

#[repr(C)]
pub union _v4l2_dbg_match_u {
    addr: u32,
    name: [c_char; 32],
}

#[repr(C, packed)]
pub struct v4l2_dbg_match {
    typ: u32,
    u: _v4l2_dbg_match_u,
}

#[repr(C, packed)]
pub struct v4l2_dbg_register {
    pub match_: v4l2_dbg_match,
    pub size: u32,
    pub reg: u64,
    pub val: u64,
}

pub const VIDIOC_QUERYCAP: ioctl_num_type =
    request_code_read!(b'V', 0, mem::size_of::<v4l2_capability>());
pub const VIDIOC_ENUM_FMT: ioctl_num_type =
    request_code_readwrite!(b'V', 2, mem::size_of::<v4l2_fmtdesc>());
pub const VIDIOC_G_FMT: ioctl_num_type =
    request_code_readwrite!(b'V', 4, mem::size_of::<v4l2_format>());
pub const VIDIOC_S_FMT: ioctl_num_type =
    request_code_readwrite!(b'V', 5, mem::size_of::<v4l2_format>());
pub const VIDIOC_REQBUFS: ioctl_num_type =
    request_code_readwrite!(b'V', 8, mem::size_of::<v4l2_requestbuffers>());
pub const VIDIOC_QUERYBUF: ioctl_num_type =
    request_code_readwrite!(b'V', 9, mem::size_of::<v4l2_buffer>());

pub const VIDIOC_QBUF: ioctl_num_type =
    request_code_readwrite!(b'V', 15, mem::size_of::<v4l2_buffer>());
pub const VIDIOC_DQBUF: ioctl_num_type =
    request_code_readwrite!(b'V', 17, mem::size_of::<v4l2_buffer>());
pub const VIDIOC_STREAMON: ioctl_num_type = request_code_write!(b'V', 18, mem::size_of::<c_int>());
pub const VIDIOC_STREAMOFF: ioctl_num_type = request_code_write!(b'V', 19, mem::size_of::<c_int>());
pub const VIDIOC_G_PARM: ioctl_num_type =
    request_code_readwrite!(b'V', 21, mem::size_of::<v4l2_streamparm>());
pub const VIDIOC_S_PARM: ioctl_num_type =
    request_code_readwrite!(b'V', 22, mem::size_of::<v4l2_streamparm>());
pub const VIDIOC_G_INPUT: ioctl_num_type = request_code_read!(b'V', 38, mem::size_of::<c_int>());
pub const VIDIOC_S_INPUT: ioctl_num_type =
    request_code_readwrite!(b'V', 39, mem::size_of::<c_int>());

pub const VIDIOC_ENUM_FRAMESIZES: ioctl_num_type =
    request_code_readwrite!(b'V', 74, mem::size_of::<v4l2_frmsizeenum>());

pub const VIDIOC_DBG_S_REGISTER: ioctl_num_type =
    request_code_write!(b'V', 79, mem::size_of::<v4l2_dbg_register>());
pub const VIDIOC_DBG_G_REGISTER: ioctl_num_type =
    request_code_readwrite!(b'V', 80, mem::size_of::<v4l2_dbg_register>());

pub const VIDIOC_DQEVENT: ioctl_num_type =
    request_code_read!(b'V', 89, mem::size_of::<v4l2_event>());
pub const VIDIOC_SUBSCRIBE_EVENT: ioctl_num_type =
    request_code_write!(b'V', 90, mem::size_of::<v4l2_event_subscription>());

mod sunxi {
    pub const V4L2_MODE_VIDEO: u32 = 0x0002; /*  Added by raymonxiu For video capture */
    pub const V4L2_MODE_IMAGE: u32 = 0x0003; /*  Added by raymonxiu For image capture */
    pub const V4L2_MODE_PREVIEW: u32 = 0x0004;  /*  Added by raymonxiu For preview capture */
}

pub use self::sunxi::*;
