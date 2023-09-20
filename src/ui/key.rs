use gtk::prelude::*;
use relm4::prelude::*;

use super::app::AppInput;

pub struct Key {
    character: String,
}

#[derive(Debug)]
pub enum KeyOutput {
    KeyPress(String),
}

#[derive(Debug)]
pub enum KeyInput {
    KeyPress,
}

#[relm4::factory(pub)]
impl FactoryComponent for Key {
    type Init = String;
    type Input = KeyInput;
    type Output = KeyOutput;
    type CommandOutput = ();
    type ParentInput = AppInput;
    type ParentWidget = gtk::Box;

    view! {
        gtk::Button {
            #[watch]
            set_label: &self.character,
            set_height_request: 80,
            set_hexpand: true,
            connect_clicked => KeyInput::KeyPress,
        }
    }

    fn init_model(
        character: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { character }
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            KeyInput::KeyPress => sender.output(KeyOutput::KeyPress(self.character.clone())),
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<AppInput> {
        match output {
            KeyOutput::KeyPress(key) => Some(AppInput::KeyPress(key)),
        }
    }
}
