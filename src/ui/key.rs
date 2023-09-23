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
    type Init = (KeyConfig, Layer);
    type Input = KeyInput;
    type Output = KeyOutput;
    type CommandOutput = ();
    type ParentInput = RowInput;
    type ParentWidget = gtk::Box;

    view! {
        gtk::Button {
            #[watch]
            set_label?: match &self.config.icon {
                Some(_) => None,
                None => self.config.label.as_ref().or(Some(self.character())),
            },
            set_icon_name?: self.config.icon.as_ref(),
            set_height_request: 80,
            set_hexpand: true,
            #[watch]
            set_css_classes: &self.classes,
            connect_clicked => KeyInput::KeyPress,
        }
    }

    fn init_model(
        (config, layer): Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        let mut model = Self {
            config,
            classes: Vec::new(),
            // classes: config.classes.unwrap_or(Vec::new()),
            layer,
        };
        model.update_classes();
        model
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            KeyInput::KeyPress => sender.output(KeyOutput::KeyPress(self.character().clone())),
            KeyInput::Shift(layer) => {
                self.layer = layer;
                self.update_classes()
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
    fn update_classes(&mut self) {
        self.classes.clear();
        if self.config.char == "<shift>" && self.layer == Layer::Locked {
            self.classes.push("suggested-action");
        }
    }

    fn character(&self) -> &String {
        match (&self.layer, &self.config.upper) {
            (Layer::Locked | Layer::Shifted, Some(c)) => &c,
            _ => &self.config.char,
        }
    }
}
