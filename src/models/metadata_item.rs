// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;

use glib::subclass::prelude::*;
use gtk::{
    glib::{self, ParamSpec, Properties, Value},
    prelude::*,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::MetadataItem)]
    pub struct MetadataItem {
        #[property(get, set)]
        pub key: RefCell<String>,
        #[property(get, set)]
        pub value: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MetadataItem {
        const NAME: &'static str = "MetadataItem";
        type Type = super::MetadataItem;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for MetadataItem {
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
