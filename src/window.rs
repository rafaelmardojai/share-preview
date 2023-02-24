use crate::application::SharePreviewApplication;
use crate::backend::{scrape, Data, Error, Social};
use crate::config::{APP_ID, PROFILE};
use crate::widgets::{CardBox, DataDialog};

use adw::subclass::prelude::*;
use gettextrs::*;
use glib::clone;
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
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

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
        let window: Self = glib::Object::builder().build();
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
        self.imp().start_page.set_icon_name(Some(APP_ID));
    }

    fn setup_actions(&self) {
        let social = &*self.imp().social;
        let url_box = &*self.imp().url_box;
        let url_entry = &*self.imp().url_entry;
        let url_error = &*self.imp().url_error;
        let stack = &*self.imp().stack;
        let spinner = &*self.imp().spinner;
        let error_title = &*self.imp().error_title;
        let error_message = &*self.imp().error_message;
        let cardbox = &*self.imp().cardbox;

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
                                        win.imp().data.replace(data);
                                        win.imp().active_url.replace(url.to_string());
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
                let data = win.imp().data.borrow();

                let dialog = DataDialog::new(&data);
                dialog.set_transient_for(Some(&win));
                dialog.show();
            })
        );
    }

    fn setup_signals(&self) {
        let url_entry = &*self.imp().url_entry;

        self.imp().color_scheme.connect_clicked(
            clone!(@weak self as win => move |_| {
                let style_manager = adw::StyleManager::default();
                if style_manager.is_dark() {
                    style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
                } else {
                    style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
                }
            })
        );

        self.imp().url_entry.connect_activate(
            clone!(@weak self as win => move |_| {
                WidgetExt::activate_action(&win, "win.run", None).unwrap();
            })
        );

        self.imp().url_entry.connect_icon_press(
            clone!(@weak self as win => move |_, icon| {
                match icon {
                    EntryIconPosition::Secondary => {
                        WidgetExt::activate_action(&win, "win.run", None).unwrap();
                    },
                    _ => {}
                }
            })
        );

        self.imp().url_entry.connect_changed(
            clone!(@weak url_entry => move |_| {
                if url_entry.text().is_empty() {
                    url_entry.set_icon_sensitive(EntryIconPosition::Secondary, false);
                } else if !url_entry.text().is_empty() && !url_entry.icon_is_sensitive(EntryIconPosition::Secondary) {
                    url_entry.set_icon_sensitive(EntryIconPosition::Secondary, true);
                }
            })
        );

        self.imp().social.connect_local(
            "notify::selected",
            false,
            clone!(@weak self as win => @default-return None, move |_| {
                let active_url = win.imp().active_url.borrow().to_string();

                if !active_url.is_empty() {
                    win.update_card();
                } else {
                    WidgetExt::activate_action(&win, "win.run", None).unwrap();
                }
                None
            }),
        );
    }

    pub fn update_card(&self) {
        let social = Self::get_social(&self.imp().social.selected());
        let data = self.imp().data.borrow();
        let card = data.get_card(social);

        let card = match card {
            Ok(card) => CardBox::new_from_card(&card),
            Err(error) => CardBox::new_from_error(&error)
        };

        let old_card = self.imp().card.replace(Some(card));
        if let Some(c) = old_card {
            self.imp().cardbox.remove(&c);
        }

        self.imp().cardbox.prepend(self.imp().card.borrow().as_ref().unwrap());
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
