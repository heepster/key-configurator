use evdev_rs::{InputEvent, enums::{EV_KEY, EventCode, EventType}};
use std::{collections::HashMap, fs::{self, File}};
use xcb::Connection;

use std::collections::HashSet;

mod emitter;
mod kb_out;
mod kb_in;

fn handle_meta_combo(fd: &File, keys_pressed: &HashSet<EV_KEY>, secondary_key: &EV_KEY) -> bool {
    if keys_pressed.contains(&EV_KEY::KEY_LEFTALT) && keys_pressed.contains(&secondary_key) {
        emitter::emit_key(&fd, EV_KEY::KEY_LEFTALT, 0);

        emitter::emit_key_sequence(&fd, vec![
            EV_KEY::KEY_LEFTCTRL,
            secondary_key.clone()
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

struct Config {
    pub singles: HashMap<EV_KEY, EV_KEY>,
}

fn main() {

    //let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    //loop  {
    //    let reply = xcb::xproto::get_input_focus(&conn).get_reply().unwrap();
    //    let window_ptr = reply.focus();
    //    let window = xcb::xproto::get_window_attributes(&conn, window_ptr).get_reply().unwrap();
    //    window.

    //    println!("{}", window);
    //    std::thread::sleep(std::time::Duration::from_secs(3));
    //}

    let single_transform_cfg = [
        [EV_KEY::KEY_CAPSLOCK,   EV_KEY::KEY_LEFTCTRL],
        [EV_KEY::KEY_F1,         EV_KEY::KEY_PLAYPAUSE],
    ];

    let secondary_keys = vec![
      EV_KEY::KEY_C,
      EV_KEY::KEY_V,
      EV_KEY::KEY_A,
    ];

    let mut config = Config { singles: HashMap::new() };
    for tuple in single_transform_cfg.iter() {
        let from = tuple[0];
        let to = tuple[1];
        config.singles.insert(from, to);
    }

    let keyboard_fd_path = "/dev/input/by-path/platform-i8042-serio-0-event-kbd";
    let device_in = kb_in::KeyboardInput::new(keyboard_fd_path);

    let device_out = kb_out::open_device();

    let mut keys_pressed = HashSet::<EV_KEY>::new();

    loop {
        let event_result = device_in.next_event();
        if event_result.is_err() {
            println!("{}", event_result.err().unwrap());
            continue;
        }

        let event = event_result.ok().unwrap();

        if event.event_type().unwrap() == EventType::EV_SYN ||
           event.event_type().unwrap() == EventType::EV_MSC {
            continue;
        }

        let mut key_code_opt = get_event_key(&event);

        // Handle single transformations
        let input_key_code = key_code_opt.unwrap();

        if config.singles.contains_key(&input_key_code) {
            let key_code = config.singles.get(&input_key_code).unwrap().clone();
            println!("Transforming {:?}", key_code);
            key_code_opt = Some(key_code);
        }


        // Handle combo transformations

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
        let mut meta_handled = false;
        for secondary_key in &secondary_keys {
            if handle_meta_combo(&device_out, &keys_pressed, &secondary_key) {
                meta_handled = true;
                break;
            }
        }

        // If we didn't handle meta combo, just pass through the key event
        if !meta_handled {
            let key_code = key_code_opt.unwrap();
            let event_code = EventCode::EV_KEY(key_code);
            println!("Emitting {}, {}", event_code, event.value);
            emitter::emit_event(&device_out, event_code, event.value);
        }
    }
}

