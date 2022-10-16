use adw::Application;
use glib::Object;
use gtk::gio;

glib::wrapper! {
    pub struct TranscribleWindow(ObjectSubclass<imp::TranscribleWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Widget, gtk::Window,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl TranscribleWindow {
    pub fn new(app: &Application) -> Self {
        Object::new(&[("application", app)]).expect("Failed to create Window")
    }
}

mod imp {
    use adw::subclass::prelude::AdwApplicationWindowImpl;
    use glib::subclass::{
        prelude::{ObjectImpl, ObjectImplExt},
        types::ObjectSubclass,
        InitializingObject,
    };
    use gtk::{
        prelude::*,
        subclass::{
            prelude::ApplicationWindowImpl,
            widget::{CompositeTemplateClass, WidgetImpl},
            window::WindowImpl,
        },
        CompositeTemplate,
    };

    use crate::ui::timeline::Timeline;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ninja/seppli/Transcrible/window.ui")]
    pub struct TranscribleWindow {}

    #[glib::object_subclass]
    impl ObjectSubclass for TranscribleWindow {
        const NAME: &'static str = "TranscribleWindow";
        type Type = super::TranscribleWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Timeline::static_type();

            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TranscribleWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for TranscribleWindow {}

    impl WindowImpl for TranscribleWindow {}

    impl ApplicationWindowImpl for TranscribleWindow {}
    impl AdwApplicationWindowImpl for TranscribleWindow {}
}
