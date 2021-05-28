use std::{fs::{File}};
use evdev_rs::Device;

pub fn open_device(file_descriptor_path: &str) -> Box<Device> {
    let file = File::open(file_descriptor_path).unwrap();
    let mut device = Box::new(Device::new_from_file(file).unwrap());
    device.grab(evdev_rs::GrabMode::Grab).expect("Couldn't grab exclusively");
    return device;
}

