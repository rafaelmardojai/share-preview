// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::backend::{CardSize, Image, ImageError};
use image;
use gettextrs::*;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{glib, CompositeTemplate};
use gtk_macros::spawn;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/image.ui")]
    pub struct CardImage {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub fallback_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub fallback_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub error_message: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CardImage {
        const NAME: &'static str = "CardImage";
        type Type = super::CardImage;
        type ParentType = gtk::Box;

        fn new() -> Self {
            Self {
                stack: TemplateChild::default(),
                spinner: TemplateChild::default(),
                fallback_box: TemplateChild::default(),
                fallback_icon: TemplateChild::default(),
                image: TemplateChild::default(),
                error_message: TemplateChild::default(),
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

    impl ObjectImpl for CardImage {}
    impl WidgetImpl for CardImage {}
    impl BoxImpl for CardImage {}
}

glib::wrapper! {
    pub struct CardImage(ObjectSubclass<imp::CardImage>)
        @extends gtk::Widget, gtk::Box;
}

impl CardImage {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_image(&self, img: &Image, size: &CardSize) {
        // Get Widgets
        let stack = self.imp().stack.clone();
        let spinner = self.imp().spinner.clone();
        let image = self.imp().image.clone();
        let error_message = self.imp().error_message.clone();

        let (width, height) = size.image_size(); // Get image size

        // Set image widget size
        image.set_width_request(width as i32);
        image.set_height_request(height as i32);

        spinner.start();

        // Fetch image and set it to the widget
        let img = img.clone();
        spawn!(async move {
            match img.fetch(width, height).await {
                Ok(texture) => {
                    image.set_paintable(Some(&texture));
                    stack.set_visible_child_name("image");
                }
                Err(err) => {
                    let error_text = match err {
                        ImageError::FetchError(_) => {
                            gettext("Failure when receiving image from the peer.")
                        },
                        ImageError::ImageError(error) => {
                            match error {
                                image::error::ImageError::Unsupported(_) => {
                                    gettext("Invalid image format.")
                                }
                                _ => gettext("The image couldn't be loaded.")
                            }
                        },
                        ImageError::TextureError | ImageError::Unexpected => {
                            gettext("Unexpected image error.")
                        }
                    };
                    error_message.set_label(&error_text);
                    stack.set_visible_child_name("error");
                    spinner.stop();
                }
            }
        });
    }

    pub fn set_fallback(&self, size: &CardSize) {
        let (width, height) = size.image_size(); // Get image size

        // Set box widget size
        self.imp().fallback_box.set_width_request(width as i32);
        self.imp().fallback_box.set_height_request(height as i32);

        self.imp().fallback_icon.set_pixel_size(size.icon_size());

        self.imp().stack.set_visible_child_name("fallback");
    }
}
