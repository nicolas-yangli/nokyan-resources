use adw::{prelude::*, subclass::prelude::*};
use gtk::glib;
use gtk::subclass::prelude::*;

use crate::config::PROFILE;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/me/nalux/Resources/ui/widgets/info_box.ui")]
    pub struct ResInfoBox {
        #[template_child]
        pub info_label: TemplateChild<gtk::Label>,
    }

    impl Default for ResInfoBox {
        fn default() -> Self {
            Self {
                info_label: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResInfoBox {
        const NAME: &'static str = "ResInfoBox";
        type Type = super::ResInfoBox;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ResInfoBox {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
        }
    }

    impl WidgetImpl for ResInfoBox {}

    impl ListBoxRowImpl for ResInfoBox {}

    impl PreferencesRowImpl for ResInfoBox {}

    impl ActionRowImpl for ResInfoBox {}
}

glib::wrapper! {
    pub struct ResInfoBox(ObjectSubclass<imp::ResInfoBox>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl ResInfoBox {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ResInfoBox")
    }

    pub fn set_info_label(&self, str: &str) {
        let imp = self.imp();
        imp.info_label.set_label(str);
    }
}