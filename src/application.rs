use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::*;
use gtk::{
    gio,
    glib,
    glib::clone
};
use gtk_macros::action;
use log::info;

use crate::{
    config,
    window::SharePreviewWindow
};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SharePreviewApplication {
        pub(super) windows: gtk::WindowGroup,
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
            self.parent_activate();

            if let Some(win) = self.obj().active_window() {
                win.present();
                return;
            }

            let win = self.obj().create_window();
            win.present();
        }

        fn startup(&self) {
            self.parent_startup();

            let app = self.obj();
            app.setup_gactions();
            app.setup_accels();
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

    fn setup_gactions(&self) {
        // Quit
        action!(
            self,
            "quit",
            clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _| {
                    app.quit();
                }
            )
        );

        // About
        action!(
            self,
            "about",
            clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _| {
                    app.show_about_dialog();
                }
            )
        );

        // New Window
        action!(
            self,
            "new",
            clone!(
                #[weak(rename_to = app)]
                self,
                move |_, _| {
                    let win = app.create_window();
                    win.present();
                }
            )
        );
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("app.new", &["<primary>n"]);
        self.set_accels_for_action("win.url", &["<primary>l"]);
    }

    fn show_about_dialog(&self) {
        let dialog = adw::AboutDialog::builder()
            .application_name(&gettext("Share Preview"))
            .application_icon(config::APP_ID)
            .issue_url("https://github.com/rafaelmardojai/share-preview/issues")
            .license_type(gtk::License::Gpl30)
            .website("https://apps.gnome.org/SharePreview/")
            .version(config::VERSION)
            .developers(vec!["Rafael Mardojai CM https://mardojai.com".to_string()])
            .artists(vec!["Rafael Mardojai CM https://mardojai.com".to_string(), "Tobias Bernard".to_string()])
            .build();

        dialog.present(Some(&self.active_window().unwrap()));
    }

    fn create_window(&self) -> SharePreviewWindow {
        let imp = self.imp();
        let window = SharePreviewWindow::new(&self);
        imp.windows.add_window(&window);
        window
    }

    pub fn run(&self) {
        info!("Share Preview ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}
