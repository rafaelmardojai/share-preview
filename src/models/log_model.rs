// Copyright 2023 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;

use gio::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

use im_rc::Vector;

use crate::backend::{Log, LogLevel};
use super::LogItem;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LogListModel(pub(super) RefCell<Vector<LogItem>>);

    #[glib::object_subclass]
    impl ObjectSubclass for LogListModel {
        const NAME: &'static str = "LogListModel";
        type Type = super::LogListModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for LogListModel {}

    impl ListModelImpl for LogListModel {
        fn item_type(&self) -> glib::Type {
            LogItem::static_type()
        }

        fn n_items(&self) -> u32 {
            self.0.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.0
                .borrow()
                .get(position as usize)
                .map(|o| o.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct LogListModel(ObjectSubclass<imp::LogListModel>) @implements gio::ListModel;
}

impl LogListModel {
    pub fn new() -> LogListModel {
        glib::Object::new()
    }

    pub fn append(&self, obj: &LogItem) {
        let index = {
            let mut data = self.imp().0.borrow_mut();
            data.push_back(obj.clone());
            data.len() - 1
        };

        // Emit change signal
        self.items_changed(index as u32, 0, 1);
    }

    pub fn clear(&self) {
        let length: u32 = self.imp().n_items();

        // Clear inner vector
        self.imp().0.borrow_mut().clear();
        // Emit change signal
        self.items_changed(0, length, 0);
    }

    /// Get the count of the worrying levels in the model
    pub fn worrying_count(&self) -> (u32, u32, u32) {
        let mut inf_count: u32 = 0;
        let mut war_count: u32 = 0;
        let mut err_count: u32 = 0;

        for item in self.imp().0.borrow().iter() {

            match item.level() {
                LogLevel::Info => {
                    inf_count += 1;
                },
                LogLevel::Warning => {
                    war_count += 1;
                },
                LogLevel::Error => {
                    err_count += 1;
                },
                _ => ()
            }
        }

        (inf_count, war_count, err_count)
    }
}

impl Log for LogListModel {
    fn log(&self, level: LogLevel, text: String) {
        let item = LogItem::new(level, &text);
        self.append(&item);
    }

    fn flush(&self) {
        self.clear();
    }
}
