// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use super::MetadataItem;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate};
use std::collections::HashMap;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/metadata.ui")]
    pub struct MetadataList {
        pub model: gio::ListStore,
        #[template_child]
        pub search: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub list: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MetadataList {
        const NAME: &'static str = "MetadataList";
        type Type = super::MetadataList;
        type ParentType = gtk::Box;

        fn new() -> Self {
            Self {
                model: gio::ListStore::new(MetadataItem::static_type()),
                search: TemplateChild::default(),
                list: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MetadataList {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for MetadataList {}
    impl BoxImpl for MetadataList {}
}

glib::wrapper! {
    pub struct MetadataList(ObjectSubclass<imp::MetadataList>)
        @extends gtk::Widget, gtk::Box;
}

impl MetadataList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MetadataList")
    }

    pub fn set_items(&self, items: &HashMap<String, String>) {
        let self_ = imp::MetadataList::from_instance(self);

        let search = &*self_.search;
        let list = &*self_.list;

        self_.model.remove_all();
        for (key, val) in items.iter() {
            let item = MetadataItem::new(&key, &val);
            self_.model.append(&item);
        }

        let expression = gtk::PropertyExpression::new(
            MetadataItem::static_type(),
            None::<&gtk::Expression>,
            "key"
        );
        let filter = gtk::StringFilter::new(Some(&expression));
        let filter_model = gtk::FilterListModel::new(Some(&self_.model), Some(&filter));
        filter_model.set_incremental(true);

        search.bind_property("text", &filter, "search")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();

        let factory = gtk::BuilderListItemFactory::from_resource(
            None::<&gtk::BuilderScope>, "/com/rafaelmardojai/SharePreview/metadata-item.ui"
        );
        let selection_model = gtk::NoSelection::new(Some(&filter_model));

        list.set_factory(Some(&factory));
        list.set_model(Some(&selection_model));
    }
}
