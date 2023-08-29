// Copyright 2023 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::subclass::prelude::*;
use gettextrs::*;
use gtk::{
    CompositeTemplate,
    gio,
    glib,
    prelude::*,
};

use crate::backend::LogLevel;
use crate::models::LogItem;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/log-dialog.ui")]
    pub struct LogDialog {
        #[template_child]
        pub list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LogDialog {
        const NAME: &'static str = "LogDialog";
        type Type = super::LogDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LogDialog {}
    impl WidgetImpl for LogDialog {}
    impl WindowImpl for LogDialog {}
    impl AdwWindowImpl for LogDialog {}
}

glib::wrapper! {
    pub struct LogDialog(ObjectSubclass<imp::LogDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

#[gtk::template_callbacks]
impl LogDialog {
    pub fn new(model: &impl IsA<gio::ListModel>) -> Self {
        let dialog: LogDialog = glib::Object::builder().build();
        let imp = dialog.imp();

        let size = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);

        imp.list.bind_model(
            Some(model),
            move |item| {
                let item = item.downcast_ref::<LogItem>().expect("Couldn't get LogItem");

                let container = gtk::Box::builder()
                    .spacing(12)
                    .css_classes(["log-item"])
                    .build();

                let level = gtk::Label::builder()
                    .valign(gtk::Align::Start)
                    .xalign(0.0)
                    .build();
                container.append(&level);
                size.add_widget(&level);

                let label = gtk::Label::builder()
                    .label(item.property::<String>("text"))
                    .wrap(true)
                    .hexpand(true)
                    .xalign(0.0)
                    .selectable(true)
                    .build();
                container.append(&label);

                match item.level() {
                    LogLevel::Debug => {
                        container.add_css_class("debug");
                        container.add_css_class("dim-label");
                        level.set_label(&gettext("DEBUG"));
                        level.add_css_class("caption");
                        label.add_css_class("caption");
                    },
                    LogLevel::Info => {
                        container.add_css_class("accent");
                        level.set_label(&gettext("INFO"));
                    },
                    LogLevel::Warning => {
                        container.add_css_class("warning");
                        level.set_label(&gettext("WARNING"));
                    },
                    LogLevel::Error => {
                        container.add_css_class("error");
                        level.set_label(&gettext("ERROR"));
                    },
                }

                gtk::ListBoxRow::builder()
                    .child(&container)
                    .build()
                    .upcast::<gtk::Widget>()
            }
        );

        dialog
    }
}
