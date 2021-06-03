use std::{fs::{self,File}, io::Error};
use evdev_rs::{Device, InputEvent, enums::{EV_KEY, EventCode, EventType}};
use evdev_rs::ReadFlag;

pub struct KeyboardInput {
    device: Box<Device>
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum KeyValue {
    On,
    Off,
    Hold
}

type KeyCode = EV_KEY;

#[derive(PartialEq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub value: KeyValue,
}

impl KeyboardInput {
    pub fn new(fd_path: &str) -> Self {
      Self {
          device: open_device(fd_path)
      }
    }

    pub fn next_event(&self) -> KeyEvent {
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
               event.event_type().unwrap() == EventType::EV_MSC ||
               event.event_type().unwrap() != EventType::EV_KEY {
                continue;
            }

            // Todo -- ensure compiler knows event type is EV_KEY
            // so we don't have to return an option
            return get_key_event(&event).unwrap();
        }
    }
}

fn get_key_value(val: i32) -> KeyValue {
    match val {
        0 => KeyValue::Off,
        1 => KeyValue::On,
        2 => KeyValue::Hold,
        _ => KeyValue::Off,
    }
}

fn get_key_event(event: &InputEvent) -> Option<KeyEvent> {
    match event.event_code {
        EventCode::EV_KEY(key_code) =>
            return Some(
                KeyEvent{
                    code: key_code,
                    value: get_key_value(event.value)
                }
            ),
        _ =>
            return None,
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


