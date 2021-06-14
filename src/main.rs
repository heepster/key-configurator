use evdev_rs::{enums::{EV_KEY, EventCode, EventType}};
use std::{collections::{HashMap, VecDeque}, fs::{File}, os::unix::process};
use xcb::{Connection, ffi::{XCB_PROPERTY_NOTIFY, xcb_property_notify_event_t}};

use crate::kb_in::KeyValue;

mod emitter;
mod kb_out;
mod kb_in;

fn handle_meta_combo(fd: &File, state: &State, config: &Config, key_code: &EV_KEY) -> bool {
    let current_combo = state.pressed.iter().map(|k| k.clone()).collect::<Vec<EV_KEY>>();
    println!("Current combo {:?}", current_combo);
    let current_combo_key = get_combo_key(current_combo);
    println!("Current combo key {:?}", current_combo_key);

    if config.combos.contains_key(&current_combo_key) {
        let key_sequence = config.combos.get(&current_combo_key).unwrap().clone();
        println!("Emitting key sequence {:?}", key_sequence);
        emitter::emit_key_sequence(&fd, key_sequence.clone(), KeyValue::Off);
        emitter::emit_key_sequence_toggle(&fd, key_sequence.clone());
        return true;
    }
    return false;
}

fn get_combo_key(key_sequence: Vec<EV_KEY>) -> String {
    return key_sequence
        .iter()
        .map(|k| (k.clone() as i32).to_string())
        .collect::<Vec<String>>()
        .join("-");
}

#[derive(Debug)]
struct Config {
    pub singles: HashMap<EV_KEY, EV_KEY>,
    pub combos: HashMap<String, Vec<EV_KEY>>
}

struct State {
    pub pressed: VecDeque<EV_KEY>
}

fn main() {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(30));
        std::process::exit(0);
    });

    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    loop {
        unsafe {
            let focus_cookie = xcb::ffi::xproto::xcb_get_input_focus(conn.get_raw_conn());
            let focus_reply = xcb::ffi::xproto::xcb_get_input_focus_reply(
                conn.get_raw_conn(),
                focus_cookie,
                std::ptr::null_mut(),
            );
            let window = (*focus_reply).focus;
            let attributes_reply = xcb::xproto::get_property(
                &conn,
                false,
                window,
                xcb::xproto::ATOM_WM_CLASS,
                xcb::xproto::ATOM_STRING,
                0,
                16
            ).get_reply().unwrap();
            let mut buf = Vec::new();
            let value: &[u8]  = attributes_reply.value();
            buf.extend_from_slice(value);
            let string_title = String::from_utf8(buf).unwrap();
            println!("{}", string_title);
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    loop {
        unsafe {
            let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
            let setup = xcb::ffi::base::xcb_get_setup(conn.get_raw_conn());
            let screen = xcb::ffi::xproto::xcb_setup_roots_iterator(setup).data;
            println!("{}", (*screen).width_in_pixels);
            let screen_count = xcb::ffi::xproto::xcb_setup_roots_iterator(setup).rem;
            let mut values = [ xcb::ffi::xproto::XCB_EVENT_MASK_PROPERTY_CHANGE ];

            xcb::ffi::xproto::xcb_change_window_attributes(
                 conn.get_raw_conn(),
                 (*screen).root,
                 xcb::ffi::xproto::XCB_CW_EVENT_MASK,
                 values.as_mut_ptr()
            );

            xcb::Connection::flush(&conn);

            println!("waiting for event");
            let event = conn.wait_for_event();
            if event.is_some() {
                let event2 = event.unwrap();
                if (event2.response_type()) == XCB_PROPERTY_NOTIFY {
                    let event3 = xcb::cast_event::<xcb_property_notify_event_t>(&event2);
                    println!("Event Window: {:?}", event3.atom);
                }
            } else {
                println!("Event error");
            }
        }
    }

    let single_transform_cfg = [
        [EV_KEY::KEY_CAPSLOCK,   EV_KEY::KEY_LEFTCTRL],
        [EV_KEY::KEY_F1,         EV_KEY::KEY_PLAYPAUSE],
    ];

    let combo_transform_cfg = [
        [
            [EV_KEY::KEY_LEFTALT, EV_KEY::KEY_C],
            [EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_C]
        ],
        [
            [EV_KEY::KEY_LEFTALT, EV_KEY::KEY_V],
            [EV_KEY::KEY_LEFTCTRL, EV_KEY::KEY_V]
        ],
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

    println!("{:?}", config);

    let mut state = State { pressed: VecDeque::new() };


    //let keyboard_fd_path = "/dev/input/by-path/platform-i8042-serio-0-event-kbd";
    let keyboard_fd_path = "/dev/input/by-path/pci-0000:00:14.0-usb-0:2.2.4.3:1.0-event-kbd";
    //let keyboard_fd_path = "/dev/input/by-path/pci-0000:00:14.0-usb-0:10.4.1:1.0-event-kbd";

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

