use std::io::prelude::*;
use std::{collections::HashMap, fs::File};

use gtk::{glib, prelude::*, Application, ApplicationWindow, Button};
use serde::Deserialize;

const APP_ID: &str = "org.smona.keyboard";
const SPACING: i32 = 4;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    // println!("{}", config.keys.get("a").unwrap().classes);
    app.connect_activate(build_ui);
    app.run()
}

#[derive(Deserialize)]
struct Config {
    keys: HashMap<String, KeyConfig>,
    pages: HashMap<String, PageConfig>,
}

#[derive(Deserialize)]
struct KeyConfig {
    classes: Option<String>,
}

#[derive(Deserialize)]
struct PageConfig {
    keys: Vec<Vec<String>>,
}

fn get_config() -> Result<Config, String> {
    let mut file = File::open("config.toml").map_err(|e| "Could not find config file.")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Could not read config file: {}", e.to_string()))?;
    Ok(toml::from_str::<Config>(&contents).unwrap())
    // .map_err(|e| format!("Could not parse config file: {}", e.message()))?
}

fn build_ui(app: &Application) {
    let config = get_config()
        .map_err(|e| format!("Failed to load config: {}", e))
        .unwrap();

    let layout = config
        .pages
        .values()
        .next()
        .expect("You must define at least one page.");

    let container = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
    // container.set_hexpand(true);
    for row in &layout.keys {
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
        container.append(&row_box);
        for key in row {
            let key = key.to_owned();
            let button = Button::builder()
                .label(&key)
                .height_request(80)
                .hexpand(true)
                .build();

            button.connect_clicked(move |_| {
                print!("{}", key);
                std::io::stdout().flush().unwrap();
            });

            row_box.append(&button);
        }
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Smona's Cool Keyboard!")
        .child(&container)
        .build();

    gtk4_layer_shell::init_for_window(&window);
    gtk4_layer_shell::set_layer(&window, gtk4_layer_shell::Layer::Overlay);
    // Push other windows out of the way
    gtk4_layer_shell::auto_exclusive_zone_enable(&window);
    let anchors = [
        (gtk4_layer_shell::Edge::Left, true),
        (gtk4_layer_shell::Edge::Right, true),
        (gtk4_layer_shell::Edge::Top, false),
        (gtk4_layer_shell::Edge::Bottom, true),
    ];

    for (anchor, state) in anchors {
        gtk4_layer_shell::set_anchor(&window, anchor, state);
    }

    window.present();
}
