use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, prelude::*};

use super::{app::AppInput, key::Key};

pub struct Row {
    keys: Vec<String>,
    buttons: FactoryVecDeque<Key>,
}

#[derive(Debug)]
pub enum RowOutput {
    KeyPress(String),
}

#[derive(Debug)]
pub enum RowInput {
    KeyPress(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for Row {
    type Init = Vec<String>;
    type Input = RowInput;
    type Output = RowOutput;
    type CommandOutput = ();
    type ParentInput = AppInput;
    type ParentWidget = gtk::Box;

    view! {
        self.buttons.widget().clone() -> gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: super::SPACING,
        }
    }

    fn init_model(keys: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let mut buttons = FactoryVecDeque::new(gtk::Box::default(), sender.input_sender());
        for key in &keys {
            buttons.guard().push_back(key.clone());
        }
        let model = Self { keys, buttons };
        model
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            RowInput::KeyPress(key) => sender.output(RowOutput::KeyPress(key)),
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<AppInput> {
        match output {
            RowOutput::KeyPress(key) => Some(AppInput::KeyPress(key)),
        }
    }
}
