use gtk::prelude::*;
use relm4::prelude::*;

use crate::config::KeyConfig;

use super::{app::Layer, row::RowInput};

pub struct Key {
    config: KeyConfig,
    layer: Layer,
    classes: Vec<&'static str>,
}

#[derive(Debug)]
pub enum KeyOutput {
    KeyPress(String),
}

#[derive(Debug, Clone)]
pub enum KeyInput {
    Shift(Layer),
    KeyPress,
}

#[relm4::factory(pub)]
impl FactoryComponent for Key {
    type Init = KeyConfig;
    type Input = KeyInput;
    type Output = KeyOutput;
    type CommandOutput = ();
    type ParentInput = RowInput;
    type ParentWidget = gtk::Box;

    view! {
        gtk::Button {
            #[watch]
            set_label: self.character(),
            set_height_request: 80,
            set_hexpand: true,
            #[watch]
            set_css_classes: &self.classes,
            connect_clicked => KeyInput::KeyPress,
        }
    }

    fn init_model(config: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            config,
            classes: Vec::new(),
            // classes: config.classes.unwrap_or(Vec::new()),
            layer: Layer::Normal,
        }
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            KeyInput::KeyPress => sender.output(KeyOutput::KeyPress(self.character().clone())),
            KeyInput::Shift(layer) => {
                self.layer = layer;
                if self.config.char == "<shift>" && self.layer == Layer::Locked {
                    self.classes.push("suggested-action");
                } else {
                    self.classes.clear();
                }
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<RowInput> {
        match output {
            KeyOutput::KeyPress(key) => Some(RowInput::KeyPress(key)),
        }
    }
}

impl Key {
    fn character(&self) -> &String {
        match (&self.layer, &self.config.upper) {
            (Layer::Locked | Layer::Shifted, Some(c)) => &c,
            _ => &self.config.char,
        }
    }
}
