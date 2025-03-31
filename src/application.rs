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

    impl ObjectImpl for SharePreviewApplication {
        fn constructed(&self) {
            let app = self.obj();

            app.add_main_option(
                "new-window",
                b'w'.into(),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                "Always open a new window for opening specified URIs",
                None
            );

            app.setup_gactions();
            app.setup_accels();
        }
    }

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
        }

        fn command_line(&self, command_line: &gio::ApplicationCommandLine) -> glib::ExitCode {
            let args = command_line.arguments();
            let options = command_line.options_dict();
            let uris = args[1..].to_vec();
            let new_window = options.contains("new-window");

            if uris.len() > 0 {
                // Create a new window instance for each URL argument
                for (i, uri) in uris.iter().enumerate()  {
                    // If not new window option and we're in the first URL, get the active window if exists
                    let win = if let (Some(win), false, 0) = (self.obj().active_window(), new_window, i) {
                        win.downcast::<SharePreviewWindow>().unwrap()
                    } else {
                        self.obj().create_window()
                    };

                    win.present();
                    win.open_uri(uri.to_str().unwrap());
                }
            } else {
                self.activate();
            }

            glib::ExitCode::SUCCESS
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
            .property("flags", &gio::ApplicationFlags::HANDLES_COMMAND_LINE)
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
            "new-window",
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
        self.set_accels_for_action("app.new-window", &["<primary>n"]);
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
