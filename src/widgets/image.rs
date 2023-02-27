// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{
    CompositeTemplate,
    gdk::Texture,
    glib,
    prelude::*,
    subclass::prelude::*
};

use crate::backend::CardSize;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/image.ui")]
    pub struct CardImage {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub fallback_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub fallback_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub image: TemplateChild<gtk::Picture>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CardImage {
        const NAME: &'static str = "CardImage";
        type Type = super::CardImage;
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

    pub fn set_image(&self, bytes: &Vec<u8>, size: &CardSize) {
        let (width, height) = size.image_size(); // Get image size

        // Set image widget size
        self.imp().image.set_width_request(width as i32);
        self.imp().image.set_height_request(height as i32);

        match Texture::from_bytes(&glib::Bytes::from(bytes)) {
            Ok(texture) => {
                self.imp().image.set_paintable(Some(&texture));
                self.imp().stack.set_visible_child_name("image");
            },
            Err(_) => {
                self.set_fallback(size);
            }
        }
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
