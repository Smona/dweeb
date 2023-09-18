use std::thread;

use gtk::{glib, prelude::*, Application, ApplicationWindow, Button};

mod config;
mod wayland;

use tokio::{
    io::unix::AsyncFd,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use wayland::KeyboardWriter;

const APP_ID: &str = "org.smona.keyboard";
const SPACING: i32 = 4;

#[tokio::main(flavor = "current_thread")]
async fn run_wayland_thread(
    mut recv_from_gtk: UnboundedReceiver<String>,
    send_to_gtk: glib::Sender<bool>,
) {
    let conn = wayland_client::Connection::connect_to_env()
        .map_err(|_| "Could not connect to wayland socket.")
        .unwrap();
    let wl_display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let _registry = wl_display.get_registry(&event_queue.handle(), ());
    let mut writer = KeyboardWriter::new(&mut event_queue);

    let mut was_active = writer.is_active();
    // Send initial value, since active state is edge-detected
    send_to_gtk.send(was_active).unwrap();

    loop {
        // This would be required if other threads were reading from the socket.
        // event_queue.dispatch_pending(&mut state).unwrap();
        let read_guard = event_queue.prepare_read().unwrap();
        let fd = read_guard.connection_fd();
        let async_fd = AsyncFd::new(fd).unwrap();

        tokio::select! {
            keymsg = recv_from_gtk.recv() => {
                match keymsg {
                    Some(key) => writer.send_key(key),
                    None => {
                        // Receiver is dead -- all senders are dropped.
                    }
                }
            },
            async_guard = async_fd.readable() => {
                async_guard.unwrap().clear_ready();
                // Drop the async_fd since it's holding a reference to the read_guard,
                // which is dropped on read. We don't need to read from it anyways.
                std::mem::drop(async_fd);
                // This should not block because we already ensured readiness
                let event = read_guard.read();
                match event {
                    // There are events but another thread processed them, we don't need to dispatch
                    Ok(0) => {}
                    // We have some events
                    Ok(_) => {
                        event_queue.dispatch_pending(&mut writer).unwrap();
                    }
                    // No events to receive
                    Err(_) => {} // Err(e) => eprintln!("{}", e),
                }

                let is_active = writer.is_active();
                if was_active != is_active {
                    send_to_gtk.send(is_active).unwrap();
                    was_active = is_active;
                }
            },
        }

        // Send any new messages to the socket.
        event_queue.flush().unwrap();
    }
}

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        let (send_to_gtk, recv_from_wl) =
            glib::MainContext::channel(glib::source::Priority::DEFAULT);
        let (send_to_wl, recv_from_gtk) = unbounded_channel::<String>();

        let inner_app = app.clone();
        let inner_sender = send_to_wl.clone();
        recv_from_wl.attach(None, move |should_be_open| {
            if should_be_open {
                build_ui(&inner_app, &inner_sender);
            } else if let Some(active_window) = inner_app.active_window() {
                active_window.set_visible(false);
            }
            glib::ControlFlow::Continue
        });

        thread::spawn(move || run_wayland_thread(recv_from_gtk, send_to_gtk));
    });

    // Prevent the app from exiting when the window is hidden
    let _hold = app.hold();

    app.run()
}

fn build_ui(app: &Application, send_key: &UnboundedSender<String>) {
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
        for key in row.split(' ') {
            let kb = send_key.clone();
            let key = key.to_owned();
            let button = Button::builder()
                .label(&key)
                .height_request(80)
                .hexpand(true)
                .build();

            button.connect_clicked(move |_| {
                kb.send(key.to_owned()).unwrap();
            });

            row_box.append(&button);
        }
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("dweeb")
        .child(&container)
        .build();

    configure_layer_shell(&window);

    window.present();
}

fn configure_layer_shell(window: &ApplicationWindow) {
    gtk4_layer_shell::init_for_window(window);
    gtk4_layer_shell::set_layer(window, gtk4_layer_shell::Layer::Overlay);
    // Push other windows out of the way
    gtk4_layer_shell::auto_exclusive_zone_enable(window);
    let anchors = [
        (gtk4_layer_shell::Edge::Left, true),
        (gtk4_layer_shell::Edge::Right, true),
        (gtk4_layer_shell::Edge::Top, false),
        (gtk4_layer_shell::Edge::Bottom, true),
    ];

    for (anchor, state) in anchors {
        gtk4_layer_shell::set_anchor(window, anchor, state);
    }
}
