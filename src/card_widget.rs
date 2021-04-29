// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::backend::{Card, CardSize, Social};
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/card.ui")]
    pub struct CardWidget {
        #[template_child]
        pub large_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub small_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub textbox: TemplateChild<gtk::Box>,
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub description: TemplateChild<gtk::Label>,
        #[template_child]
        pub site: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CardWidget {
        const NAME: &'static str = "CardWidget";
        type Type = super::CardWidget;
        type ParentType = gtk::Box;

        fn new() -> Self {
            Self {
                large_image: TemplateChild::default(),
                small_image: TemplateChild::default(),
                textbox: TemplateChild::default(),
                title: TemplateChild::default(),
                description: TemplateChild::default(),
                site: TemplateChild::default(),
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

    impl ObjectImpl for CardWidget {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for CardWidget {}
    impl BoxImpl for CardWidget {}
}

glib::wrapper! {
    pub struct CardWidget(ObjectSubclass<imp::CardWidget>)
        @extends gtk::Widget, gtk::Box;
}

impl CardWidget {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create CardWidget")
    }

    pub fn new_from_card(card: &Card) -> Self {
        let new = CardWidget::new();
        new.set_card(&card);
        new
    }

    pub fn set_card(&self, card: &Card) {
        let self_ = imp::CardWidget::from_instance(self);

        // Get Widgets
        let large_image = &*self_.large_image;
        let small_image = &*self_.small_image;
        let textbox = &*self_.textbox;
        let title = &*self_.title;
        let description = &*self_.description;
        let site = &*self_.site;

        title.set_label(&card.title);
        if let Some(text) = &card.description {
            description.set_label(&text); 
        }
        site.set_label(&card.site);
        
        match &card.social {
            Social::Facebook => {
                textbox.reorder_child_after(site, None::<&gtk::Widget>);
                title.set_lines(2);
                title.set_wrap(true);
                if let Some(_) = &card.description {
                    if &card.title.len() <= &60 {
                        description.set_visible(true);
                    }
                }

            }
            Social::Mastodon => {
                            
            }
            Social::Twitter => {
                if let Some(_) = &card.description {
                    description.set_visible(true);
                    description.set_wrap(true);
                }

                match card.size.as_ref().unwrap() {
                    CardSize::Medium => {
                        description.set_lines(3);
                    }
                    CardSize::Large => {
                        description.set_lines(2);
                    }
                    _ => {}
                }
            }
        }
    }
}
