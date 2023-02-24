// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::backend::{Data};
use super::MetadataItem;

use adw::subclass::prelude::*;
use glib::clone;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/data-dialog.ui")]
    pub struct DataDialog {
        pub model: gio::ListStore,
        pub images_model: gtk::StringList,
        #[template_child]
        pub search: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub images_search: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub url: TemplateChild<gtk::Label>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub images_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub images_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DataDialog {
        const NAME: &'static str = "DataDialog";
        type Type = super::DataDialog;
        type ParentType = adw::Window;

        fn new() -> Self {
            Self {
                model: gio::ListStore::new(MetadataItem::static_type()),
                images_model: gtk::StringList::default(),
                search: TemplateChild::default(),
                images_search: TemplateChild::default(),
                title: TemplateChild::default(),
                url: TemplateChild::default(),
                stack: TemplateChild::default(),
                images_stack: TemplateChild::default(),
                list: TemplateChild::default(),
                images_list: TemplateChild::default(),
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
        let dialog: Self = glib::Object::builder().build();

        dialog.set_metadata(&data);
        dialog.set_images(&data);

        dialog
    }

    pub fn set_metadata(&self, data: &Data) {
        let stack = &*self.imp().stack;

        let site_title = match &data.title {
            Some(title) => title.to_string(),
            None => data.url.to_string()
        };
        self.imp().title.set_label(&site_title);
        self.imp().url.set_label(&data.url);

        // imp.model.remove_all(); // Remove previous model items
        // Add new items from HashMap:
        for (key, val) in data.get_metadata().iter() {
            let item = MetadataItem::new(&key, &val);
            self.imp().model.append(&item);
        }

        // Expressions and filters to get properties from MetadataItem:
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

        // Bind search entry text with MetadataItem properties filters
        self.imp().search.bind_property("text", &key_filter, "search")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self.imp().search.bind_property("text", &value_filter, "search")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();

        // Group filters in one:
        let filter = gtk::AnyFilter::new();
        filter.append(key_filter);
        filter.append(value_filter);

        // Create new filterable model from ListStore and filter:
        let filter_model = gtk::FilterListModel::builder()
            .model(&self.imp().model)
            .filter(&filter)
            .incremental(true)
            .build();

        // Bind model with ListBox
        self.imp().list.bind_model(
            Some(&filter_model),
            clone!(@weak self as self_ =>  @default-panic, move |item| {
                let item = item.downcast_ref::<MetadataItem>().expect("Couldn't get MetadataItem");
                self_.metadata_row(
                    Some(&item.property::<String>("key")),
                    Some(&item.property::<String>("value"))
                )
            })
        );

        // Setup no results view
        filter_model.connect_items_changed(clone!(@weak stack => move |model,_,_,_| {
            let model = model.upcast_ref::<gio::ListModel>();
            if model.n_items() > 0 {
                stack.set_visible_child_name("list");
            } else {
                stack.set_visible_child_name("empty");
            }
        }));
        filter_model.items_changed(0, 0, 0);
    }

    pub fn set_images(&self, data: &Data) {
        let images_stack = &*self.imp().images_stack;

        // Set images into the StringsList
        for image in &data.images {
            self.imp().images_model.append(&image.url);
        }

        // Create filter for the StringsList
        let filter = gtk::StringFilter::new(Some(
            &gtk::PropertyExpression::new(
                gtk::StringObject::static_type(), None::<&gtk::Expression>, "string"
            )
        ));

        let filter_model = gtk::FilterListModel::builder()
            .model(&self.imp().images_model)
            .filter(&filter)
            .incremental(true)
            .build();

        // Bind search entry with filter
        self.imp().images_search.bind_property("text", &filter, "search")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();

        // Bind model with ListBox
        self.imp().images_list.bind_model(
            Some(&filter_model),
            clone!(@weak self as self_ =>  @default-panic, move |item| {
            let item = item.downcast_ref::<gtk::StringObject>().expect("Couldn't get MetadataItem");

            self_.metadata_row(None, Some(&item.string().to_string()))
        }));

        // Setup no results view
        filter_model.connect_items_changed(clone!(@weak images_stack => move |model,_,_,_| {
            let model = model.upcast_ref::<gio::ListModel>();
            if model.n_items() > 0 {
                images_stack.set_visible_child_name("list");
            } else {
                images_stack.set_visible_child_name("empty");
            }
        }));
        filter_model.items_changed(0, 0, 0);
    }

    pub fn metadata_row(&self, key: Option<&String>, value: Option<&String>) -> gtk::Widget {
        let builder = gtk::Builder::from_resource("/com/rafaelmardojai/SharePreview/metadata-item.ui");
        let row: gtk::ListBoxRow = builder.object("row").expect("Couldn't get widget");
        let key_label: gtk::Label = builder.object("key").expect("Couldn't get widget");
        let value_label: gtk::Label = builder.object("value").expect("Couldn't get widget");

        match key {
            Some(s) => key_label.set_label(&s),
            None => key_label.set_visible(false)
        };
        match value {
            Some(s) => value_label.set_label(&s),
            None => value_label.set_visible(false)
        };

        row.upcast::<gtk::Widget>()
    }
}
