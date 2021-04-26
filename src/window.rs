use crate::application::SharePreviewApplication;
use crate::config::{APP_ID, PROFILE};
use crate::backend::{
    scraper::scrape,
    elements::{Metadata, Error}
};
use glib::clone;
use glib::signal::Inhibit;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate};
use gtk_macros::{action, spawn};
use log::warn;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/rafaelmardojai/SharePreview/window.ui")]
    pub struct SharePreviewWindow {
        pub settings: gio::Settings,
        #[template_child]
        pub headerbar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub url_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub card_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub card_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub card_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub card_url: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SharePreviewWindow {
        const NAME: &'static str = "SharePreviewWindow";
        type Type = super::SharePreviewWindow;
        type ParentType = gtk::ApplicationWindow;

        fn new() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),
                headerbar: TemplateChild::default(),                
                url_entry: TemplateChild::default(),
                card_stack: TemplateChild::default(),
                spinner: TemplateChild::default(),
                card_title: TemplateChild::default(),
                card_description: TemplateChild::default(),
                card_url: TemplateChild::default(),
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
            let shortcuts = builder.get_object("shortcuts").unwrap();
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
}

glib::wrapper! {
    pub struct SharePreviewWindow(ObjectSubclass<imp::SharePreviewWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl SharePreviewWindow {
    pub fn new(app: &SharePreviewApplication) -> Self {
        let window: Self =
            glib::Object::new(&[]).expect("Failed to create SharePreviewWindow");
        window.set_application(Some(app));

        // Set icons for shell
        gtk::Window::set_default_icon_name(APP_ID);
        // Setup window actions
        window.setup_actions();
         // Setup widgets signals
        window.setup_signals();

        window
    }

    fn setup_actions(&self) {
        let self_ = imp::SharePreviewWindow::from_instance(self);
        let url = &*self_.url_entry;
        let stack = &*self_.card_stack;
        let spinner = &*self_.spinner;
        let title = &*self_.card_title;
        // Run
        action!(
            self,
            "run",
            clone!(@weak url, @weak stack, @weak spinner, @weak title => move |_, _| {
                stack.set_visible_child_name("loading");
                spinner.start();
                spawn!(async move {
                    // TODO: match
                    let metadata = scrape(url.text().as_str()).await.unwrap();
                    title.set_label(&metadata.title);
                    stack.set_visible_child_name("card");
                });
            })
        );
    }

    fn setup_signals(&self) {
        let self_ = imp::SharePreviewWindow::from_instance(self);

        self_.url_entry.connect_activate(clone!(@weak self as win => move |_| {
                WidgetExt::activate_action(&win, "win.run", None);
        }));
    }
}
