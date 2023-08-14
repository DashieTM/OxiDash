use std::cell::Cell;

use gtk::subclass::prelude::*;
use gtk::{glib, Box};

#[derive(Default)]
pub struct NotificationButton {
    pub notibox: Cell<Box>,
    pub notification_id: Cell<i32>,
}

#[glib::object_subclass]
impl ObjectSubclass for NotificationButton {
    const NAME: &'static str = "NotificationButton";
    type Type = super::NotificationButton;
    type ParentType = gtk::Button;
}

impl ObjectImpl for NotificationButton {}

impl WidgetImpl for NotificationButton {}

impl ButtonImpl for NotificationButton {}
