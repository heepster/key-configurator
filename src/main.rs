use evdev_rs::{InputEvent, enums::{EV_KEY, EventCode, EventType}};
use std::{fs::{File}};
use evdev_rs::ReadFlag;

use std::collections::HashSet;

mod emitter;
mod kb_out;
mod kb_in;

fn handle_meta_combo(fd: &File, keys_pressed: &HashSet<EV_KEY>, secondary_key: EV_KEY) -> bool {
    if keys_pressed.contains(&EV_KEY::KEY_LEFTALT) && keys_pressed.contains(&secondary_key) {
        emitter::emit_key(&fd, EV_KEY::KEY_LEFTALT, 0);

        emitter::emit_key_sequence(&fd, vec![
            EV_KEY::KEY_LEFTCTRL,
            secondary_key
        ]);
        return true;
    }
    return false;
}

fn get_event_key(event: &InputEvent) -> Option<EV_KEY> {
    match event.event_code {
        EventCode::EV_KEY(key_code) =>
            return Some(key_code),
        _ =>
            return None,
    }
}

fn main() {
    let fd_path = "/dev/input/by-path/platform-i8042-serio-0-event-kbd";
    let device_in = kb_in::open_device(fd_path);
    let device_out = kb_out::open_device();

    let mut keys_pressed = HashSet::<EV_KEY>::new();

    loop {
        let event_result = device_in.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING).map(|val| val.1);

        if event_result.is_err() {
            println!("{}", event_result.err().unwrap());
            continue;
        }

        let event = event_result.ok().unwrap();

        if event.event_type().unwrap() == EventType::EV_SYN ||
           event.event_type().unwrap() == EventType::EV_MSC {
            continue;
        }

        let key_code_opt = get_event_key(&event);

        if event.value == 0 {
            if key_code_opt.is_some() {
                keys_pressed.remove(&key_code_opt.unwrap());
            }
        } else if event.value == 1 {
            if key_code_opt.is_some() {
                keys_pressed.insert(key_code_opt.unwrap());
            }
        } else {
          // no-op
        }

        let secondary_keys = vec![
          EV_KEY::KEY_C,
          EV_KEY::KEY_V,
          EV_KEY::KEY_A,
        ];

        let mut meta_handled = false;
        for secondary_key in secondary_keys {
            if handle_meta_combo(&device_out, &keys_pressed, secondary_key) {
                meta_handled = true;
                break;
            }
        }

        // If we didn't handle meta combo, just pass through the key event
        if !meta_handled {
            println!("Emitting {}, {}", event.event_code, event.value);
            emitter::emit_event(&device_out, event.event_code, event.value);
        }
    }
}

