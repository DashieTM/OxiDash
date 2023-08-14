mod utils;
mod window;

use directories_next as dirs;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::{env, fs};

use gtk::gdk::Key;
use gtk::gio::SimpleAction;
use gtk::glib::{clone, ExitCode};
use gtk::prelude::*;
use gtk::{gio, glib, Application};
use gtk4_layer_shell::Edge;
use serde_derive::{Deserialize, Serialize};
use window::Window;

const APP_ID: &str = "org.dashie.oxidash";

#[derive(Serialize, Deserialize, Debug, Default)]
struct Notifications {
    data: Vec<Vec<Notification>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Notification {
    body: Body,
    message: NotificationMessage,
    summary: Summary,
    appname: Appname,
    category: Category,
    icon_path: IconPath,
    id: ID,
    timestamp: TimeStamp,
    timeout: Timeout,
    progress: Progress,
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotificationMessage {
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Summary {
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Appname {
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Category {
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DefaultAction {
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct IconPath {
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ID {
    data: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct TimeStamp {
    data: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Timeout {
    data: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Progress {
    data: i32,
}

fn get_notifications() -> Notifications {
    let dunst = Command::new("dunstctl")
        .arg("history")
        .output()
        .expect("dunstctl could not be run")
        .stdout;
    let notifications: Notifications =
        serde_json::from_str(&String::from_utf8(dunst).expect("Could not parse json"))
            .expect("Could not parse json");
    return notifications;
}

fn create_config_dir() -> PathBuf {
    let maybe_config_dir = dirs::ProjectDirs::from("com", "dashie", "oxidash");
    if maybe_config_dir.is_none() {
        panic!("Could not get config directory");
    }
    let config = maybe_config_dir.unwrap();
    let config_dir = config.config_dir();
    if !config_dir.exists() {
        fs::create_dir(config_dir).expect("Could not create config directory");
    }
    let file_path = config_dir.join("style.css");
    if !file_path.exists() {
        fs::File::create(&file_path).expect("Could not create css config file");
        fs::write(
            &file_path,
            "#MainWindow {
                border-radius: 10px;
            }",
        )
        .expect("Could not write default values");
    }
    file_path
}

fn main() -> glib::ExitCode {
    let mut css_string = "".to_string();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let mut argiter = args.iter();
        argiter.next().unwrap();
        match argiter.next().unwrap().as_str() {
            "--css" => {
                let next = argiter.next();
                if next.is_some() {
                    css_string = next.unwrap().clone();
                }
            }
            _ => {
                print!(
                    "usage:
    --css: use a specific path to load a css style sheet.
    --help: show this message.\n"
                );
                return ExitCode::FAILURE;
            }
        }
    } else {
        css_string = create_config_dir().to_str().unwrap().into();
        println!("{css_string}");
    }
    // Register and include resources
    gio::resources_register_include!("src.templates.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(move |_| {
        adw::init().unwrap();
        load_css(&css_string);
    });

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run_with_args(&[""])
}
fn build_ui(app: &Application) {
    // Create new window and present it
    let window = Window::new(app);
    let action_close = SimpleAction::new("close", None);
    let delete_notifications = SimpleAction::new("delete_notifications", None);
    let do_not_disturb = SimpleAction::new("do_not_disturb", None);

    delete_notifications.connect_activate(|_, _| {
        Command::new("dunstctl")
            .arg("history-clear")
            .spawn()
            .expect("Could not use dunstctl");
    });

    do_not_disturb.connect_activate(|_, _| {
        Command::new("dunstctl")
            .arg("set-paused")
            .arg("toggle")
            .spawn()
            .expect("Could not use dunstctl");
    });

    action_close.connect_activate(clone!(@weak window => move |_, _| {
        window.close();
    }));

    window.add_action(&action_close);
    window.add_action(&delete_notifications);
    window.add_action(&do_not_disturb);

    gtk4_layer_shell::init_for_window(&window);
    // gtk4_layer_shell::set_keyboard_interactivity(&window, true);
    gtk4_layer_shell::set_keyboard_mode(&window, gtk4_layer_shell::KeyboardMode::Exclusive);
    gtk4_layer_shell::set_layer(&window, gtk4_layer_shell::Layer::Overlay);
    gtk4_layer_shell::set_anchor(&window, Edge::Right, true);
    gtk4_layer_shell::set_anchor(&window, Edge::Top, true);

    let key_event_controller = gtk::EventControllerKey::new();
    let windowrc = Rc::new(window.clone());
    key_event_controller.connect_key_pressed(move |_controller, key, _keycode, _state| match key {
        Key::Escape => {
            windowrc.close();

            gtk::Inhibit(true)
        }
        _ => gtk::Inhibit(false),
    });

    window.add_controller(key_event_controller);
    window.present();
}

fn load_css(css_string: &String) {
    let context_provider = gtk::CssProvider::new();
    if css_string != "" {
        context_provider.load_from_path(css_string);
    }

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &context_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
