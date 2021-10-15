use crate::application::SharePreviewApplication;
use crate::backend::{scrape, Data, Error, Social};
use crate::config::{APP_ID, PROFILE};
use crate::widgets::{CardBox, DataDialog};

use adw::subclass::prelude::*;
use gettextrs::*;
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate, EntryIconPosition};
use gtk_macros::{action, spawn};
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
        pub color_scheme: TemplateChild<gtk::Button>,
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
        pub start_page: TemplateChild<adw::StatusPage>,
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
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),
                card: RefCell::new(Option::default()),
                data: RefCell::new(Data::default()),
                active_url: RefCell::new(String::default()),
                color_scheme: TemplateChild::default(),
                social: TemplateChild::default(),
                url_box: TemplateChild::default(),
                url_entry: TemplateChild::default(),
                url_error: TemplateChild::default(),
                stack: TemplateChild::default(),
                start_page: TemplateChild::default(),
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
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
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
        let imp = imp::SharePreviewWindow::from_instance(self);
        imp.start_page.set_icon_name(Some(APP_ID));
    }

    fn setup_actions(&self) {
        let imp = imp::SharePreviewWindow::from_instance(self);
        let social = &*imp.social;
        let url_box = &*imp.url_box;
        let url_entry = &*imp.url_entry;
        let url_error = &*imp.url_error;
        let stack = &*imp.stack;
        let spinner = &*imp.spinner;
        let error_title = &*imp.error_title;
        let error_message = &*imp.error_message;
        let cardbox = &*imp.cardbox;

        // Run
        action!(
            self,
            "run",
            clone!(
                    @weak self as win, @weak social, @weak url_entry,
                    @weak url_error, @weak url_box, @weak stack, @weak spinner,
                    @weak error_title, @weak error_message,
                    @weak cardbox => move |_, _| {

                if !url_entry.text().is_empty() {
                    let mut url = url_entry.text().trim().to_string();
                    match (url.starts_with("http://"), url.starts_with("https://")) {
                        (false, false) => {
                            url.insert_str(0, "http://");
                        },
                        _ => ()
                    }
                    url_entry.set_text(&url);

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
                                                gettext("Network Error"),
                                                gettext("Couldn't connect to the given URL.")
                                            ),
                                            Error::Unexpected(status) => (
                                                gettext("Unexpected Error"),
                                                if !status.is_empty() {
                                                    gettext!("Server Error {}", status)
                                                } else {
                                                    gettext("Couldn't connect to the given URL.")
                                                }
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
            clone!(@weak self as win => move |_, _| {
                let win_ = imp::SharePreviewWindow::from_instance(&win);
                let data = win_.data.borrow();

                let dialog = DataDialog::new(&data);
                dialog.set_transient_for(Some(&win));
                dialog.show();
            })
        );
    }

    fn setup_signals(&self) {
        let imp = imp::SharePreviewWindow::from_instance(self);
        let url_entry = &*imp.url_entry;

        imp.color_scheme.connect_clicked(
            clone!(@weak self as win => move |_| {
                let style_manager = adw::StyleManager::default().unwrap();
                if style_manager.is_dark() {
                    style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
                } else {
                    style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
                }
            })
        );

        imp.url_entry.connect_activate(
            clone!(@weak self as win => move |_| {
                WidgetExt::activate_action(&win, "win.run", None);
            })
        );

        imp.url_entry.connect_icon_press(
            clone!(@weak self as win => move |_, icon| {
                match icon {
                    EntryIconPosition::Secondary => {
                        WidgetExt::activate_action(&win, "win.run", None);
                    },
                    _ => {}
                }
            })
        );

        imp.url_entry.connect_changed(
            clone!(@weak url_entry => move |_| {
                if url_entry.text().is_empty() {
                    url_entry.set_icon_sensitive(EntryIconPosition::Secondary, false);
                } else if !url_entry.text().is_empty() && !url_entry.icon_is_sensitive(EntryIconPosition::Secondary) {
                    url_entry.set_icon_sensitive(EntryIconPosition::Secondary, true);
                }
            })
        );

        imp.social.connect_local(
            "notify::selected",
            false,
            clone!(@weak self as win => @default-return None, move |_| {
                let win_ = imp::SharePreviewWindow::from_instance(&win);
                let active_url = win_.active_url.borrow().to_string();

                if !active_url.is_empty() {
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
        let imp = imp::SharePreviewWindow::from_instance(self);
        let social = Self::get_social(&imp.social.selected());
        let data = imp.data.borrow();
        let card = data.get_card(social);

        let card = match card {
            Ok(card) => CardBox::new_from_card(&card),
            Err(error) => CardBox::new_from_error(&error)
        };

        let old_card = imp.card.replace(Some(card));
        if let Some(c) = old_card {
            imp.cardbox.remove(&c);
        }

        imp.cardbox.prepend(imp.card.borrow().as_ref().unwrap());
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
