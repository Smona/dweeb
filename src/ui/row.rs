use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, prelude::*};

use crate::config::KeyConfig;

use super::{
    app::{AppInput, Layer},
    key::{Key, KeyInput},
};

pub struct Row {
    buttons: FactoryVecDeque<Key>,
    spacing: i32,
}

#[derive(Debug)]
pub enum RowOutput {
    KeyPress(String),
}

#[derive(Debug, Clone)]
pub enum RowInput {
    KeyPress(String),
    Shift(Layer),
}

#[relm4::factory(pub)]
impl FactoryComponent for Row {
    type Init = (Vec<(KeyConfig, Layer)>, i32);
    type Input = RowInput;
    type Output = RowOutput;
    type CommandOutput = ();
    type ParentInput = AppInput;
    type ParentWidget = gtk::Box;

    view! {
        self.buttons.widget().clone() -> gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: self.spacing,
        }
    }

    fn init_model(
        (keys, spacing): Self::Init,
        _index: &DynamicIndex,
        sender: FactorySender<Self>,
    ) -> Self {
        let mut buttons = FactoryVecDeque::new(gtk::Box::default(), sender.input_sender());
        for key in &keys {
            buttons.guard().push_back(key.clone());
        }
        let model = Self { buttons, spacing };
        model
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            RowInput::KeyPress(key) => sender.output(RowOutput::KeyPress(key)),
            RowInput::Shift(shifted) => self.buttons.broadcast(KeyInput::Shift(shifted)),
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<AppInput> {
        match output {
            RowOutput::KeyPress(key) => Some(AppInput::KeyPress(key)),
        }
    }
}
