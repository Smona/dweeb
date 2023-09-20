use gtk::prelude::*;
use relm4::prelude::*;

use crate::config::KeyConfig;

use super::row::RowInput;

pub struct Key {
    config: KeyConfig,
    shifted: bool,
}

#[derive(Debug)]
pub enum KeyOutput {
    KeyPress(String),
}

#[derive(Debug, Clone)]
pub enum KeyInput {
    Shift(bool),
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
            connect_clicked => KeyInput::KeyPress,
        }
    }

    fn init_model(config: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            config,
            shifted: false,
        }
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            KeyInput::KeyPress => sender.output(KeyOutput::KeyPress(self.character().clone())),
            KeyInput::Shift(shifted) => self.shifted = shifted,
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
        match (self.shifted, &self.config.upper) {
            (true, Some(c)) => &c,
            _ => &self.config.char,
        }
    }
}
