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
        let self_ = imp::CardBox::from_instance(self);

        // Get Widgets
        let cardbox = &*self_.cardbox;
        let image = &*self_.image;
        let textbox = &*self_.textbox;
        let title = &*self_.title;
        let description = &*self_.description;
        let site = &*self_.site;

        title.set_label(&card.title);
        if let Some(text) = &card.description {
            description.set_label(&text); 
        }
        site.set_label(&card.site);

        if let Some(img) = &card.image {
            //let img = img.clone();
            image.set_image(&img);
            image.set_visible(true);                     
        }
        
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
                cardbox.set_orientation(gtk::Orientation::Horizontal);
            }
            Social::Twitter => {
                if let Some(_) = &card.description {
                    description.set_visible(true);
                    description.set_wrap(true);
                }

                match card.size.as_ref().unwrap() {
                    CardSize::Medium => {
                        cardbox.set_orientation(gtk::Orientation::Horizontal);
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
