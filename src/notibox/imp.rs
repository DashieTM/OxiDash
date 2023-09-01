use std::cell::{RefCell, Cell};

use gtk::subclass::prelude::*;
use gtk::{glib, Image, Label, ProgressBar};

#[derive(Default)]
pub struct NotiBox {
    pub basebox: RefCell<gtk::Box>,
    pub textbox: RefCell<gtk::Box>,
    pub picbuttonbox: RefCell<gtk::Box>,
    pub progbar: RefCell<ProgressBar>,
    pub has_progbar: Cell<bool>,
    pub body: RefCell<Label>,
    pub has_body: Cell<bool>,
    pub summary: RefCell<Label>,
    pub has_summary: Cell<bool>,
    pub image: RefCell<Image>,
    pub has_image: Cell<bool>,
    pub inline_reply: RefCell<gtk::Entry>,
    pub has_inline_reply: Cell<bool>,
    pub body_image: RefCell<Image>,
    pub has_body_image: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for NotiBox {
    const NAME: &'static str = "NotiBox";
    type Type = super::NotiBox;
    type ParentType = gtk::Box;
}

impl ObjectImpl for NotiBox {}

impl WidgetImpl for NotiBox {}

impl BoxImpl for NotiBox {}
