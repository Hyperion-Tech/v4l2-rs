extern crate v4l2;

use std::io;

use v4l2::Capture;

fn main() -> io::Result<()> {
    let mut capture = Capture::with_device("/dev/video0")
        .input(0) // VFE는 VIDIOC_S_INPUT을 하지 않으면 죽는다.
        .video_size(1920, 1080)
        .open()?;
    println!("open");

    capture.prepare_mmapped(3)?;
    println!("prepare");

    capture.start()?;
    println!("start");

    capture.stop()?;
    println!("stop");

    Ok(())
}
