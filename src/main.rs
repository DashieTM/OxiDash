mod utils;
mod window;

use std::process::Command;

use gtk::gio::SimpleAction;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib, Application};
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

fn main() -> glib::ExitCode {
    // Register and include resources
    gio::resources_register_include!("src.templates.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
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

    window.present();
}
