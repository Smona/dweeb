use gtk::{
    glib::{clone, Receiver},
    prelude::*,
};

use relm4::{factory::FactoryVecDeque, ComponentParts, ComponentSender, SimpleComponent};
use tokio::sync::mpsc::UnboundedSender;

use crate::config;

use super::row::Row;

#[derive(Debug)]
pub enum AppInput {
    Shift,
    Open,
    Close,
    KeyPress(String),
}

pub struct AppModel {
    is_open: bool,
    current_layer: &'static str,
    send_key: UnboundedSender<String>,
    rows: FactoryVecDeque<Row>,
}

#[relm4::component(pub)]
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
            set_visible: model.is_open,

            #[local_ref]
            rows_container -> gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: super::SPACING,
            }
        }
    }

    fn init(
        (send_key, recv_from_wl): Self::Init,
        window: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let config = config::get_config()
            .map_err(|e| format!("Failed to load config: {}", e))
            .unwrap();

        let layout = config.layout;
        let page = &config.pages[&layout.default];

        let mut rows = FactoryVecDeque::new(gtk::Box::default(), sender.input_sender());

        for row in &page.keys {
            rows.guard().push_back(
                row.split(' ')
                    .map(|s| s.into())
                    .collect::<Vec<String>>()
                    .into(),
            );
        }

        let model = AppModel {
            current_layer: "default",
            is_open: false,
            send_key,
            rows,
        };

        configure_layer_shell(&window);

        let rows_container = model.rows.widget();
        let widgets = view_output!();

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

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppInput::Close => self.is_open = false,
            AppInput::Open => self.is_open = true,
            AppInput::KeyPress(key) => self.send_key.send(key).unwrap(),
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
