// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{CardImage};
use crate::backend::{Card, CardError, CardSize, Social};
use gettextrs::*;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/card.ui")]
    pub struct CardBox {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
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
        #[template_child]
        pub error_message: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CardBox {
        const NAME: &'static str = "CardBox";
        type Type = super::CardBox;
        type ParentType = gtk::Box;

        fn new() -> Self {
            Self::default()
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CardBox {}
    impl WidgetImpl for CardBox {}
    impl BoxImpl for CardBox {}
}

glib::wrapper! {
    pub struct CardBox(ObjectSubclass<imp::CardBox>)
        @extends gtk::Widget, gtk::Box;
}

impl CardBox {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn new_from_card(card: &Card) -> Self {
        let new = CardBox::new();
        new.set_card(&card);
        new
    }

    pub fn new_from_error(error: &CardError) -> Self {
        let new = CardBox::new();

        let error_text = match error {
            CardError::NotEnoughData => {
                gettext("Couldn't find enough data to generate a card for this social media.")
            },
            CardError::TwitterNoCardFound => {
                gettext("Twitter: No card found.")
            }
        };

        new.imp().error_message.set_label(&error_text);
        new.imp().stack.set_visible_child_name("error");
        new
    }

    pub fn set_card(&self, card: &Card) {
        // Put the card values on the widgets
        self.imp().title.set_label(&card.title);
        if let Some(text) = &card.description {
            self.imp().description.set_label(&text);
        }
        self.imp().site.set_label(&card.site);

        if let Some(img_bytes) = &card.image {
            self.imp().image.set_image(&img_bytes, &card.size);
            self.imp().image.set_visible(true);
        } else {
            match &card.social {
                Social::Mastodon | Social::Twitter => {
                    self.imp().image.set_fallback(&card.size);
                    self.imp().image.set_visible(true);
                }
                _ => ()
            }
        }

        // Change widget aparence by social
        match &card.social {
            Social::Facebook => {
                self.imp().textbox.reorder_child_after(&*self.imp().site, None::<&gtk::Widget>);
                self.imp().title.set_lines(2);
                self.imp().title.set_wrap(true);
                self.imp().title.style_context().add_class("title-4");
                if let Some(_) = &card.description {
                    if &card.title.len() <= &65 {
                        self.imp().description.set_visible(true);
                    }
                }

                if let CardSize::Medium = card.size {
                    self.imp().cardbox.set_orientation(gtk::Orientation::Horizontal);
                }
            }
            Social::Mastodon => {
                self.imp().cardbox.set_orientation(gtk::Orientation::Horizontal);
                self.imp().title.style_context().add_class("heading");
            }
            Social::Twitter => {
                self.imp().title.style_context().add_class("heading");
                if let Some(_) = &card.description {
                    self.imp().description.set_visible(true);
                    self.imp().description.set_wrap(true);
                }
                match card.size {
                    CardSize::Medium => {
                        self.imp().cardbox.set_orientation(gtk::Orientation::Horizontal);
                        self.imp().description.set_lines(3);
                    }
                    CardSize::Large => {
                        self.imp().description.set_lines(2);
                    }
                    _ => {}
                }
            }
        }
    }
}
