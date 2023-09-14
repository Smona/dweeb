use wayland_client::{
    protocol::{
        wl_keyboard::{self, WlKeyboard},
        wl_registry,
        wl_seat::{self, WlSeat},
    },
    Connection, Dispatch, DispatchError, EventQueue, QueueHandle,
};
use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_manager_v2::{self, ZwpInputMethodManagerV2},
    zwp_input_method_v2::{self, ZwpInputMethodV2},
};

pub struct WaylandBackend {
    event_queue: EventQueue<AppData>,
    state: AppData,
}

impl WaylandBackend {
    pub fn new() -> Result<WaylandBackend, String> {
        let conn = wayland_client::Connection::connect_to_env()
            .map_err(|_| "Could not connect to wayland socket.")?;
        let wl_display = conn.display();
        let mut event_queue = conn.new_event_queue();
        let _registry = wl_display.get_registry(&event_queue.handle(), ());
        let mut state = AppData {
            seat: None,
            input_manager: None,
            input_active: false,
        };
        event_queue.roundtrip(&mut state).unwrap();
        Ok(WaylandBackend { event_queue, state })
    }

    pub fn tick(&mut self) -> Result<usize, DispatchError> {
        self.event_queue.blocking_dispatch(&mut self.state)
    }

    pub fn is_active(&self) -> bool {
        self.state.input_active
    }
}

struct AppData {
    seat: Option<WlSeat>,
    input_manager: Option<ZwpInputMethodManagerV2>,
    input_active: bool,
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("[{}] {} v{}", name, interface, version);
            if interface == "wl_seat" {
                registry.bind::<WlSeat, (), AppData>(name, version, qh, ());
            }
            if interface == "zwp_input_method_manager_v2" {
                println!("Binding input method");
                state.input_manager = Some(registry.bind::<ZwpInputMethodManagerV2, (), AppData>(
                    name,
                    version,
                    qh,
                    (),
                ));
            }
        }
    }
}

impl Dispatch<WlSeat, ()> for AppData {
    fn event(
        state: &mut Self,
        seat: &WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_seat::Event::Name { name } = event {
            state.seat = Some(seat.to_owned());
            println!("Found seat: {}", name);
            state
                .input_manager
                .as_ref()
                .unwrap()
                .get_input_method(seat, qh, ());
        }
    }
}

impl Dispatch<ZwpInputMethodManagerV2, ()> for AppData {
    fn event(
        state: &mut Self,
        manager: &ZwpInputMethodManagerV2,
        event: zwp_input_method_manager_v2::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodV2, ()> for AppData {
    fn event(
        state: &mut Self,
        input_method: &ZwpInputMethodV2,
        event: zwp_input_method_v2::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        match event {
            zwp_input_method_v2::Event::Activate => {
                println!("Input method activated");
                state.input_active = true;
            }
            zwp_input_method_v2::Event::Deactivate => {
                println!("Input method deactivated");
                state.input_active = false;
            }
            _ => {}
        }
    }
}
