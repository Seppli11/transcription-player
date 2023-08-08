use std::path::PathBuf;

use adw::prelude::*;
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{
    send, AppUpdate, Components, Model, RelmApp, RelmComponent, Sender, WidgetPlus, Widgets,
};
use relm4_components::{
    open_button::{OpenButtonConfig, OpenButtonModel, OpenButtonParent, OpenButtonSettings},
    open_dialog::{OpenDialogConfig, OpenDialogSettings},
    ParentWindow,
};

use crate::audio::AudioPlayer;

struct AppModel {
    player: AudioPlayer,
}

enum AppMsg {
    LoadFile(PathBuf),
    TogglePlayStatus,
}

struct AppComponents {
    open_button: RelmComponent<OpenButtonModel<AppOpenButtonConfig>, AppModel>,
}

struct AppOpenButtonConfig {}
impl OpenButtonConfig for AppOpenButtonConfig {
    fn open_button_config(_model: &AppModel) -> OpenButtonSettings {
        OpenButtonSettings {
            text: "Open",
            recently_opened_files: None,
            max_recent_files: 0,
        }
    }
}

impl OpenDialogConfig for AppOpenButtonConfig {
    type Model = AppModel;
    fn open_dialog_config(_model: &AppModel) -> OpenDialogSettings {
        OpenDialogSettings {
            cancel_label: "Cancel",
            accept_label: "Open",
            create_folders: false,
            is_modal: true,
            filters: vec![],
        }
    }
}

impl OpenButtonParent for AppModel {
    fn open_msg(path: std::path::PathBuf) -> AppMsg {
        AppMsg::LoadFile(path)
    }
}

impl Components<AppModel> for AppComponents {
    fn init_components(
        parent_model: &AppModel,
        parent_sender: Sender<<AppModel as Model>::Msg>,
    ) -> Self {
        AppComponents {
            open_button: RelmComponent::new(parent_model, parent_sender),
        }
    }

    fn connect_parent(&mut self, _parent_widgets: &<AppModel as Model>::Widgets) {}
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(
        &mut self,
        msg: AppMsg,
        _components: &AppComponents,
        _sender: Sender<AppMsg>,
    ) -> bool {
        match msg {
            AppMsg::LoadFile(path) => self.player.load(&path).unwrap(),
            AppMsg::TogglePlayStatus => self.player.toggle_play_status(),
        };
        true
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        adw::ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            set_content = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                append = &adw::HeaderBar {
                    pack_start: components.open_button.root_widget()
                },
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    append = &gtk::Button::with_label("Play") {
                        connect_clicked(sender) => move |_| {
                            send!(sender, AppMsg::TogglePlayStatus);
                        }
                    }
                }
            }
        }
    }
}

impl ParentWindow for AppWidgets {
    fn parent_window(&self) -> Option<gtk::Window> {
        Some(self.root_widget().upcast())
    }
}

pub fn start_app(player: AudioPlayer) {
    let model = AppModel { player };
    let app = RelmApp::new(model);
    app.run()
}
