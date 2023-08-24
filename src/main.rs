#![feature(string_remove_matches)]
mod notibox;
mod utils;
mod window;

use dbus::blocking::Connection;
use directories_next as dirs;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use std::{env, fs, thread};
use utils::listener::run;
use utils::NotificationButton;
use window::imp::{check_duplicates, modify_notification, resize_window, show_notification};

use gtk::gdk::Key;
use gtk::gio::SimpleAction;
use gtk::glib::{clone, ExitCode, MainContext};
use gtk::prelude::*;
use gtk::{gio, glib, Application};
use gtk4_layer_shell::Edge;
use window::Window;

const APP_ID: &str = "org.dashie.oxidash";

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum Urgency {
    Low,
    Normal,
    Urgent,
}

impl Urgency {
    fn from_i32(value: i32) -> Result<Urgency, &'static str> {
        match value {
            1 => Ok(Urgency::Low),
            2 => Ok(Urgency::Normal),
            3 => Ok(Urgency::Urgent),
            _ => Err("invalid number, only 1,2 or 3 allowed"),
        }
    }
    fn to_i32(&self) -> i32 {
        match self {
            Urgency::Low => 1,
            Urgency::Normal => 2,
            Urgency::Urgent => 3,
        }
    }
    pub fn to_str(&self) -> &str {
        match self {
            Urgency::Low => "NotificationLow",
            Urgency::Normal => "NotificationNormal",
            Urgency::Urgent => "NotificationUrgent",
        }
    }
}

impl Display for Urgency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_i32())
    }
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct Notification {
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub expire_timeout: i32,
    pub urgency: Urgency,
    pub image_path: String,
    pub progress: i32,
}

impl Notification {
    pub fn create(
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        expire_timeout: i32,
        urgency: i32,
        image_path: String,
        progress: i32,
    ) -> Self {
        Self {
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            expire_timeout,
            urgency: Urgency::from_i32(urgency).unwrap_or_else(|_| Urgency::Low),
            image_path,
            progress,
        }
    }
}

fn get_notifications() -> Vec<Notification> {
    let mut notifications = Vec::new();
    let conn = Connection::new_session().unwrap();
    let proxy = conn.with_proxy(
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        Duration::from_millis(1000),
    );
    let (res,): (
        Vec<(
            String,
            u32,
            String,
            String,
            String,
            Vec<String>,
            i32,
            i32,
            String,
            i32,
        )>,
    ) = proxy
        .method_call("org.freedesktop.Notifications", "GetAllNotifications", ())
        .unwrap_or_else(|_| (Vec::new(),));
    for notification in res {
        notifications.push(Notification::create(
            notification.0,
            notification.1,
            notification.2,
            notification.3,
            notification.4,
            notification.5,
            notification.6,
            notification.7,
            notification.8,
            notification.9,
        ));
    }
    notifications
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

    gio::resources_register_include!("src.templates.gresource")
        .expect("Failed to register resources.");

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(move |_| {
        adw::init().unwrap();
        load_css(&css_string);
    });

    app.connect_activate(build_ui);
    app.run_with_args(&[""])
}

fn build_ui(app: &Application) {
    let (tx, rx) = MainContext::channel(glib::PRIORITY_DEFAULT);
    thread::spawn(move || {
        run(tx);
    });
    let window = Window::new(app);
    window.set_vexpand(true);
    window.set_default_size(300, 100);
    let action_close = SimpleAction::new("close", None);
    let delete_notifications = SimpleAction::new("delete_notifications", None);
    let do_not_disturb = SimpleAction::new("do_not_disturb", None);

    let id_map = Rc::new(RefCell::new(HashMap::<u32, Rc<NotificationButton>>::new()));
    let map_clone = id_map.clone();
    let notifications = get_notifications();
    let windowimp = window.imp();
    for notification in notifications {
        show_notification(&notification, &windowimp, id_map.clone());
    }

    toggle_notification_center();

    delete_notifications.connect_activate(clone!(@weak window => move |_, _| {
        thread::spawn(|| {
            let conn = Connection::new_session().unwrap();
            let proxy = conn.with_proxy(
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                Duration::from_millis(1000),
            );
            let _: Result<(), dbus::Error> =
                proxy.method_call("org.freedesktop.Notifications", "RemoveAllNotifications", ());
        });
        loop {
            let child = window.imp().notibox.first_child();
            if (child).is_none() {
                break;
            }
            window.imp().notibox.remove(&child.unwrap());
        }
    }));

    do_not_disturb.connect_activate(|_, _| {
        thread::spawn(|| {
            let conn = Connection::new_session().unwrap();
            let proxy = conn.with_proxy(
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                Duration::from_millis(1000),
            );
            let _: Result<(), dbus::Error> =
                proxy.method_call("org.freedesktop.Notifications", "DoNotDisturb", ());
        });
    });

    action_close.connect_activate(clone!(@weak window => move |_, _| {
        toggle_notification_center();
        window.close();
    }));

    window.add_action(&action_close);
    window.add_action(&delete_notifications);
    window.add_action(&do_not_disturb);

    gtk4_layer_shell::init_for_window(&window);
    gtk4_layer_shell::set_keyboard_mode(&window, gtk4_layer_shell::KeyboardMode::Exclusive);
    gtk4_layer_shell::auto_exclusive_zone_enable(&window);
    gtk4_layer_shell::set_layer(&window, gtk4_layer_shell::Layer::Overlay);
    gtk4_layer_shell::set_anchor(&window, Edge::Right, true);
    gtk4_layer_shell::set_anchor(&window, Edge::Top, true);

    let windowrc = Rc::new(window.clone());
    let windowrc1 = windowrc.clone();
    let windowrc2 = windowrc.clone();
    let windowrc3 = windowrc.clone();

    let focus_event_controller = gtk::EventControllerFocus::new();
    focus_event_controller.connect_leave(move |_| {
        toggle_notification_center();
        windowrc.close();
    });

    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);

    gesture.connect_pressed(move |_gesture, _, _, _| {
        if !windowrc1.imp().has_pointer.get() {
            toggle_notification_center();
            windowrc1.close();
        }
    });

    let key_event_controller = gtk::EventControllerKey::new();
    key_event_controller.connect_key_pressed(move |_controller, key, _keycode, _state| match key {
        Key::Escape => {
            toggle_notification_center();
            windowrc2.close();
            gtk::Inhibit(true)
        }
        Key::_1 => {
            do_not_disturb.activate(None);
            gtk::Inhibit(true)
        }
        Key::_2 => {
            toggle_notification_center();
            windowrc2.close();
            gtk::Inhibit(true)
        }
        Key::_3 => {
            delete_notifications.activate(None);
            gtk::Inhibit(true)
        }
        _ => gtk::Inhibit(false),
    });

    rx.attach(None, move |notification| {
        if check_duplicates(&notification, map_clone.clone()) {
            println!("trying to modify");
            modify_notification(notification, map_clone.clone());
        } else {
            show_notification(&notification, &windowrc3.imp(), map_clone.clone());
            resize_window(&windowrc3);
        }
        glib::Continue(true)
    });
    window.add_controller(key_event_controller);
    window.add_controller(focus_event_controller);
    window.add_controller(gesture);
    resize_window(&window);
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

fn toggle_notification_center() {
    let conn = Connection::new_session().unwrap();
    let proxy = conn.with_proxy(
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        Duration::from_millis(1000),
    );
    let res: Result<(bool,), dbus::Error> = proxy.method_call(
        "org.freedesktop.Notifications",
        "ToggleNotificationCenter",
        (),
    );
    if res.is_ok() {
        println!("{}", res.unwrap().0);
    }
}
