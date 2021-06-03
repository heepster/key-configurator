use evdev_rs::{enums::{EV_KEY, EventCode, EventType}};
use std::{collections::{HashMap, VecDeque}, fs::{File}, os::unix::process};
use xcb::Connection;

use crate::kb_in::KeyValue;

mod emitter;
mod kb_out;
mod kb_in;

fn handle_meta_combo(fd: &File, state: &State, config: &Config, key_code: &EV_KEY) -> bool {
    let current_combo = state.pressed.iter().map(|k| k.clone()).collect::<Vec<EV_KEY>>();
    let current_combo_key = get_combo_key(current_combo);

    if config.combos.contains_key(&current_combo_key) {
        let key_sequence = config.combos.get(&current_combo_key).unwrap().clone();
        emitter::emit_key_sequence(&fd, key_sequence.clone(), KeyValue::Off);
        emitter::emit_key_sequence_toggle(&fd, key_sequence.clone());
        return true;
    }
    return false;
}

fn get_combo_key(key_sequence: Vec<EV_KEY>) -> String {
    return key_sequence
        .iter()
        .map(|k| (k.clone() as u32).to_string())
        .collect::<Vec<String>>()
        .join("-");
}

struct Config {
    pub singles: HashMap<EV_KEY, EV_KEY>,
    pub combos: HashMap<String, Vec<EV_KEY>>
}

struct State {
    pub pressed: VecDeque<EV_KEY>
}

fn main() {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(15));
        std::process::exit(0);
    });

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

    let combo_transform_cfg = [
        [
            [EV_KEY::KEY_LEFTMETA, EV_KEY::KEY_C],
            [EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_C]
        ],
        [
            [EV_KEY::KEY_LEFTMETA, EV_KEY::KEY_V],
            [EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_V]
        ],
        [
            [EV_KEY::KEY_LEFTMETA, EV_KEY::KEY_A],
            [EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_A]
        ],
    ];

    let mut config = Config {
        singles: HashMap::new(),
        combos: HashMap::new()
    };
    for tuple in single_transform_cfg.iter() {
        let from = tuple[0];
        let to = tuple[1];
        config.singles.insert(from, to);
    }

    for combo in combo_transform_cfg.iter() {
        let from_combo = combo[0];
        let to_combo = combo[1];
        let key = get_combo_key(from_combo.to_vec());
        config.combos.insert(key.clone(), to_combo.to_vec());
    }

    let mut state = State { pressed: VecDeque::new() };

    let keyboard_fd_path = "/dev/input/by-path/platform-i8042-serio-0-event-kbd";
    let device_in = kb_in::KeyboardInput::new(keyboard_fd_path);

    let device_out = kb_out::open_device();

    loop {
        let mut event = device_in.next_event();

        // Handle single transformations
        if config.singles.contains_key(&event.code) {
            let key_code = config.singles.get(&event.code).unwrap().clone();
            println!("Transforming {:?}", key_code);
            event.code = key_code;
        }


        // Handle state
        if event.value == KeyValue::Off  {
            state.pressed.pop_front();
        } else if event.value == KeyValue::On {
            state.pressed.push_back(event.code.clone());
        } else {
          // no-op
        }

        // Handle combo transformations
        let mut meta_handled = false;
        if handle_meta_combo(&device_out, &state, &config, &event.code) {
            meta_handled = true;
            break;
        }

        // If we didn't handle meta combo, just pass through the key event
        if !meta_handled {
            let event_code = EventCode::EV_KEY(event.code.clone());
            println!("Emitting {}, {:?}", event_code, event.value);
            emitter::emit_key(&device_out, event.code, event.value);
        }
    }
}

