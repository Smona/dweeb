use std::time::{Duration, Instant};

use dirs::config_dir;
use gtk::{
    gdk::Display,
    glib::{clone, Receiver},
    prelude::*,
    CssProvider,
};

use relm4::{factory::FactoryVecDeque, ComponentParts, ComponentSender, SimpleComponent};
use tokio::sync::mpsc::UnboundedSender;

use crate::config::{self, KeyConfig};

use super::row::{Row, RowInput};

#[derive(Debug)]
pub enum AppInput {
    Open,
    Close,
    KeyPress(String),
}

/// Represents the keyboard's shift/capslock state
#[derive(PartialEq, Debug, Clone)]
pub enum Layer {
    Normal,
    Shifted,
    Locked,
}

pub struct AppModel {
    is_open: bool,
    current_page: &'static str,
    current_layer: Layer,
    last_layer_change: Instant,
    send_key: UnboundedSender<String>,
    rows: FactoryVecDeque<Row>,
    config: config::Config,
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
            set_default_height: 100,
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

        let rows = FactoryVecDeque::new(gtk::Box::default(), sender.input_sender());

        AppModel::load_css();
        let mut model = AppModel {
            current_page: "uninitialized",
            current_layer: Layer::Normal,
            last_layer_change: Instant::now(),
            is_open: false,
            send_key,
            rows,
            config,
        };

        model.set_page("default");

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

            AppInput::KeyPress(key) => match key.as_str() {
                "<shift>" => self.set_layer(match self.current_layer {
                    Layer::Normal => Layer::Shifted,
                    Layer::Shifted => {
                        if self.last_layer_change.elapsed() < Duration::from_millis(500) {
                            Layer::Locked
                        } else {
                            Layer::Normal
                        }
                    }
                    Layer::Locked => Layer::Normal,
                }),
                "<symbols>" => self.set_page("symbols"),
                "<default>" => self.set_page("default"),
                key => {
                    self.send_key.send(key.to_string()).unwrap();
                    if self.current_layer == Layer::Shifted {
                        self.set_layer(Layer::Normal);
                    }
                }
            },
        }
    }
}

impl AppModel {
    fn load_css() {
        // Load the CSS file and add it to the provider
        let provider = CssProvider::new();
        provider.load_from_data(include_str!("style.css"));

        if let Some(path) = config_dir() {
            if let Ok(user_css) = std::fs::read_to_string(path.join("dweeb/style.css")) {
                provider.load_from_data(&user_css);
            }
        }

        // Add the provider to the default screen
        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn set_page(&mut self, page: &'static str) {
        if page == self.current_page {
            return;
        }
        self.current_page = page;

        let layout_name = &self.config.layout;
        let layout = &self.config.layouts[layout_name];
        let page = &self.config.pages[&layout[self.current_page]];

        let mut rows = self.rows.guard();
        rows.clear();
        for row in &page.keys {
            let foo = row
                .split(' ')
                .map(|s| {
                    (
                        match self.config.keys.get(s) {
                            Some(config) => config.clone(),
                            // Provide a default config for simple keys
                            None => KeyConfig::new(s),
                        },
                        self.current_layer.clone(),
                    )
                })
                .collect();
            rows.push_back(foo);
        }
    }

    fn set_layer(&mut self, layer: Layer) {
        self.current_layer = layer;
        self.last_layer_change = Instant::now();

        self.rows
            .guard()
            .broadcast(RowInput::Shift(self.current_layer.clone()))
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
