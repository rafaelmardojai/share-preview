// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::backend::{Data};
use super::MetadataItem;

use adw::subclass::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/data-dialog.ui")]
    pub struct DataDialog {
        pub model: gio::ListStore,
        #[template_child]
        pub search_bar: TemplateChild<gtk::SearchBar>,
        #[template_child]
        pub search: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub list: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DataDialog {
        const NAME: &'static str = "DataDialog";
        type Type = super::DataDialog;
        type ParentType = adw::Window;

        fn new() -> Self {
            Self {
                model: gio::ListStore::new(MetadataItem::static_type()),
                search_bar: TemplateChild::default(),
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

    impl ObjectImpl for DataDialog {}
    impl WidgetImpl for DataDialog {}
    impl WindowImpl for DataDialog {}
    impl AdwWindowImpl for DataDialog {}
}

glib::wrapper! {
    pub struct DataDialog(ObjectSubclass<imp::DataDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl DataDialog {
    pub fn new(data: &Data) -> Self {
        let dialog: Self = glib::Object::new(&[]).expect("Failed to create DataDialog");

        dialog.setup();
        dialog.set_data(&data);

        dialog
    }

    pub fn setup(&self) {
        let imp = imp::DataDialog::from_instance(self);

        imp.search_bar.set_key_capture_widget(Some(self));
    }

    pub fn set_data(&self, data: &Data) {
        let imp = imp::DataDialog::from_instance(self);

        let site_title = match &data.title {
            Some(title) => title.to_string(),
            None => data.url.to_string()
        };
        //imp.title.set_subtitle(Some(&site_title));
        //imp.title.set_tooltip_text(Some(&site_title));

        // imp.model.remove_all(); // Remove previous model items
        // Add new items from HashMap:
        for (key, val) in data.get_metadata().iter() {
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
