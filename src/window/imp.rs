use std::cell::Cell;

use crate::utils::NotificationButton;
use adw::subclass::prelude::AdwApplicationWindowImpl;
use glib::subclass::InitializingObject;
use gtk::glib::clone;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, Label, Overflow, Picture};
use gtk::{prelude::*, Box, Text};

use crate::{get_notifications, Notifications};

// ANCHOR: object
// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/dashie/oxidash/window.ui")]
pub struct Window {
    #[template_child]
    pub button: TemplateChild<Button>,
    #[template_child]
    pub exit_button: TemplateChild<Button>,
    #[template_child]
    pub clear_history_button: TemplateChild<Button>,
    #[template_child]
    pub notibox: TemplateChild<Box>,
    notifications: Cell<Notifications>,
}
// ANCHOR_END: object

#[gtk::template_callbacks]
impl Window {
    #[template_callback]
    fn delete_notifications(&self, _: Button) {
        loop {
            let child = self.notibox.first_child();
            if (child).is_none() {
                break;
            }
            self.notibox.remove(&child.unwrap());
        }
    }
    fn delete_specific_notification(&self, button: &NotificationButton) {
        self.notibox.remove(&button.imp().notibox.take());
        self.notibox.remove(button);
    }
}

// ANCHOR: subclass
// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "MyGtkAppWindow";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        NotificationButton::ensure_type();
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}
// ANCHOR_END: subclass

// ANCHOR: object_impl
// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();
        self.notifications.set(get_notifications());

        let notiref = self.notifications.take();
        let notifications = notiref.data.get(0).unwrap();

        for notification in notifications.iter() {
            let notibox = Box::new(gtk::Orientation::Horizontal, 5);
            let textbox = Box::new(gtk::Orientation::Vertical, 5);
            textbox.set_width_request(380);
            let picbuttonbox = Box::new(gtk::Orientation::Vertical, 5);

            let text = Label::new(Some(&notification.body.data));
            text.set_xalign(0.0);
            text.set_wrap(true);
            let summary = Label::new(Some(&notification.summary.data));
            summary.set_xalign(0.0);
            summary.set_wrap(true);
            let appname = Label::new(Some(&notification.appname.data));
            appname.set_xalign(0.0);
            appname.set_wrap(true);

            let picture = Picture::new();
            picture.set_filename(notification.icon_path.data.clone().into());

            picbuttonbox.append(&picture);
            textbox.append(&appname);
            textbox.append(&summary);
            textbox.append(&text);

            self.notibox.append(&notibox);
            let button = NotificationButton::new();
            button.imp().notibox.set(notibox.clone());
            button.connect_clicked(clone!(@weak self as window => move |button| {
                window.delete_specific_notification(button);
            }));

            picbuttonbox.append(&button);
            notibox.append(&textbox);
            notibox.append(&picbuttonbox);
            self.notibox.append(&notibox);
        }

        // Connect to "clicked" signal of `button`
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
// ANCHOR_END: object_impl

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {}

// Trait shared by all application windows
impl AdwApplicationWindowImpl for Window {}

impl ApplicationWindowImpl for Window {}
