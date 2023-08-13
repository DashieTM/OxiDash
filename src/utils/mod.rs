mod notificationbutton;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct NotificationButton(ObjectSubclass<notificationbutton::NotificationButton>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl NotificationButton {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

impl Default for NotificationButton {
    fn default() -> Self {
        Self::new()
    }
}
