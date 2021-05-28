use evdev_rs::{Device, InputEvent, TimeVal, enums::{EV_KEY, EV_SYN, EventCode, EventType}};
use uinput_sys::uinput_user_dev;
use std::{fs::{File, OpenOptions}, io::Write, os::unix::prelude::AsRawFd};
use evdev_rs::ReadFlag;
use libc::{input_event as raw_event};

use std::collections::HashSet;
use std::mem;
use std::slice;

fn emit_input_event(mut fd: &File, input_event: InputEvent) {
    unsafe {
        let input_bytes = slice::from_raw_parts(
            mem::transmute(&input_event.as_raw() as *const raw_event),
            mem::size_of::<raw_event>(),
        );
        fd.write(input_bytes).expect("Couldn't write input event");
    }
}

fn emit_event(fd: &File, event_code: EventCode, value: i32) {
    let key_input_event = InputEvent::new(
        &TimeVal {
            tv_sec: 0,
            tv_usec: 0,
        },
        &event_code,
        value
    );

    let sync_input_event = InputEvent::new(
        &TimeVal {
            tv_sec: 0,
            tv_usec: 0,
        },
        &EventCode::EV_SYN(EV_SYN::SYN_REPORT),
        0,
    );

    emit_input_event(fd, key_input_event);
    emit_input_event(fd, sync_input_event);
}

fn emit_key(fd: &File, key_code: EV_KEY, value: i32) {
    let event_code = EventCode::EV_KEY(key_code);
    emit_event(fd, event_code, value)
}

fn emit_key_sequence(fd: &File, key_code_list: Vec<EV_KEY>) {
    for key_code in &key_code_list {
        emit_key(fd, key_code.clone(), 1);
    }
    for key_code in &key_code_list {
        emit_key(fd, key_code.clone(), 0);
    }
}

fn handle_meta_combo(fd: &File, keys_pressed: &HashSet<EV_KEY>, secondary_key: EV_KEY) -> bool {
    if keys_pressed.contains(&EV_KEY::KEY_LEFTALT) && keys_pressed.contains(&secondary_key) {
        emit_key(&fd, EV_KEY::KEY_LEFTALT, 0);

        emit_key_sequence(&fd, vec![
            EV_KEY::KEY_LEFTCTRL,
            secondary_key
        ]);
        return true;
    }
    return false;
}

fn main() {
    let mut d_out = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/uinput")
        .unwrap();

        unsafe {
            uinput_sys::ui_set_evbit(d_out.as_raw_fd(), uinput_sys::EV_SYN);
            uinput_sys::ui_set_evbit(d_out.as_raw_fd(), uinput_sys::EV_KEY);

            for key in 0..uinput_sys::KEY_MAX {
                uinput_sys::ui_set_keybit(d_out.as_raw_fd(), key);
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

            d_out.write(uidev_bytes).expect("Couldn't write");

            uinput_sys::ui_dev_create(d_out.as_raw_fd());
        }

    let file = File::open("/dev/input/by-path/platform-i8042-serio-0-event-kbd").unwrap();
    let mut d = Device::new_from_file(file).unwrap();
    d.grab(evdev_rs::GrabMode::Grab).expect("Couldn't grab exclusively");

    let mut keys_pressed = HashSet::<EV_KEY>::new();

    loop {
        let ev = d.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING).map(|val| val.1);

        match ev {
            Ok(ev) => {
                if ev.event_type().unwrap() == EventType::EV_SYN ||
                   ev.event_type().unwrap() == EventType::EV_MSC {
                  continue;
                }

                match ev.value {
                    0 => {
                        match ev.event_code {
                          EventCode::EV_KEY(key_code) => {
                              keys_pressed.remove(&key_code);
                          },
                          _ => {}
                        }
                    },
                    1 => {
                        match ev.event_code {
                          EventCode::EV_KEY(key_code) => {
                              keys_pressed.insert(key_code);
                          },
                          _ => {}
                        }
                    },
                    2 => {

                    }
                    _ => {
                        // no-op
                    }
                }

                let secondary_keys = vec![
                  EV_KEY::KEY_C,
                  EV_KEY::KEY_V,
                  EV_KEY::KEY_A,
                ];

                let mut meta_handled = false;
                for secondary_key in secondary_keys {
                    if handle_meta_combo(&d_out, &keys_pressed, secondary_key) {
                        meta_handled = true;
                        break;
                    }
                }

                // If we didn't handle meta combo, just pass through the key event
                if !meta_handled {
                    println!("Emitting {}, {}", ev.event_code, ev.value);
                    emit_event(&d_out, ev.event_code, ev.value);
                }
            }
            Err(e) => println!("{}", e),
        }
    }
}

