use std::cell::Cell;
use std::thread;
use std::time::Duration;

use crate::utils::NotificationButton;
use adw::subclass::prelude::AdwApplicationWindowImpl;
use dbus::blocking::Connection;
use glib::subclass::InitializingObject;
use gtk::glib::clone;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, Label, Picture, PolicyType, ScrolledWindow};
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
                "org.freedesktop.Notifications2",
                "/org/freedesktop/Notifications2",
                Duration::from_millis(1000),
            );
            let _: Result<(), dbus::Error> = proxy.method_call(
                "org.freedesktop.Notifications2",
                "RemoveNotification",
                (id,),
            );
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

impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();
        self.notifications.set(get_notifications());
        self.scrolled_window
            .set_hscrollbar_policy(PolicyType::Never);

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
            let notibox = Box::new(gtk::Orientation::Horizontal, 5);
            notibox.set_widget_name("Notification");
            notibox.set_css_classes(&["Notification"]);
            let textbox = Box::new(gtk::Orientation::Vertical, 5);
            textbox.set_width_request(380);
            let picbuttonbox = Box::new(gtk::Orientation::Vertical, 5);

            let text = Label::new(Some(&notification.body));
            text.set_xalign(0.0);
            text.set_wrap(true);
            let summary = Label::new(Some(&notification.summary));
            summary.set_xalign(0.0);
            summary.set_wrap(true);
            let appname = Label::new(Some(&notification.app_name));
            appname.set_xalign(0.0);
            appname.set_wrap(true);

            let picture = Picture::new();
            picture.set_filename(notification.app_icon.clone().into());

            picbuttonbox.append(&picture);
            textbox.append(&appname);
            textbox.append(&summary);
            textbox.append(&text);

            self.notibox.append(&notibox);
            let button = NotificationButton::new();
            button.imp().notification_id.set(notification.replaces_id);
            button.set_icon_name("small-x-symbolic");
            button.imp().notibox.set(notibox.clone());
            button.connect_clicked(clone!(@weak self as window => move |button| {
                window.delete_specific_notification(button);
            }));

            picbuttonbox.append(&button);
            notibox.append(&textbox);
            notibox.append(&picbuttonbox);
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
