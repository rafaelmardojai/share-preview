use adw::subclass::prelude::*;
use gettextrs::*;
use gtk::{
    gio,
    glib,
    glib::clone,
    glib::WeakRef,
    prelude::*,
};
use gtk_macros::action;
use log::{info};
use once_cell::sync::OnceCell;

use crate::{
    config,
    window::SharePreviewWindow
};

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

    impl ApplicationImpl for SharePreviewApplication {
        fn activate(&self) {
            let app = self.obj();
            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }

            let window = SharePreviewWindow::new(&app.clone());
            window.present();
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.setup_gactions();
            app.setup_accels();
        }

        fn startup(&self) {
            self.parent_startup();
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
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("flags", &gio::ApplicationFlags::empty())
            .property("resource-base-path", Some("/com/rafaelmardojai/SharePreview/"))
            .build()
    }

    fn get_main_window(&self) -> SharePreviewWindow {
        self.imp().window.get().unwrap().upgrade().unwrap()
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
    }

    fn show_about_dialog(&self) {
        let dialog = gtk::AboutDialog::builder()
            .program_name(&gettext("Share Preview"))
            .logo_icon_name(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/rafaelmardojai/share-preview/")
            .version(config::VERSION)
            .transient_for(&self.get_main_window())
            .modal(true)
            .authors(vec!["Rafael Mardojai CM".to_string()])
            .artists(vec!["Rafael Mardojai CM".to_string(), "Tobias Bernard".to_string()])
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
