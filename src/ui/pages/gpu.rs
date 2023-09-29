use adw::{prelude::*, subclass::prelude::*};
use gtk::glib::{self, clone, timeout_future_seconds, MainContext};

use crate::config::PROFILE;
use crate::i18n::i18n;
use crate::utils::gpu::GPU;
use crate::utils::units::{to_largest_unit, Base};
use crate::utils::NaNDefault;

mod imp {
    use std::{
        cell::{Cell, RefCell},
        sync::OnceLock,
    };

    use crate::{ui::widgets::graph_box::ResGraphBox, utils::gpu::GPU};

    use super::*;

    use gtk::{
        gio::{Icon, ThemedIcon},
        glib::{ParamSpec, Properties, Value},
        CompositeTemplate,
    };

    #[derive(CompositeTemplate, Properties)]
    #[template(resource = "/me/nalux/Resources/ui/pages/gpu.ui")]
    #[properties(wrapper_type = super::ResGPU)]
    pub struct ResGPU {
        #[template_child]
        pub gpu_usage: TemplateChild<ResGraphBox>,
        #[template_child]
        pub vram_usage: TemplateChild<ResGraphBox>,
        #[template_child]
        pub temperature: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub power_usage: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub gpu_clockspeed: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub vram_clockspeed: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub manufacturer: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub pci_slot: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub driver_used: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub current_power_cap: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub max_power_cap: TemplateChild<adw::ActionRow>,

        pub gpu: OnceLock<GPU>,
        pub number: OnceLock<usize>,

        #[property(get)]
        uses_progress_bar: Cell<bool>,

        #[property(get)]
        icon: RefCell<Icon>,

        #[property(get, set)]
        usage: Cell<f64>,

        #[property(get = Self::tab_name, set = Self::set_tab_name, type = glib::GString)]
        tab_name: Cell<glib::GString>,
    }

    impl ResGPU {
        pub fn tab_name(&self) -> glib::GString {
            let tab_name = self.tab_name.take();
            let result = tab_name.clone();
            self.tab_name.set(tab_name);
            result
        }

        pub fn set_tab_name(&self, tab_name: &str) {
            self.tab_name.set(glib::GString::from(tab_name));
        }
    }

    impl Default for ResGPU {
        fn default() -> Self {
            Self {
                gpu_usage: Default::default(),
                vram_usage: Default::default(),
                temperature: Default::default(),
                power_usage: Default::default(),
                gpu_clockspeed: Default::default(),
                vram_clockspeed: Default::default(),
                manufacturer: Default::default(),
                pci_slot: Default::default(),
                driver_used: Default::default(),
                current_power_cap: Default::default(),
                max_power_cap: Default::default(),
                gpu: Default::default(),
                number: Default::default(),
                uses_progress_bar: Cell::new(true),
                icon: RefCell::new(ThemedIcon::new("gpu-symbolic").into()),
                usage: Default::default(),
                tab_name: Cell::new(glib::GString::from(i18n("GPU"))),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResGPU {
        const NAME: &'static str = "ResGPU";
        type Type = super::ResGPU;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ResGPU {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
        }

        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }
    }

    impl WidgetImpl for ResGPU {}
    impl BinImpl for ResGPU {}
}

glib::wrapper! {
    pub struct ResGPU(ObjectSubclass<imp::ResGPU>)
        @extends gtk::Widget, adw::Bin;
}

impl ResGPU {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn init(&self, gpu: GPU, number: usize) {
        let imp = self.imp();
        imp.gpu.set(gpu).unwrap_or_default();
        imp.number.set(number).unwrap_or_default();
        self.setup_widgets();
        self.setup_listener();
    }

    pub fn setup_widgets(&self) {
        let imp = self.imp();
        let gpu = &imp.gpu.get().unwrap();
        imp.gpu_usage.set_title_label(&i18n("GPU Usage"));
        imp.gpu_usage.set_data_points_max_amount(60);
        imp.gpu_usage.set_graph_color(230, 97, 0);
        imp.vram_usage.set_title_label(&i18n("Video Memory Usage"));
        imp.vram_usage.set_data_points_max_amount(60);
        imp.vram_usage.set_graph_color(192, 28, 40);
        imp.manufacturer
            .set_subtitle(&gpu.get_vendor().unwrap_or_else(|_| i18n("N/A")));
        imp.pci_slot.set_subtitle(&gpu.pci_slot);
        imp.driver_used.set_subtitle(&gpu.driver);
    }

    pub fn setup_listener(&self) {
        let main_context = MainContext::default();
        let gpu_usage_update = clone!(@strong self as this => async move {
            let imp = this.imp();
            let gpu = imp.gpu.get().unwrap();

            loop {
                if let Ok(gpu_usage) = gpu.get_gpu_usage().await {
                    let gpu_usage_fraction = gpu_usage as f64 / 100.0;
                    imp.gpu_usage.set_subtitle(&format!("{gpu_usage} %"));
                    imp.gpu_usage.push_data_point(gpu_usage_fraction);
                    imp.gpu_usage.set_graph_visible(true);
                    this.set_property("usage", gpu_usage_fraction);
                } else {
                    imp.gpu_usage.set_subtitle(&i18n("N/A"));
                    imp.gpu_usage.push_data_point(0.0);
                    imp.gpu_usage.set_graph_visible(false);
                }

                if let (Ok(total_vram), Ok(used_vram)) = (gpu.get_total_vram().await, gpu.get_used_vram().await) {
                    let total_vram_unit = to_largest_unit(total_vram as f64, &Base::Decimal);
                    let used_vram_unit = to_largest_unit(used_vram as f64, &Base::Decimal);
                    let used_vram_percentage = (used_vram as f64 / total_vram as f64).nan_default(0.0) * 100.0;
                    imp.vram_usage.set_subtitle(&format!("{:.2} {}B / {:.2} {}B · {} %", used_vram_unit.0, used_vram_unit.1, total_vram_unit.0, total_vram_unit.1, used_vram_percentage as u8));
                    imp.vram_usage.push_data_point((used_vram as f64) / (total_vram as f64));
                    imp.vram_usage.set_graph_visible(true);
                } else {
                    imp.vram_usage.set_subtitle(&i18n("N/A"));
                    imp.vram_usage.push_data_point(0.0);
                    imp.vram_usage.set_graph_visible(false);
                }

                // TODO: handle the user's choice of temperature unit
                let temp_unit = "C";
                imp.temperature.set_subtitle(&gpu.get_gpu_temp().await.map_or_else(|_| i18n("N/A"), |x| format!("{x} °{temp_unit}")));

                imp.power_usage.set_subtitle(&gpu.get_power_usage().await.map_or_else(|_| i18n("N/A"), |x| format!("{x:.1} W")));

                if let Ok(gpu_clockspeed) = gpu.get_gpu_speed().await {
                    let gpu_clockspeed_unit = to_largest_unit(gpu_clockspeed, &Base::Decimal);
                    imp.gpu_clockspeed.set_subtitle(&format!("{:.2} {}Hz", gpu_clockspeed_unit.0, gpu_clockspeed_unit.1));
                } else {
                    imp.gpu_clockspeed.set_subtitle(&i18n("N/A"));
                }

                if let Ok(vram_clockspeed) = gpu.get_vram_speed().await {
                    let vram_clockspeed_unit = to_largest_unit(vram_clockspeed, &Base::Decimal);
                    imp.vram_clockspeed.set_subtitle(&format!("{:.2} {}Hz", vram_clockspeed_unit.0, vram_clockspeed_unit.1));
                } else {
                    imp.vram_clockspeed.set_subtitle(&i18n("N/A"));
                }

                imp.current_power_cap.set_subtitle(&gpu.get_power_cap().await.map_or_else(|_| i18n("N/A"), |x| format!("{x:.1} W")));

                imp.max_power_cap.set_subtitle(&gpu.get_power_cap_max().await.map_or_else(|_| i18n("N/A"), |x| format!("{x:.1} W")));

                timeout_future_seconds(1).await;
            }
        });
        main_context.spawn_local(gpu_usage_update);
    }
}
