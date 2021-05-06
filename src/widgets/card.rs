// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{CardImage};
use crate::backend::{Card, CardSize, Social};
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/card.ui")]
    pub struct CardBox {
        #[template_child]
        pub cardbox: TemplateChild<gtk::Box>,
        #[template_child]
        pub image: TemplateChild<CardImage>,
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
    impl ObjectSubclass for CardBox {
        const NAME: &'static str = "CardBox";
        type Type = super::CardBox;
        type ParentType = gtk::Box;

        fn new() -> Self {
            Self {
                cardbox: TemplateChild::default(),
                image: TemplateChild::default(),
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

    impl ObjectImpl for CardBox {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for CardBox {}
    impl BoxImpl for CardBox {}
}

glib::wrapper! {
    pub struct CardBox(ObjectSubclass<imp::CardBox>)
        @extends gtk::Widget, gtk::Box;
}

impl CardBox {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create CardBox")
    }

    pub fn new_from_card(card: &Card) -> Self {
        let new = CardBox::new();
        new.set_card(&card);
        new
    }

    pub fn set_card(&self, card: &Card) {
        let imp = imp::CardBox::from_instance(self);

        // Put the card values on the widgets
        imp.title.set_label(&card.title);
        if let Some(text) = &card.description {
            imp.description.set_label(&text); 
        }
        imp.site.set_label(&card.site);

        if let Some(img) = &card.image {
            imp.image.set_image(&img, &card.size);
            imp.image.set_visible(true);
        }

        // Change widget aparence by social
        match &card.social {
            Social::Facebook => {
                imp.textbox.reorder_child_after(&*imp.site, None::<&gtk::Widget>);
                imp.title.set_lines(2);
                imp.title.set_wrap(true);
                imp.title.style_context().add_class("title-4");
                if let Some(_) = &card.description {
                    if &card.title.len() <= &60 {
                        imp.description.set_visible(true);
                    }
                }

            }
            Social::Mastodon => {
                imp.cardbox.set_orientation(gtk::Orientation::Horizontal);
                imp.title.style_context().add_class("heading");
            }
            Social::Twitter => {
                imp.title.style_context().add_class("heading");
                if let Some(_) = &card.description {
                    imp.description.set_visible(true);
                    imp.description.set_wrap(true);
                }
                match card.size {
                    CardSize::Medium => {
                        imp.cardbox.set_orientation(gtk::Orientation::Horizontal);
                        imp.description.set_lines(3);
                    }
                    CardSize::Large => {
                        imp.description.set_lines(2);
                    }
                    _ => {}
                }
            }
        }
    }
}
