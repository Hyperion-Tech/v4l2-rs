#[macro_use]
extern crate nix;

pub mod sys {
    mod device;
    pub mod ioctl;

    pub use self::device::V4l2Device;
}

mod capture;

pub use self::capture::Capture;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
