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
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub fb_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub fb_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub fb_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub fb_site: TemplateChild<gtk::Label>,
        #[template_child]
        pub ms_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub ms_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub ms_site: TemplateChild<gtk::Label>,
        #[template_child]
        pub tw1_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub tw1_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub tw1_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub tw1_site: TemplateChild<gtk::Label>,
        #[template_child]
        pub tw2_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub tw2_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub tw2_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub tw2_site: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CardWidget {
        const NAME: &'static str = "CardWidget";
        type Type = super::CardWidget;
        type ParentType = gtk::Box;

        fn new() -> Self {
            Self {
                stack: TemplateChild::default(),
                fb_image: TemplateChild::default(),
                fb_title: TemplateChild::default(),
                fb_description: TemplateChild::default(),
                fb_site: TemplateChild::default(),
                ms_image: TemplateChild::default(),
                ms_title: TemplateChild::default(),
                ms_site: TemplateChild::default(),
                tw1_image: TemplateChild::default(),
                tw1_title: TemplateChild::default(),
                tw1_description: TemplateChild::default(),
                tw1_site: TemplateChild::default(),
                tw2_image: TemplateChild::default(),
                tw2_title: TemplateChild::default(),
                tw2_description: TemplateChild::default(),
                tw2_site: TemplateChild::default(),
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
        let stack =  &*self_.stack;
        let fb_image = &*self_.fb_image;
        let fb_title = &*self_.fb_title;
        let fb_description = &*self_.fb_description;
        let fb_site = &*self_.fb_site;
        let ms_image = &*self_.ms_image;
        let ms_title = &*self_.ms_title;
        let ms_site = &*self_.ms_site;
        let tw1_image = &*self_.tw1_image;
        let tw1_title = &*self_.tw1_title;
        let tw1_description = &*self_.tw1_description;
        let tw1_site = &*self_.tw1_site;
        let tw2_image = &*self_.tw2_image;
        let tw2_title = &*self_.tw2_title;
        let tw2_description = &*self_.tw2_description;
        let tw2_site = &*self_.tw2_site;

        match &card.social {
            Social::Facebook => {
                stack.set_visible_child_name("facebook");
                fb_title.set_label(&card.title);
                match &card.description {
                    Some(text) => {
                        if &card.title.len() <= &64 {
                            fb_description.set_label(&text);
                            fb_description.set_visible(true);
                        } else {
                            fb_description.set_visible(false);
                        }                        
                    }
                    None => {
                        fb_description.set_visible(false);
                    }
                }
                fb_site.set_label(&card.site.to_uppercase());
            }
            Social::Mastodon => {
                stack.set_visible_child_name("mastodon");                
                ms_title.set_label(&card.title);
                ms_site.set_label(&card.site);
            }
            Social::Twitter => {
                match card.size.as_ref().unwrap() {
                    CardSize::Medium => {
                        stack.set_visible_child_name("twitter-1");                        
                        tw1_title.set_label(&card.title);
                        match &card.description {
                            Some(text) => {
                                tw1_description.set_label(&text);
                                tw1_description.set_visible(true);                  
                            }
                            None => {
                                tw1_description.set_visible(false);
                            }
                        }
                        tw1_site.set_label(&card.site);
                    }
                    CardSize::Large => {
                        stack.set_visible_child_name("twitter-2");                        
                        tw2_title.set_label(&card.title);
                        match &card.description {
                            Some(text) => {
                                tw2_description.set_label(&text);
                                tw2_description.set_visible(true);                  
                            }
                            None => {
                                tw2_description.set_visible(false);
                            }
                        }
                        tw2_site.set_label(&card.site);
                    }
                    _ => {}
                }
            }
        }
    }
}
