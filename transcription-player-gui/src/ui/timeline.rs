use glib::Object;

glib::wrapper! {
    pub struct Timeline(ObjectSubclass<imp::Timeline>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Timeline {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create Timeline")
    }
}

mod imp {
    use std::cell::RefCell;

    use adw::subclass::prelude::BinImpl;
    use glib::subclass::{
        prelude::{ObjectImpl, ObjectImplExt},
        types::ObjectSubclass,
    };
    use gtk::{
        prelude::AccessibleExtManual,
        subclass::widget::{WidgetClassSubclassExt, WidgetImpl},
        traits::{GestureExt, WidgetExt},
        Adjustment,
    };

    #[derive(Default)]
    pub struct Timeline {
        child: RefCell<Option<gtk::Scale>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Timeline {
        const NAME: &'static str = "Timeline";
        type Type = super::Timeline;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("timeline");
            klass.set_accessible_role(gtk::AccessibleRole::Slider);
        }
    }

    impl ObjectImpl for Timeline {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let child = gtk::Scale::new(gtk::Orientation::Horizontal, None::<&Adjustment>);
            child.set_parent(obj);
            *self.child.borrow_mut() = Some(child);

            obj.add_css_class("text-button");
            //obj.update_property(&[gtk::accessible::Property::);

            let gesture = gtk::GestureClick::new();
            gesture.connect_released(|gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
            });

            obj.add_controller(&gesture);
        }

        fn dispose(&self, _obj: &Self::Type) {
            if let Some(child) = self.child.borrow_mut().take() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Timeline {}

    impl BinImpl for Timeline {}
}
