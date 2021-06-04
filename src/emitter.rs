use std::mem;
use std::slice;
use std::{fs::File, io::Write};
use libc::{input_event as raw_event};
use evdev_rs::{InputEvent, TimeVal, enums::{EV_KEY, EV_SYN, EventCode}};

use crate::kb_in::KeyEvent;
use crate::kb_in::KeyValue;

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

pub fn emit_key(fd: &File, key_code: EV_KEY, value: KeyValue) {
    let event_value = match value {
        KeyValue::On => 1,
        KeyValue::Off => 0,
        KeyValue::Hold => 2,
        _ => 0,
    };
    let event_code = EventCode::EV_KEY(key_code);
    emit_event(fd, event_code, event_value)
}

pub fn emit_key_sequence(fd: &File, key_code_list: Vec<EV_KEY>, value: KeyValue) {
    for key_code in &key_code_list {
        emit_key(fd, key_code.clone(), value.clone());
    }
}

pub fn emit_key_sequence_toggle(fd: &File, key_code_list: Vec<EV_KEY>) {
    for key_code in &key_code_list {
        println!("Emitting {:?}, {:?}", key_code.clone(), KeyValue::On);
        emit_key(fd, key_code.clone(), KeyValue::On);
    }
    for key_code in &key_code_list {
        println!("Emitting {:?}, {:?}", key_code.clone(), KeyValue::Off);
        emit_key(fd, key_code.clone(), KeyValue::Off);
    }
}

