// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use glib::ToValue;
use gtk::{glib, subclass::prelude::*};
use std::cell::RefCell;

mod imp {
    use super::*;
    use glib::{ParamSpec, ParamSpecString};

    #[derive(Debug)]
    pub struct MetadataItem {
        pub key: RefCell<String>,
        pub value: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MetadataItem {
        const NAME: &'static str = "MetadataItem";
        type Type = super::MetadataItem;
        type ParentType = glib::Object;

        fn new() -> Self {
            Self {
                key: RefCell::new(String::default()),
                value: RefCell::new(String::default()),
            }
        }
    }

    impl ObjectImpl for MetadataItem {
        fn properties() -> &'static [ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("key")
                    .default_value(None)
                    .build(),
                    ParamSpecString::builder("value")
                    .default_value(None)
                    .build()
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "key" => {
                    let key = value.get().unwrap();
                    self.key.replace(key);
                }
                "value" => {
                    let val = value.get().unwrap();
                    self.value.replace(val);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "key" => self.key.borrow().to_value(),
                "value" => self.value.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct MetadataItem(ObjectSubclass<imp::MetadataItem>);
}

impl MetadataItem {
    pub fn new(key: &String, value: &String) -> Self {
        glib::Object::builder()
            .property("key", &key)
            .property("value", &value)
            .build()
    }
}
