mod audio;
pub mod ui {
    pub mod timeline;
    pub mod window;
}

use gtk::{gio, prelude::*};
use ui::window::TranscribleWindow;

const APP_ID: &str = "ninja.seppli.Transcrible";

fn main() {
    gio::resources_register_include!("transcrible.gresource")
        .expect("Failed to register resources");

    let mut app = adw::Application::builder().application_id(APP_ID).build();
    let app2 = &mut app;
    app.connect_activate(build_ui);
    //app.connect_startup(setup_shortcuts);

    app.run();
}

fn build_ui(app: &adw::Application) {
    let window = TranscribleWindow::new(app);
    window.present();
    window.show();
}
fn setup_shortcuts(app: &adw::Application) {}
