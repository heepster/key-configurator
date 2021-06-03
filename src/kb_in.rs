use std::{fs::{self,File}, io::Error};
use evdev_rs::{Device, InputEvent, enums::EventType};
use evdev_rs::ReadFlag;

pub struct KeyboardInput {
    device: Box<Device>
}

impl KeyboardInput {
    pub fn new(fd_path: &str) -> Self {
      Self {
          device: open_device(fd_path)
      }
    }

    pub fn next_event(&self) -> InputEvent {
        loop {
            let event_result = self.device.next_event(
                ReadFlag::NORMAL |
                ReadFlag::BLOCKING
            ).map(|val| val.1);

            if event_result.is_err() {
                println!("{}", event_result.err().unwrap());
                continue;
            }

            let event = event_result.ok().unwrap();

            if event.event_type().unwrap() == EventType::EV_SYN ||
               event.event_type().unwrap() == EventType::EV_MSC {
                continue;
            }

            return event;
        }
    }
}

fn open_device(file_descriptor_path: &str) -> Box<Device> {
    let file = File::open(file_descriptor_path).unwrap();
    let mut device = Box::new(Device::new_from_file(file).unwrap());
    device.grab(evdev_rs::GrabMode::Grab).expect("Couldn't grab exclusively");
    return device;
}

fn get_keyboard_file_descriptors() -> Vec::<String> {
    let all_input_fds = fs::read_dir("/dev/input/by-path").unwrap();
    let mut keyboard_fds = Vec::<String>::new();
    for fd in all_input_fds {
        // Todo -- safer way than all of these unwraps?
        let path = fd.unwrap().path();
        if str::ends_with(path.to_str().unwrap(), "kbd") {
            keyboard_fds.push(path.to_str().unwrap().to_owned());
        }
    }
    return keyboard_fds;
}


