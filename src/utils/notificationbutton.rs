use std::cell::Cell;

use gtk::{glib, Box};
use gtk::subclass::prelude::*;


// Object holding the state
#[derive(Default)]
pub struct NotificationButton{
    pub notibox: Cell<Box>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for NotificationButton {
    const NAME: &'static str = "NotificationButton";
    type Type = super::NotificationButton;
    type ParentType = gtk::Button;
}

// Trait shared by all GObjects
impl ObjectImpl for NotificationButton {}

// Trait shared by all widgets
impl WidgetImpl for NotificationButton {}

// Trait shared by all buttons
impl ButtonImpl for NotificationButton {}
