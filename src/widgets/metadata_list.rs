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
        let imp = imp::MetadataList::from_instance(self);

        imp.model.remove_all(); // Remove previous model items
        // Add new items from HashMap:
        for (key, val) in items.iter() {
            let item = MetadataItem::new(&key, &val);
            imp.model.append(&item);
        }

        // Expresions and filters to get propeties from MetadataItem:
        let key_filter = gtk::StringFilter::new(Some(
            &gtk::PropertyExpression::new(
                MetadataItem::static_type(), None::<&gtk::Expression>, "key"
            )
        ));
        let value_filter = gtk::StringFilter::new(Some(
            &gtk::PropertyExpression::new(
                MetadataItem::static_type(), None::<&gtk::Expression>, "value"
            )
        ));

        // Group filters in one:
        let filter = gtk::AnyFilter::new();
        filter.append(&key_filter);
        filter.append(&value_filter);

        // Create new filterable model from ListStore and filter:
        let filter_model = gtk::FilterListModel::new(Some(&imp.model), Some(&filter));
        filter_model.set_incremental(true);

        // Bind search entry text with MetadataItem propeties filters
        imp.search.bind_property("text", &key_filter, "search")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        imp.search.bind_property("text", &value_filter, "search")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();

        // Items factory for ListView from UI resource
        let factory = gtk::BuilderListItemFactory::from_resource(
            None::<&gtk::BuilderScope>, "/com/rafaelmardojai/SharePreview/metadata-item.ui"
        );
        let selection_model = gtk::NoSelection::new(Some(&filter_model)); // Selection model

        // Set factory and model to ListView
        imp.list.set_factory(Some(&factory));
        imp.list.set_model(Some(&selection_model));
    }
}
