use std::io::prelude::*;
use wayland_client::{
    protocol::{
        wl_registry,
        wl_seat::{self, WlSeat},
    },
    Connection, Dispatch, EventQueue, QueueHandle,
};
use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_manager_v2::{self, ZwpInputMethodManagerV2},
    zwp_input_method_v2::{self, ZwpInputMethodV2},
};

#[derive(Clone)]
pub struct KeyboardWriter {
    seat: Option<WlSeat>,
    input_manager: Option<ZwpInputMethodManagerV2>,
    pub input_method: Option<ZwpInputMethodV2>,
    input_active: bool,
}
impl KeyboardWriter {
    pub fn new(queue: &mut EventQueue<KeyboardWriter>) -> KeyboardWriter {
        let mut state = KeyboardWriter {
            seat: None,
            input_manager: None,
            input_method: None,
            input_active: false,
        };
        queue.roundtrip(&mut state).unwrap();
        queue.roundtrip(&mut state).unwrap();
        state
    }

    pub fn is_active(&self) -> bool {
        self.input_active
    }

    pub fn send_key(&self, key: String) {
        print!("{}", key);
        std::io::stdout().flush().unwrap();
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for KeyboardWriter {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<KeyboardWriter>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("[{}] {} v{}", name, interface, version);
            if interface == "wl_seat" {
                registry.bind::<WlSeat, (), KeyboardWriter>(name, version, qh, ());
            }
            if interface == "zwp_input_method_manager_v2" {
                println!("Binding input method");
                state.input_manager = Some(
                    registry.bind::<ZwpInputMethodManagerV2, (), KeyboardWriter>(
                        name,
                        version,
                        qh,
                        (),
                    ),
                );
            }
        }
    }
}

impl Dispatch<WlSeat, ()> for KeyboardWriter {
    fn event(
        state: &mut Self,
        seat: &WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<KeyboardWriter>,
    ) {
        if let wl_seat::Event::Name { name } = event {
            state.seat = Some(seat.to_owned());
            println!("Found seat: {}", name);
            state.input_method = Some(state.input_manager.as_ref().unwrap().get_input_method(
                seat,
                qh,
                (),
            ));
        }
    }
}

impl Dispatch<ZwpInputMethodManagerV2, ()> for KeyboardWriter {
    fn event(
        state: &mut Self,
        manager: &ZwpInputMethodManagerV2,
        event: zwp_input_method_manager_v2::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<KeyboardWriter>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodV2, ()> for KeyboardWriter {
    fn event(
        state: &mut Self,
        input_method: &ZwpInputMethodV2,
        event: zwp_input_method_v2::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<KeyboardWriter>,
    ) {
        match event {
            zwp_input_method_v2::Event::Activate => {
                state.input_active = true;
            }
            zwp_input_method_v2::Event::Deactivate => {
                state.input_active = false;
            }
            _ => {}
        }
    }
}
