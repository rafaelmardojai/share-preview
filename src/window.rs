use crate::application::SharePreviewApplication;
use crate::backend::{scrape, Data, Error, Social};
use crate::config::{APP_ID, PROFILE};
use crate::widgets::{CardBox, MetadataList};
use gettextrs::*;
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate, EntryIconPosition};
use gtk_macros::{action, spawn};
use libadwaita::subclass::prelude::*;
use url::Url;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/window.ui")]
    pub struct SharePreviewWindow {
        pub settings: gio::Settings,
        pub card: RefCell<Option<CardBox>>,
        pub data: RefCell<Data>,
        pub active_url: RefCell<String>,
        #[template_child]
        pub dark_theme: TemplateChild<gtk::Button>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub metadata: TemplateChild<MetadataList>,
        #[template_child]
        pub social: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub url_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub url_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub url_error: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub error_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub error_message: TemplateChild<gtk::Label>,
        #[template_child]
        pub cardbox: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SharePreviewWindow {
        const NAME: &'static str = "SharePreviewWindow";
        type Type = super::SharePreviewWindow;
        type ParentType = libadwaita::ApplicationWindow;

        fn new() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),
                card: RefCell::new(Option::default()),
                data: RefCell::new(Data::default()),
                active_url: RefCell::new(String::default()),
                dark_theme: TemplateChild::default(),
                main_stack: TemplateChild::default(),
                metadata: TemplateChild::default(),
                social: TemplateChild::default(),
                url_box: TemplateChild::default(),
                url_entry: TemplateChild::default(),
                url_error: TemplateChild::default(),
                stack: TemplateChild::default(),
                spinner: TemplateChild::default(),
                error_title: TemplateChild::default(),
                error_message: TemplateChild::default(),
                cardbox: TemplateChild::default(),
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

    impl ObjectImpl for SharePreviewWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let builder =
                gtk::Builder::from_resource("/com/rafaelmardojai/SharePreview/shortcuts.ui");
            let shortcuts = builder.object("shortcuts").unwrap();
            obj.set_help_overlay(Some(&shortcuts));

            // Devel Profile
            if PROFILE == "Devel" {
                obj.style_context().add_class("devel");
            }
        }
    }

    impl WidgetImpl for SharePreviewWindow {}
    impl WindowImpl for SharePreviewWindow {}
    impl ApplicationWindowImpl for SharePreviewWindow {}
    impl AdwApplicationWindowImpl for SharePreviewWindow {}
}

glib::wrapper! {
    pub struct SharePreviewWindow(ObjectSubclass<imp::SharePreviewWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, libadwaita::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl SharePreviewWindow {
    pub fn new(app: &SharePreviewApplication) -> Self {
        let window: Self =
            glib::Object::new(&[]).expect("Failed to create SharePreviewWindow");
        window.set_application(Some(app));

        // Set icons for shell
        gtk::Window::set_default_icon_name(APP_ID);
        // Setup widgets
        window.setup_widgets();
        // Setup window actions
        window.setup_actions();
        // Setup widgets signals
        window.setup_signals();

        window
    }

    fn setup_widgets(&self) {
        let self_ = imp::SharePreviewWindow::from_instance(self);
        let gtk_settings = gtk::Settings::default().unwrap();
        
        self_.settings.bind(
            "dark-theme",
            &gtk_settings,
            "gtk-application-prefer-dark-theme",
        ).build();
    }

    fn setup_actions(&self) {
        let self_ = imp::SharePreviewWindow::from_instance(self);
        let main_stack = &*self_.main_stack;
        let metadata = &*self_.metadata;
        let social = &*self_.social;
        let url_box = &*self_.url_box;
        let url_entry = &*self_.url_entry;
        let url_error = &*self_.url_error;
        let stack = &*self_.stack;
        let spinner = &*self_.spinner;
        let error_title = &*self_.error_title;
        let error_message = &*self_.error_message;
        let cardbox = &*self_.cardbox;

        // Run
        action!(
            self,
            "run",
            clone!(
                    @weak self as win, @weak social,
                    @weak url_entry, @weak url_error, @weak url_box,
                    @weak stack, @weak spinner, @weak error_title, @weak error_message,
                    @weak cardbox => move |_, _| {
                
                if !url_entry.text().is_empty() {
                    let mut url = url_entry.text().to_string();
                    match (url.starts_with("http://"), url.starts_with("https://")) {
                        (false, false) => {
                            url.insert_str(0, "http://");
                            url_entry.set_text(&url);
                        },
                        _ => ()
                    }

                    match Url::parse(&url) {
                        Ok(url) => {
                            url_error.set_reveal_child(false);
                            url_box.set_sensitive(false);
                            stack.set_visible_child_name("loading");
                            spinner.start();
                            spawn!(async move {
                                match scrape(&url).await {
                                    Ok(data) => {
                                        let win_ = imp::SharePreviewWindow::from_instance(&win);
                                        win_.data.replace(data);
                                        win_.active_url.replace(url.to_string());
                                        win.update_card();
                                        stack.set_visible_child_name("card");
                                    }
                                    Err(error) => {
                                        let error_texts = match error {
                                            Error::NetworkError(_) => (
                                                gettext("Network error"),
                                                gettext("Couldn't connect to the given URL.")
                                            ),
                                            Error::Unexpected => (
                                                gettext("Unexpected error"),
                                                gettext("Couldn't connect to the given URL.")
                                            )
                                        };
                                        error_title.set_label(&error_texts.0);
                                        error_message.set_label(&error_texts.1);
                                        stack.set_visible_child_name("error");
                                    }
                                }
                                spinner.stop();
                                url_box.set_sensitive(true);
                            });
                        }
                        Err(_) => {
                            url_error.set_reveal_child(true);
                        }
                    }
                }
            })
        );

        // Metadata
        action!(
            self,
            "metadata",
            clone!(@weak self as win, @weak main_stack, @weak metadata, => move |_, _| {
                let win_ = imp::SharePreviewWindow::from_instance(&win);
                let data = win_.data.borrow();
                metadata.set_items(&data.get_metadata());
                main_stack.set_visible_child_name("metadata");
            })
        );

        // Back
        action!(
            self,
            "back",
            clone!(@weak main_stack => move |_, _| {
                main_stack.set_visible_child_name("home");
            })
        );
    }

    fn setup_signals(&self) {
        let self_ = imp::SharePreviewWindow::from_instance(self);
        let url_entry = &*self_.url_entry;

        self_.dark_theme.connect_clicked(
            clone!(@weak self as win => move |_| {
                let win_ = imp::SharePreviewWindow::from_instance(&win);
                win_.settings.set_boolean("dark-theme", !win_.settings.boolean("dark-theme"))
                .expect("Error setting dark theme.");
            })
        );

        self_.url_entry.connect_activate(
            clone!(@weak self as win => move |_| {
                WidgetExt::activate_action(&win, "win.run", None);
            })
        );

        self_.url_entry.connect_icon_press(
            clone!(@weak self as win => move |_, icon| {
                match icon {
                    EntryIconPosition::Secondary => {
                        WidgetExt::activate_action(&win, "win.run", None);
                    },
                    _ => {}
                }
            })
        );

        self_.url_entry.connect_changed(
            clone!(@weak url_entry => move |_| {
                if url_entry.text().is_empty() {
                    url_entry.set_icon_sensitive(EntryIconPosition::Secondary, false);
                } else if !url_entry.text().is_empty() && !url_entry.icon_is_sensitive(EntryIconPosition::Secondary) {
                    url_entry.set_icon_sensitive(EntryIconPosition::Secondary, true);
                }
            })
        );

        self_.social.connect_local(
            "notify::selected",
            false,
            clone!(@weak self as win, @weak url_entry => @default-return None, move |_| {
                let win_ = imp::SharePreviewWindow::from_instance(&win);
                let active_url = win_.active_url.borrow().to_string();
                let entry_url = url_entry.text().to_string();

                if active_url == entry_url {
                    win.update_card();
                } else {
                    WidgetExt::activate_action(&win, "win.run", None);
                }
                None
            }),
        )
        .unwrap();
    }

    pub fn update_card(&self) {
        let self_ = imp::SharePreviewWindow::from_instance(self);
        let social = &*self_.social;
        let social = Self::get_social(&social.selected());
        let cardbox = &*self_.cardbox;
        let data = self_.data.borrow();
        let card = data.get_card(social);
        
        let old_card = self_.card.replace(Some(CardBox::new_from_card(&card)));
        
        if let Some(c) = old_card {
            cardbox.remove(&c);
        }
        cardbox.prepend(self_.card.borrow().as_ref().unwrap());
    }

    fn get_social(i: &u32) -> Social {
        match i {
            0 => Social::Facebook,
            1 => Social::Mastodon,
            2 => Social::Twitter,
            _ => unimplemented!()
        }
    }
}
