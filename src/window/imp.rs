use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::notibox::NotiBox;
use crate::utils::NotificationButton;
use adw::subclass::prelude::AdwApplicationWindowImpl;
use dbus::blocking::Connection;
use glib::subclass::InitializingObject;
use gtk::gdk_pixbuf::{self, Pixbuf};
use gtk::glib::clone;
use gtk::subclass::prelude::*;
use gtk::{
    glib, Button, CompositeTemplate, Entry, Image, Label, PolicyType, ProgressBar, ScrolledWindow,
};
use gtk::{prelude::*, Box};

use crate::{ImageData, Notification};

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
    pub has_pointer: Cell<bool>,
}

impl Window {
    fn delete_specific_notification(
        &self,
        button: &NotificationButton,
        id_map: Rc<RefCell<HashMap<u32, Rc<NotificationButton>>>>,
    ) {
        let id = button.imp().notification_id.get();
        let mut map = id_map.borrow_mut();
        map.remove(&id);
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
        self.notibox.remove(&*button.imp().notibox.take());
        self.notibox.remove(button);
        if map.is_empty() {
            self.scrolled_window.hide();
        }
    }
    fn delete_specific_notification_with_id(
        &self,
        id: u32,
        id_map: Rc<RefCell<HashMap<u32, Rc<NotificationButton>>>>,
    ) {
        let mut map = id_map.borrow_mut();
        let button = map.get(&id);
        if button.is_none() {
            return;
        }
        let button = button.unwrap();
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
        self.notibox.remove(&*button.imp().notibox.take());
        self.notibox.remove(&*button.clone());
        map.remove(&id);
        if map.is_empty() {
            self.scrolled_window.hide();
        }
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

pub fn modify_notification(
    notification: Notification,
    id_map: Rc<RefCell<HashMap<u32, Rc<NotificationButton>>>>,
) {
    let id = notification.replaces_id;
    let map = id_map.borrow_mut();
    let notibutton = map.get(&id).unwrap().imp();
    let notibox_borrow = notibutton.notibox.borrow_mut();
    let notibox = notibox_borrow.imp();
    let basebox = notibox.basebox.borrow_mut();
    let textbox = notibox.textbox.borrow_mut();
    let picbuttonbox = notibox.picbuttonbox.borrow_mut();

    let exists = notibox.has_progbar.get();
    if notification.progress < 0 && exists {
        basebox.remove(&notibox.progbar.take());
        notibox.has_progbar.set(false);
    } else if notification.progress > 0 {
        let mut progbar = notibox.progbar.borrow_mut();
        if !exists {
            let newprog = ProgressBar::new();
            *progbar = newprog;
            basebox.append(&*progbar);
            notibox.has_progbar.set(true);
        }
        progbar.set_fraction(notification.progress as f64 / 100.0);
    }

    let exists = notibox.has_summary.get();
    if notification.summary == "" && exists {
        textbox.remove(&notibox.summary.take());
        notibox.has_summary.set(false);
    } else if notification.summary != "" {
        let (text, css_classes) = class_from_html(notification.summary);
        let mut text_borrow = notibox.summary.borrow_mut();
        if !exists {
            *text_borrow = Label::new(None);
            textbox.append(&*text_borrow);
            notibox.has_summary.set(true);
        }
        text_borrow.set_text(text.as_str());
        text_borrow.set_css_classes(&[&"summary", &css_classes]);
    }

    let exists = notibox.has_body.get();
    if notification.body == "" && exists {
        textbox.remove(&notibox.body.take());
        notibox.has_body.set(false);
    } else if notification.body != "" {
        let (text, css_classes) = class_from_html(notification.body);
        let mut text_borrow = notibox.body.borrow_mut();
        if !exists {
            *text_borrow = Label::new(None);
            textbox.append(&*text_borrow);
            notibox.has_body.set(true);
        }
        text_borrow.set_text(text.as_str());
        text_borrow.set_css_classes(&[&"text", &css_classes]);
    }

    let exists = notibox.has_image.get();
    if notification.image_path == "" && notification.app_icon == "" && exists {
        picbuttonbox.remove(&notibox.image.take());
        notibox.has_image.set(false);
    } else {
        let mut image_borrow = notibox.image.borrow_mut();
        if !exists {
            let img = Image::new();
            *image_borrow = img;
            picbuttonbox.append(&*image_borrow);
            notibox.has_image.set(true);
        }
        set_image(
            &notification.image_data,
            &notification.image_path,
            &notification.app_icon,
            &image_borrow,
        );
    }
}

pub fn show_notification(
    notification: &Notification,
    window: &Window,
    id_map: Rc<RefCell<HashMap<u32, Rc<NotificationButton>>>>,
) {
    let notibox = Rc::new(NotiBox::new(gtk::Orientation::Vertical, 5));
    let notiimp = notibox.imp();
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

    let mut has_inline_reply = false;
    for action in notification.actions.iter() {
        if action == "inline-reply" {
            has_inline_reply = true;
        }
    }

    if notification.body != "" {
        notiimp.has_body.set(true);
        let (textstr, css_classes) = class_from_html(notification.body.clone());
        let text = Label::new(Some(&textstr));
        text.set_css_classes(&["text", &css_classes]);
        text.set_xalign(0.0);
        text.set_wrap(true);
        text.set_halign(gtk::Align::Center);
        let mut shared_text = notiimp.body.borrow_mut();
        *shared_text = text;
        textbox.append(&*shared_text);
    }
    if notification.summary != "" {
        notiimp.has_summary.set(true);
        let (textstr, css_classes) = class_from_html(notification.summary.clone());
        let summary = Label::new(Some(&textstr));
        summary.set_css_classes(&["summary", &css_classes]);
        summary.set_xalign(0.0);
        summary.set_wrap(true);
        summary.set_halign(gtk::Align::Center);
        let mut shared_summary = notiimp.summary.borrow_mut();
        *shared_summary = summary;
        textbox.append(&*shared_summary);
    }
    if notification.app_name != "" {
        let (textstr, css_classes) = class_from_html(notification.app_name.clone());
        let appname = Label::new(Some(&textstr));
        appname.set_css_classes(&["app_name", &css_classes]);
        appname.set_xalign(0.0);
        appname.set_wrap(true);
        appname.set_halign(gtk::Align::Center);
        textbox.append(&appname);
    }
    basebox.append(&textbox);

    let image = Image::new();
    image.set_size_request(100, 100);
    notiimp.has_image.set(set_image(
        &notification.image_data,
        &notification.image_path,
        &notification.app_icon,
        &image,
    ));
    let mut shared_image = notiimp.image.borrow_mut();
    *shared_image = image;
    picbuttonbox.append(&*shared_image);

    notibox.append(&basebox);
    let progbar = ProgressBar::new();
    if notification.progress > -1 {
        notiimp.has_progbar.set(true);
        progbar.set_fraction(notification.progress as f64 / 100.0);
        let mut shared_progbar = notiimp.progbar.borrow_mut();
        *shared_progbar = progbar;
        notibox.append(&*shared_progbar);
    }

    let inline_reply = Entry::new();
    if has_inline_reply {
        let id = notification.replaces_id;
        inline_reply.connect_activate(clone!(@weak window, @weak id_map => move |entry| {
        let text = entry.text().to_string();
            thread::spawn(move || {
                let conn = Connection::new_session().unwrap();
                let proxy = conn.with_proxy(
                    "org.freedesktop.Notifications",
                    "/org/freedesktop/Notifications",
                    Duration::from_millis(1000),
                );
                let _: Result<(), dbus::Error> = proxy.method_call(
                    "org.freedesktop.Notifications",
                    "InlineReply",
                    (id, text),
                );
            });
            window.delete_specific_notification_with_id(id, id_map.clone());
        }));
        notibox.append(&inline_reply);
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
    button.connect_clicked(clone!(@weak id_map, @weak window => move |button| {
        window.delete_specific_notification(button, id_map);
    }));
    button.set_valign(gtk::Align::Center);
    button.set_halign(gtk::Align::Center);

    picbuttonbox.append(&buttonbox);
    basebox.append(&picbuttonbox);
    window.notibox.append(&*notibox);
    let notibutton = Rc::new(button);
    let mut notibox_borrow = notibutton.imp().notibox.borrow_mut();
    *notibox_borrow = notibox.clone();
    buttonbox.append(&*notibutton);
    if !window.scrolled_window.is_visible() {
        window.scrolled_window.show();
    }
    id_map
        .borrow_mut()
        .insert(notification.replaces_id, notibutton.clone());
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

pub fn check_duplicates(
    notification: &Notification,
    id_map: Rc<RefCell<HashMap<u32, Rc<NotificationButton>>>>,
) -> bool {
    let borrow = id_map.borrow_mut();
    let opt = borrow.get(&notification.replaces_id);
    if opt.is_none() {
        return false;
    }
    true
}

impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();
        self.scrolled_window
            .set_hscrollbar_policy(PolicyType::Never);
        self.scrolled_window
            .set_vscrollbar_policy(PolicyType::Never);
        self.scrolled_window.hide();

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

fn class_from_html(mut body: String) -> (String, String) {
    let mut open = false;
    let mut ret: &str = "";
    for char in body.chars() {
        if char == '<' && !open {
            open = true;
        } else if open {
            ret = match char {
                'b' => "bold",
                'i' => "italic",
                'u' => "underline",
                'h' => "hyprlink",
                _ => {
                    ret = "";
                    break;
                }
            };
            break;
        }
    }
    body.remove_matches("<b>");
    body.remove_matches("</b>");
    body.remove_matches("<i>");
    body.remove_matches("</i>");
    body.remove_matches("<a href=\">");
    body.remove_matches("</a>");
    body.remove_matches("<u>");
    body.remove_matches("</u>");
    (body, String::from(ret))
}

fn set_image(data: &ImageData, picture: &String, icon: &String, image: &Image) -> bool {
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
    } else if data.width != -1 {
        let bytes = gtk::glib::Bytes::from(&data.data);
        pixbuf = Some(Pixbuf::from_bytes(
            &bytes,
            gdk_pixbuf::Colorspace::Rgb,
            data.has_alpha,
            data.bits_per_sample,
            data.width,
            data.height,
            data.rowstride,
        ));
        pixbuf = resize_pixbuf(pixbuf);
        image.set_from_pixbuf(Some(&pixbuf.unwrap()));
        image.style_context().add_class("picture");
        return true;
    }
    println!("{}", data.width);
    false
}
