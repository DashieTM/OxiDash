use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::glib;
use gtk::subclass::prelude::*;

use crate::notibox::NotiBox;

#[derive(Default)]
pub struct NotificationButton {
    pub notibox: RefCell<Rc<NotiBox>>,
    pub notification_id: Cell<u32>,
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
