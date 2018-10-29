extern crate v4l2;

use std::io;

use v4l2::sys::ioctl::pix_fmt::V4L2_PIX_FMT_NV12;
use v4l2::Capture;

fn main() -> io::Result<()> {
    let mut capture = Capture::with_device("/dev/video0")
        .input(0) // VFE는 VIDIOC_S_INPUT을 하지 않으면 죽는다.
        .video_size(1920, 1080)
        .pixel_format(V4L2_PIX_FMT_NV12)
        .open()?;
    println!("open");

    // Get actual pixel format
    let _pix_fmt = capture.pix_format()?;

    capture.prepare_mmapped(3)?;
    println!("prepare");

    capture.start()?;
    println!("start");

    while let Ok((buf, _mmap)) = capture.take_frame() {
        println!(
            "used {} flags {:08x} field {:?} seq {} length {} input {} t {}/{}",
            buf.bytesused, buf.flags, buf.field, buf.sequence, buf.length, buf.input,
            buf.timestamp.tv_sec, buf.timestamp.tv_usec
        );
        capture.return_frame(&buf)?;
    }

    capture.stop()?;
    println!("stop");

    Ok(())
}
