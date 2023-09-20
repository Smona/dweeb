use gtk::{
    glib::{clone, Receiver},
    prelude::*,
};
use relm4::{component, ComponentParts, ComponentSender, SimpleComponent};
use tokio::sync::mpsc::UnboundedSender;

use crate::config;

const SPACING: i32 = 4;

#[derive(Debug)]
pub enum AppInput {
    Shift,
    Open,
    Close,
}

pub struct AppModel {
    is_open: bool,
    current_layer: &'static str,
    send_key: UnboundedSender<String>,
}

#[component(pub)]
impl SimpleComponent for AppModel {
    type Input = AppInput;
    type Output = ();
    type Init = (UnboundedSender<String>, Receiver<bool>);

    view! {
        gtk::Window {
            set_title: Some("dweeb"),
            set_default_width: 300,
            set_default_height: 200,
            #[watch]
            set_visible: model.is_open
        }
    }

    fn init(
        (send_key, recv_from_wl): Self::Init,
        window: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = AppModel {
            current_layer: "default",
            is_open: false,
            send_key,
        };

        configure_layer_shell(&window);

        let widgets = view_output!();

        let config = config::get_config()
            .map_err(|e| format!("Failed to load config: {}", e))
            .unwrap();

        let layout = config.layout;
        let page = &config.pages[&layout.default];

        recv_from_wl.attach(
            None,
            clone!(@strong window, @strong sender => move |should_be_open| {
                if should_be_open {
                    sender.input(AppInput::Open);
                } else {
                    sender.input(AppInput::Close);
                }
                Continue(true)
            }),
        );

        let container = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
        for row in &page.keys {
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
            container.append(&row_box);
            for key in row.split(' ') {
                let kb = model.send_key.clone();
                let key = key.to_owned();
                let button = gtk::Button::builder()
                    .label(&key)
                    .height_request(80)
                    .hexpand(true)
                    .build();

                button.connect_clicked(move |_| {
                    eprintln!("Sending key");
                    kb.send(key.to_owned()).unwrap();
                });

                row_box.append(&button);
            }
        }

        window.set_child(Some(&container));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppInput::Close => self.is_open = false,
            AppInput::Open => self.is_open = true,
            AppInput::Shift => {}
        }
    }
}

fn configure_layer_shell(window: &gtk::Window) {
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
