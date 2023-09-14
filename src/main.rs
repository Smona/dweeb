use std::thread;
use std::time::Duration;

use gtk::glib::timeout_add_local;
use gtk::{glib, prelude::*, Application, ApplicationWindow, Button};

mod config;
mod wayland;

use wayland::KeyboardWriter;
use wayland_protocols_misc::zwp_input_method_v2::client::zwp_input_method_v2::ZwpInputMethodV2;

const APP_ID: &str = "org.smona.keyboard";
const SPACING: i32 = 4;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        let (tx, rx) = glib::MainContext::channel(glib::source::Priority::DEFAULT);

        let conn = wayland_client::Connection::connect_to_env()
            .map_err(|_| "Could not connect to wayland socket.")
            .unwrap();
        let wl_display = conn.display();
        let mut event_queue = conn.new_event_queue();
        let _registry = wl_display.get_registry(&event_queue.handle(), ());
        let mut writer = KeyboardWriter::new(&mut event_queue);
        let method = writer.input_method.as_ref().unwrap().clone();
        let mut was_active = false;

        let inner_app = app.clone();
        // timeout_add_local(Duration::from_millis(300), move || {
        //     event_queue.roundtrip(&mut writer).unwrap();
        //     let is_active = writer.is_active();
        //     if was_active != is_active {
        //         if is_active {
        //             build_ui(&inner_app, &method);
        //         } else {
        //             inner_app.active_window().unwrap().set_visible(false);
        //         }
        //         was_active = is_active;
        //     }
        //     glib::ControlFlow::Continue
        // });
        thread::spawn(move || {
            let mut was_active = false;
            loop {
                event_queue.blocking_dispatch(&mut writer).unwrap();
                let is_active = writer.is_active();
                if was_active != is_active {
                    tx.send(is_active).unwrap();
                    was_active = is_active;
                }
            }
        });
        let inner_app = app.clone();
        rx.attach(None, move |should_be_open| {
            if should_be_open {
                build_ui(&inner_app, &method);
            } else {
                inner_app.active_window().unwrap().set_visible(false);
            }
            glib::ControlFlow::Continue
        });
    });

    // Prevent the app from exiting when the window is hidden
    let _hold = app.hold();

    app.run()
}

fn build_ui(app: &Application, kb: &ZwpInputMethodV2) {
    let config = config::get_config()
        .map_err(|e| format!("Failed to load config: {}", e))
        .unwrap();

    let layout = config
        .pages
        .values()
        .next()
        .expect("You must define at least one page.");

    let container = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
    for row in &layout.keys {
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
        container.append(&row_box);
        for key in row {
            let kb = kb.clone();
            let key = key.to_owned();
            let button = Button::builder()
                .label(&key)
                .height_request(80)
                .hexpand(true)
                .build();

            button.connect_clicked(move |_| {
                kb.commit_string(key.to_owned());
                kb.commit(1)
            });

            row_box.append(&button);
        }
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Smona's Cool Keyboard!")
        .child(&container)
        .build();

    gtk4_layer_shell::init_for_window(&window);
    gtk4_layer_shell::set_layer(&window, gtk4_layer_shell::Layer::Overlay);
    // Push other windows out of the way
    gtk4_layer_shell::auto_exclusive_zone_enable(&window);
    let anchors = [
        (gtk4_layer_shell::Edge::Left, true),
        (gtk4_layer_shell::Edge::Right, true),
        (gtk4_layer_shell::Edge::Top, false),
        (gtk4_layer_shell::Edge::Bottom, true),
    ];

    for (anchor, state) in anchors {
        gtk4_layer_shell::set_anchor(&window, anchor, state);
    }

    window.present();
}
