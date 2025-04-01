use std::{
    cell::RefCell,
    str::FromStr
};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::*;
use gtk::{
    CompositeTemplate,
    EntryIconPosition,
    gio,
    glib,
    glib::clone
};
use gtk_macros::spawn;
use url::Url;

use crate::{
    application::SharePreviewApplication,
    backend::{Data, Error, Social, Log},
    config::{APP_ID, PROFILE},
    i18n::gettext_f,
    models::LogListModel,
    widgets::{CardBox, DataDialog, LogDialog}
};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/window.ui")]
    pub struct SharePreviewWindow {
        pub settings: gio::Settings,
        pub logger: LogListModel,
        pub card: RefCell<Option<CardBox>>,
        pub data: RefCell<Data>,
        pub active_url: RefCell<String>,
        #[template_child]
        pub toasts: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub color_scheme: TemplateChild<gtk::Button>,
        #[template_child]
        pub social: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub url_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub url_entry: TemplateChild<gtk::Entry>,
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
        #[template_child]
        pub war_count: TemplateChild<adw::ButtonContent>,
        #[template_child]
        pub err_count: TemplateChild<adw::ButtonContent>,
        #[template_child]
        pub inf_count: TemplateChild<adw::ButtonContent>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SharePreviewWindow {
        const NAME: &'static str = "SharePreviewWindow";
        type Type = super::SharePreviewWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),
                logger: LogListModel::new(),
                card: RefCell::new(Option::default()),
                data: RefCell::new(Data::default()),
                active_url: RefCell::new(String::default()),
                toasts: TemplateChild::default(),
                color_scheme: TemplateChild::default(),
                social: TemplateChild::default(),
                url_box: TemplateChild::default(),
                url_entry: TemplateChild::default(),
                stack: TemplateChild::default(),
                start_page: TemplateChild::default(),
                spinner: TemplateChild::default(),
                error_title: TemplateChild::default(),
                error_message: TemplateChild::default(),
                cardbox: TemplateChild::default(),
                war_count: TemplateChild::default(),
                err_count: TemplateChild::default(),
                inf_count: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();

            klass.install_action("win.run", None, move |win, _, _| {
                win.run();
            });

            klass.install_action("win.metadata", None, move |win, _, _| {
                win.show_metadata();
            });

            klass.install_action("win.log", None, move |win, _, _| {
                win.show_log();
            });

            klass.install_action("win.url", None, move |win, _, _| {
                win.imp().url_entry.grab_focus();
            });
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
                obj.add_css_class("devel");
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

#[gtk::template_callbacks]
impl SharePreviewWindow {
    pub fn new(app: &SharePreviewApplication) -> Self {
        let window: Self = glib::Object::builder().build();
        window.set_application(Some(app));

        // Set icons for shell
        gtk::Window::set_default_icon_name(APP_ID);
        // Setup widgets
        window.setup_widgets();

        window
    }

    pub fn open_uri(&self, uri: &str) {
        self.imp().url_entry.set_text(uri);
        self.run();
    }

    fn setup_widgets(&self) {
        self.imp().start_page.set_icon_name(Some(APP_ID));

        let social = self.imp().settings.string("social");
        self.imp().social.set_selected(Social::from_str(&social).unwrap() as u32);

        self.imp().logger.connect_items_changed(
            clone!(
                #[weak(rename_to = win)]
                self,
                move |logger,_,_,_| {
                    let (inf_count, war_count, err_count) = logger.worrying_count();

                    if inf_count > 0 {
                        win.imp().inf_count.add_css_class("accent");
                        win.imp().inf_count.remove_css_class("dim-label");
                    } else {
                        win.imp().inf_count.add_css_class("dim-label");
                        win.imp().inf_count.remove_css_class("accent");
                    }
                    if war_count > 0 {
                        win.imp().war_count.add_css_class("warning");
                        win.imp().war_count.remove_css_class("dim-label");
                    } else {
                        win.imp().war_count.add_css_class("dim-label");
                        win.imp().war_count.remove_css_class("warning");
                    }
                    if err_count > 0 {
                        win.imp().err_count.add_css_class("error");
                        win.imp().err_count.remove_css_class("dim-label");
                    } else {
                        win.imp().err_count.add_css_class("dim-label");
                        win.imp().err_count.remove_css_class("error");
                    }

                    win.imp().inf_count.set_label(&inf_count.to_string());
                    win.imp().war_count.set_label(&war_count.to_string());
                    win.imp().err_count.set_label(&err_count.to_string());
                }
            )
        );
    }

    fn run(&self) {
        let imp = self.imp();

        if !imp.url_entry.text().is_empty() {
            let mut url = imp.url_entry.text().trim().to_string();
            match (url.starts_with("http://"), url.starts_with("https://")) {
                (false, false) => {
                    url.insert_str(0, "http://");
                },
                _ => ()
            }
            imp.url_entry.set_text(&url);

            match Url::parse(&url) {
                Ok(url) => {
                    imp.url_entry.remove_css_class("error");
                    imp.url_box.set_sensitive(false);
                    imp.stack.set_visible_child_name("loading");
                    imp.spinner.start();
                    let spawn = clone!(
                        #[weak(rename_to = win)]
                        self,
                        move || {
                            spawn!(async move {
                                let imp = win.imp();
                                match Data::from_url(&url).await {
                                    Ok(data) => {
                                        imp.data.replace(data);
                                        imp.active_url.replace(url.to_string());
                                        win.update_card().await;
                                        imp.stack.set_visible_child_name("card");
                                    }
                                    Err(error) => {
                                        let error_texts = match error {
                                            Error::NetworkError(_) => (
                                                gettext("Network Error"),
                                                gettext("Couldn’t connect to the given URL.")
                                            ),
                                            Error::Unexpected(status) => (
                                                gettext("Unexpected Error"),
                                                if !status.is_empty() {
                                                    gettext_f("Server Error {status}",  &[("status", &status)])
                                                } else {
                                                    gettext("Couldn’t connect to the given URL.")
                                                }
                                            )
                                        };
                                        imp.error_title.set_label(&error_texts.0);
                                        imp.error_message.set_label(&error_texts.1);
                                        imp.stack.set_visible_child_name("error");
                                    }
                                }
                                imp.spinner.stop();
                                imp.url_box.set_sensitive(true);
                            });
                        }
                    );
                    spawn();
                }
                Err(_) => {
                    let toast = adw::Toast::new(&gettext("Invalid URL"));
                    imp.toasts.add_toast(toast);
                    imp.url_entry.add_css_class("error");
                }
            }
        }
    }

    fn show_metadata(&self) {
        let data = self.imp().data.borrow();
        let dialog = DataDialog::new(&data);
        dialog.set_transient_for(Some(self));
        dialog.present();
    }

    fn show_log(&self) {
        let dialog = LogDialog::new(&self.imp().logger);
        dialog.present(Some(self));
    }

    #[template_callback]
    fn on_color_scheme_clicked(&self) {
        let style_manager = adw::StyleManager::default();
        if style_manager.is_dark() {
            style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
        } else {
            style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
        }
    }

    #[template_callback]
    fn on_url_entry_changed(&self, entry: &gtk::Entry) {
        if entry.text().is_empty() {
            entry.set_icon_sensitive(EntryIconPosition::Secondary, false);
        } else if !entry.text().is_empty() && !entry.icon_is_sensitive(EntryIconPosition::Secondary) {
            entry.set_icon_sensitive(EntryIconPosition::Secondary, true);
        }
    }

    #[template_callback]
    fn on_url_entry_activate(&self, _entry: &gtk::Entry) {
        WidgetExt::activate_action(self, "win.run", None).unwrap();
    }

    #[template_callback]
    fn on_url_entry_icon_activate(&self, icon: EntryIconPosition) {
        match icon {
            EntryIconPosition::Secondary => {
                WidgetExt::activate_action(self, "win.run", None).unwrap();
            },
            _ => {}
        }
    }

    #[template_callback]
    fn on_social_selected(&self, _pspec: glib::ParamSpec, drop_down: &gtk::DropDown) {
        let active_url = self.imp().active_url.borrow().to_string();
        let string = &drop_down
            .selected_item()
            .unwrap()
            .downcast::<gtk::StringObject>()
            .unwrap()
            .string();
        let social = Social::from_str(string).unwrap();
        self.imp().settings.set_string("social", &social.to_string()).ok();

        if !active_url.is_empty() {
            self.imp().stack.set_visible_child_name("loading");
            self.imp().spinner.start();
            let spawn = clone!(
                #[weak(rename_to = win)]
                self,
                move || {
                    spawn!(async move {
                        win.update_card().await;
                        win.imp().stack.set_visible_child_name("card");
                        win.imp().spinner.stop();
                    });
                }
            );
            spawn();
        }
    }

    pub async fn update_card(&self) {
        self.imp().logger.flush();

        let string =&self.imp().social
            .selected_item()
            .unwrap()
            .downcast::<gtk::StringObject>()
            .unwrap()
            .string();
        let social = Social::from_str(string).unwrap();
        let data = self.imp().data.borrow();
        let card = data.get_card(social, &self.imp().logger).await;

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
}
