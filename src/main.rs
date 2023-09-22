use std::thread;

use gtk::glib;

mod config;
mod ui;
mod wayland;

use relm4::RelmApp;
use tokio::{
    io::unix::AsyncFd,
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
};
use ui::app::AppModel;
use wayland::KeyboardWriter;

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

fn main() -> Result<(), String> {
    let config = config::get_config()?;

    let (send_to_gtk, recv_from_wl) = glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);
    let (send_to_wl, recv_from_gtk) = unbounded_channel::<String>();

    // This has to come before the GUI app is initialized
    thread::spawn(move || run_wayland_thread(recv_from_gtk, send_to_gtk));

    let app = RelmApp::new("org.smona.keyboard");
    app.run::<AppModel>((send_to_wl, recv_from_wl, config));

    Ok(())
}
