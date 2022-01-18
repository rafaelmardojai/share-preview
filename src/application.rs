use crate::config;
use crate::window::SharePreviewWindow;

use adw::subclass::prelude::*;
use gettextrs::*;
use gio::ApplicationFlags;
use glib::clone;
use glib::WeakRef;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk_macros::action;
use log::{debug, info};
use once_cell::sync::OnceCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SharePreviewApplication {
        pub window: OnceCell<WeakRef<SharePreviewWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SharePreviewApplication {
        const NAME: &'static str = "SharePreviewApplication";
        type Type = super::SharePreviewApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for SharePreviewApplication {}

    impl gio::subclass::prelude::ApplicationImpl for SharePreviewApplication {
        fn activate(&self, app: &Self::Type) {
            debug!("AdwApplication<SharePreviewApplication>::activate");

            let priv_ = SharePreviewApplication::from_instance(app);
            if let Some(window) = priv_.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }

            app.set_resource_base_path(Some("/com/rafaelmardojai/SharePreview/"));

            let window = SharePreviewWindow::new(app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.setup_gactions();
            app.setup_accels();

            app.get_main_window().present();
        }

        fn startup(&self, app: &Self::Type) {
            debug!("GtkApplication<SharePreviewApplication>::startup");
            self.parent_startup(app);
        }
    }

    impl GtkApplicationImpl for SharePreviewApplication {}
    impl AdwApplicationImpl for SharePreviewApplication {}
}

glib::wrapper! {
    pub struct SharePreviewApplication(ObjectSubclass<imp::SharePreviewApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl SharePreviewApplication {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &ApplicationFlags::empty()),
            (
                "resource-base-path",
                &Some("/com/rafaelmardojai/SharePreview/"),
            ),
        ])
        .expect("Application initialization failed...")
    }

    fn get_main_window(&self) -> SharePreviewWindow {
        let priv_ = imp::SharePreviewApplication::from_instance(self);
        priv_.window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        action!(
            self,
            "quit",
            clone!(@weak self as app => move |_, _| {
                // This is needed to trigger the delete event
                // and saving the window state
                app.get_main_window().close();
                app.quit();
            })
        );

        // About
        action!(
            self,
            "about",
            clone!(@weak self as app => move |_, _| {
                app.show_about_dialog();
            })
        );
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);
    }

    fn show_about_dialog(&self) {
        let dialog = gtk::builders::AboutDialogBuilder::new()
            .program_name(&gettext("Share Preview"))
            .logo_icon_name(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/rafaelmardojai/share-preview/")
            .version(config::VERSION)
            .transient_for(&self.get_main_window())
            .modal(true)
            .authors(vec!["Rafael Mardojai CM".into()])
            .artists(vec!["Rafael Mardojai CM".into(), "Tobias Bernard".into()])
            .build();

        dialog.show();
    }

    pub fn run(&self) {
        info!("Share Preview ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}
