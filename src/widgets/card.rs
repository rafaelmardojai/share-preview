// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use gettextrs::*;
use gtk::{
    CompositeTemplate,
    glib,
    gdk::Texture,
    prelude::*,
    subclass::prelude::*,
};

use crate::backend::{Card, CardError, CardSize, Social};
use super::CardImage;

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
                gettext("Couldn’t find enough data to generate a card for this social media.")
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
        CardImage::static_type();

        // Get card UI resource for social
        let ui_resource = format!(
            "/com/rafaelmardojai/SharePreview/ui/cards/{social}.ui",
            social=&card.social.to_string().to_lowercase()
        );
        let builder = gtk::Builder::from_resource(&ui_resource);

        // Append card for social
        let card_box: gtk::Box = builder.object("card").expect("Couldn't get card");
        self.imp().cardbox.append(&card_box);

        // Apply values to well known objects
        let site: gtk::Label = builder.object("site").expect("Couldn't get UI site");
        site.set_label(&card.site);

        let title: gtk::Label = builder.object("title").expect("Couldn't get UI title");
        title.set_label(&card.title);

        let description: gtk::Label = builder.object("description").expect("Couldn't get UI title");
        if let Some(text) = &card.description {
            description.set_label(text);
        }

        let image: CardImage = builder.object("image").expect("Couldn't get UI image");
        if let Some(img_bytes) = &card.image {
            image.set_image(&img_bytes, &card.size);
        }

        // Tweak card UI
        match &card.social {
            Social::Discourse => {
                if let Some(_) = &card.image {
                    image.set_visible(true);
                }

                if let Some(fav_bytes) = &card.favicon {
                    if let Ok(texture) = Texture::from_bytes(&glib::Bytes::from(fav_bytes)) {
                        let favicon: gtk::Image = builder.object("favicon").expect("Couldn't get UI favicon");
                        favicon.set_from_paintable(Some(&texture));
                        favicon.set_visible(true);
                    }
                }
            },
            Social::Facebook => {
                if let Some(_) = &card.image {
                    image.set_visible(true);
                }

                if let Some(_) = &card.description {
                    if &card.title.chars().count() <= &65 {
                        description.set_visible(true);
                    }
                }

                if let CardSize::Medium = card.size {
                    card_box.set_orientation(gtk::Orientation::Horizontal);
                }
            },
            Social::LinkedIn => {
                if let Some(_) = &card.image {
                    image.set_visible(true);
                }
            },
            Social::Mastodon => {
                if let None = &card.image {
                    image.set_fallback(&card.size);
                }
            },
            Social::Twitter => {
                if let Some(_) = &card.image {
                    image.set_visible(true);
                }

                if let Some(_) = &card.description {
                    description.set_visible(true);
                }
                match card.size {
                    CardSize::Medium => {
                        card_box.set_orientation(gtk::Orientation::Horizontal);
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
