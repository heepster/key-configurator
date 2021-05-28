use std::mem;
use std::slice;
use std::{fs::{File, OpenOptions}, io::Write, os::unix::prelude::AsRawFd};
use uinput_sys::uinput_user_dev;

pub fn open_device() -> File {
    let device = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/uinput")
        .unwrap();
    init_output_device(&device);
    return device;
}

pub fn init_output_device(mut device: &File) {
    unsafe {
        uinput_sys::ui_set_evbit(device.as_raw_fd(), uinput_sys::EV_SYN);
        uinput_sys::ui_set_evbit(device.as_raw_fd(), uinput_sys::EV_KEY);

        for key in 0..uinput_sys::KEY_MAX {
            uinput_sys::ui_set_keybit(device.as_raw_fd(), key);
        }

        let mut uidev: uinput_user_dev = mem::zeroed();
        uidev.name[0] = 'k' as i8;
        uidev.name[1] = 'e' as i8;
        uidev.name[2] = 'y' as i8;
        uidev.name[3] = 'c' as i8;
        uidev.name[4] = 'f' as i8;
        uidev.id.bustype = 0x3; // BUS_USB
        uidev.id.vendor = 0x1;
        uidev.id.product = 0x1;
        uidev.id.version = 1;

        let uidev_bytes =
            slice::from_raw_parts(mem::transmute(&uidev), mem::size_of::<uinput_user_dev>());

        device.write(uidev_bytes).expect("Couldn't write");

        uinput_sys::ui_dev_create(device.as_raw_fd());
    }
}
