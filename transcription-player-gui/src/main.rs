mod audio;
pub mod ui {
    pub mod main_window;
}

use audio::AudioPlayer;
use gtk::{gdk::Display, gio, prelude::*, CssProvider, StyleContext};
use relm4::RelmApp;
use ui::main_window;

const APP_ID: &str = "ninja.seppli.Transcrible";

fn main() {
    gio::resources_register_include!("transcrible.gresource")
        .expect("Failed to register resources");

    let player = AudioPlayer::new().expect("Couldn't create AudioPlayer");
    main_window::start_app(player);
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_resource("/ninja/seppli/Transcrible/style.css");

    StyleContext::add_provider_for_display(
        &Display::default().expect("Could not connect to a display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn setup_shortcuts(app: &adw::Application) {}
