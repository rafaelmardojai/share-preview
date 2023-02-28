// Copyright 2023 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::convert::TryFrom;

use glib::subclass::prelude::*;
use gtk::{
    glib::{self, ParamSpec, Properties, Value},
    prelude::*,
};

use crate::backend::LogLevel;

mod imp {
    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::LogItem)]
    pub struct LogItem {
        pub level: RefCell<LogLevel>,
        #[property(get, set)]
        text: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LogItem {
        const NAME: &'static str = "LogItem";
        type Type = super::LogItem;
    }

    impl ObjectImpl for LogItem {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub struct LogItem(ObjectSubclass<imp::LogItem>);
}

impl LogItem {
    pub fn new(level: LogLevel, text: &str) -> LogItem {
        let item: LogItem = glib::Object::builder()
            .property("text", text)
            .build();

        *item.imp().level.borrow_mut() = level;

        item
    }

    pub fn level(&self) -> LogLevel {
        *self.imp().level.borrow()
    }
}
