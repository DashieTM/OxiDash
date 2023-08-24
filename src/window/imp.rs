use std::cell::Cell;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::utils::NotificationButton;
use adw::subclass::prelude::AdwApplicationWindowImpl;
use dbus::blocking::Connection;
use glib::subclass::InitializingObject;
use gtk::gdk_pixbuf::{self, Pixbuf};
use gtk::glib::clone;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, Image, Label, PolicyType, ProgressBar, ScrolledWindow};
use gtk::{prelude::*, Box};

use crate::{get_notifications, Notification};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/dashie/oxidash/window.ui")]
pub struct Window {
    #[template_child]
    pub mainbox: TemplateChild<Box>,
    #[template_child]
    pub button: TemplateChild<Button>,
    #[template_child]
    pub exit_button: TemplateChild<Button>,
    #[template_child]
    pub clear_history_button: TemplateChild<Button>,
    #[template_child]
    pub notibox: TemplateChild<Box>,
    #[template_child]
    pub scrolled_window: TemplateChild<ScrolledWindow>,
    notifications: Cell<Vec<Notification>>,
    pub has_pointer: Cell<bool>,
}

impl Window {
    fn delete_specific_notification(&self, button: &NotificationButton) {
        let id = button.imp().notification_id.get();
        thread::spawn(move || {
            let conn = Connection::new_session().unwrap();
            let proxy = conn.with_proxy(
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                Duration::from_millis(1000),
            );
            let _: Result<(), dbus::Error> =
                proxy.method_call("org.freedesktop.Notifications", "CloseNotification", (id,));
        });
        self.notibox.remove(&button.imp().notibox.take());
        self.notibox.remove(button);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "MyGtkAppWindow";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        NotificationButton::ensure_type();
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

pub fn show_notification(notification: &Notification, window: &Window) -> Box {
    let notibox = Box::new(gtk::Orientation::Vertical, 5);
    notibox.set_widget_name("Notification");
    notibox.set_css_classes(&["Notification"]);
    let basebox = Box::new(gtk::Orientation::Horizontal, 5);
    basebox.set_css_classes(&["BaseBox"]);
    basebox.set_halign(gtk::Align::Fill);
    let textbox = Box::new(gtk::Orientation::Vertical, 5);
    textbox.set_hexpand(true);
    textbox.set_valign(gtk::Align::Center);
    textbox.set_halign(gtk::Align::Fill);
    let picbuttonbox = Box::new(gtk::Orientation::Horizontal, 5);
    picbuttonbox.set_css_classes(&["PictureButtonBox"]);
    picbuttonbox.set_size_request(100, 110);
    picbuttonbox.set_halign(gtk::Align::End);
    picbuttonbox.set_hexpand(false);

    if notification.body != "" {
        let text = Label::new(Some(&notification.body));
        text.set_xalign(0.0);
        text.set_wrap(true);
        text.set_halign(gtk::Align::Center);
        textbox.append(&text);
    }
    if notification.summary != "" {
        let summary = Label::new(Some(&notification.summary));
        summary.set_xalign(0.0);
        summary.set_wrap(true);
        summary.set_halign(gtk::Align::Center);
        textbox.append(&summary);
    }
    if notification.app_name != "" {
        let appname = Label::new(Some(&notification.app_name));
        appname.set_xalign(0.0);
        appname.set_wrap(true);
        appname.set_halign(gtk::Align::Center);
        textbox.append(&appname);
    }
    basebox.append(&textbox);

    let image = Image::new();
    image.set_size_request(100, 100);
    if set_image(&notification.image_path, &notification.app_icon, &image) {
        picbuttonbox.append(&image);
    }

    notibox.append(&basebox);
    let progbar = ProgressBar::new();
    if notification.progress > -1 {
        println!("{}", notification.progress);
        progbar.set_fraction(notification.progress as f64 / 100.0);
        notibox.append(&progbar);
    }

    let buttonbox = Box::new(gtk::Orientation::Horizontal, 0);
    buttonbox.set_css_classes(&["CloseNotificationButton"]);
    buttonbox.set_size_request(60, 100);
    buttonbox.set_vexpand(true);
    buttonbox.set_hexpand(false);
    buttonbox.set_valign(gtk::Align::Fill);
    buttonbox.set_halign(gtk::Align::End);
    let button = NotificationButton::new();
    button.set_halign(gtk::Align::End);
    button.set_size_request(50, 50);
    button.imp().notification_id.set(notification.replaces_id);
    button.set_icon_name("small-x-symbolic");
    button.imp().notibox.set(notibox.clone());
    button.connect_clicked(clone!(@weak window => move |button| {
        window.delete_specific_notification(button);
    }));
    button.set_valign(gtk::Align::Center);
    button.set_halign(gtk::Align::Center);
    buttonbox.append(&button);

    picbuttonbox.append(&buttonbox);
    basebox.append(&picbuttonbox);
    window.notibox.append(&notibox);
    notibox
}

pub fn resize_window(window: &crate::Window) {
    if window.height() >= 1000 {
        window.set_height_request(1000);
        window.set_vexpand(false);
        window
            .imp()
            .scrolled_window
            .set_vscrollbar_policy(PolicyType::Always);
    } else {
        window.set_vexpand(true);
        window
            .imp()
            .scrolled_window
            .set_vscrollbar_policy(PolicyType::Never);
    }
    window.queue_resize();
}

impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();
        self.notifications.set(get_notifications());
        self.scrolled_window
            .set_hscrollbar_policy(PolicyType::Never);
        self.scrolled_window
            .set_vscrollbar_policy(PolicyType::Never);

        let motion_event_controller = gtk::EventControllerMotion::new();
        motion_event_controller.connect_enter(clone!(@weak self as window => move |_,_,_| {
            window.has_pointer.set(true);
        }));
        motion_event_controller.connect_leave(clone!(@weak self as window => move |_| {
            window.has_pointer.set(false);
        }));
        let focus_event_controller = gtk::EventControllerMotion::new();
        focus_event_controller.connect_leave(clone!(@weak self as window => move |_| {
            window.exit_button.activate_action("win.close", None).expect("wat");
        }));
        self.mainbox.add_controller(focus_event_controller);
        self.mainbox.add_controller(motion_event_controller);

        let notifications = self.notifications.take();
        // let notifications = notiref.get(0).unwrap();

        for notification in notifications.iter() {
            let notibox = show_notification(notification, &self);
            self.notibox.append(&notibox);
        }

        self.button.connect_clicked(move |button| {
            button
                .activate_action("win.do_not_disturb", None)
                .expect("wat");
        });

        self.exit_button.connect_clicked(move |button| {
            button
                .activate_action("win.close", None)
                .expect("Could not close application");
        });

        self.clear_history_button.connect_clicked(move |button| {
            button
                .activate_action("win.delete_notifications", None)
                .expect("wat");
        });
    }
}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl AdwApplicationWindowImpl for Window {}

impl ApplicationWindowImpl for Window {}

fn set_image(picture: &String, icon: &String, image: &Image) -> bool {
    let mut pixbuf: Option<Pixbuf> = None;
    let resize_pixbuf = |pixbuf: Option<Pixbuf>| {
        pixbuf
            .unwrap()
            .scale_simple(100, 100, gdk_pixbuf::InterpType::Bilinear)
    };
    let use_icon = |mut _pixbuf: Option<Pixbuf>| {
        if Path::new(&icon).is_file() {
            _pixbuf = Some(Pixbuf::from_file(&icon).unwrap());
            _pixbuf = resize_pixbuf(_pixbuf);
            image.set_from_pixbuf(Some(&_pixbuf.unwrap()));
            image.style_context().add_class("picture");
        } else {
            image.set_icon_name(Some(icon.as_str()));
            image.style_context().add_class("image");
            image.set_pixel_size(50);
        }
    };

    if picture != "" {
        if Path::new(&picture).is_file() {
            pixbuf = Some(Pixbuf::from_file(picture).unwrap());
            pixbuf = resize_pixbuf(pixbuf);
            image.set_from_pixbuf(Some(&pixbuf.unwrap()));
            image.style_context().add_class("picture");
            return true;
        } else {
            (use_icon)(pixbuf);
            return true;
        }
    } else if icon != "" {
        (use_icon)(pixbuf);
        return true;
    }
    false
}
