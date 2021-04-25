use crate::application::SharePreviewApplication;
use crate::config::{APP_ID, PROFILE};
use crate::metadata::scraper::scrape;
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
    pub struct SharePreviewApplicationWindow {
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
    impl ObjectSubclass for SharePreviewApplicationWindow {
        const NAME: &'static str = "SharePreviewApplicationWindow";
        type Type = super::SharePreviewApplicationWindow;
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

    impl ObjectImpl for SharePreviewApplicationWindow {
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

            // Setup window actions
            obj.setup_gactions();

            // load latest window state
            obj.load_window_size();
        }
    }

    impl WidgetImpl for SharePreviewApplicationWindow {}
    impl WindowImpl for SharePreviewApplicationWindow {
        // save window state on delete event
        fn close_request(&self, obj: &Self::Type) -> Inhibit {
            if let Err(err) = obj.save_window_size() {
                warn!("Failed to save window state, {}", &err);
            }
            Inhibit(false)
        }
    }

    impl ApplicationWindowImpl for SharePreviewApplicationWindow {}
}

glib::wrapper! {
    pub struct SharePreviewApplicationWindow(ObjectSubclass<imp::SharePreviewApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl SharePreviewApplicationWindow {
    pub fn new(app: &SharePreviewApplication) -> Self {
        let window: Self =
            glib::Object::new(&[]).expect("Failed to create SharePreviewApplicationWindow");
        window.set_application(Some(app));

        // Set icons for shell
        gtk::Window::set_default_icon_name(APP_ID);

        window
    }

    fn setup_gactions(&self) {
        let self_ = imp::SharePreviewApplicationWindow::from_instance(self);
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

    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = &imp::SharePreviewApplicationWindow::from_instance(self).settings;

        let size = self.default_size();

        settings.set_int("window-width", size.0)?;
        settings.set_int("window-height", size.1)?;

        settings.set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let settings = &imp::SharePreviewApplicationWindow::from_instance(self).settings;

        let width = settings.get_int("window-width");
        let height = settings.get_int("window-height");
        let is_maximized = settings.get_boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }
}
