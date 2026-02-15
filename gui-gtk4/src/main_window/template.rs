use super::MainWindow;

use glib::{
    object_subclass,
    subclass::{types::ObjectSubclass, InitializingObject},
};
use gtk4::subclass::prelude::*;
use gtk4::prelude::ButtonExt;
use gtk4::{Button, CompositeTemplate, TemplateChild};
use adw::subclass::prelude::AdwApplicationWindowImpl;
use adw::ApplicationWindow;

#[derive(CompositeTemplate, Default)]
#[template(file = "../../content/main-window.ui")]
pub struct MainWindowTemplate {
    #[template_child]
    pub button: TemplateChild<Button>,
}

#[object_subclass]
impl ObjectSubclass for MainWindowTemplate {
    const NAME: &'static str = "MainWindowTemplate";
    type Type = MainWindow;
    type ParentType = ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for MainWindowTemplate {
    fn constructed(&self) {
        self.parent_constructed();

        let button = self.button.get();
        button.connect_clicked(move |_btn| {
            println!("MainWindow button clicked");
        });
    }
}

impl WidgetImpl for MainWindowTemplate {}
impl WindowImpl for MainWindowTemplate {}
impl ApplicationWindowImpl for MainWindowTemplate {}
impl AdwApplicationWindowImpl for MainWindowTemplate {}