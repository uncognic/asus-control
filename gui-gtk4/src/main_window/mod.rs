mod template;

use template::MainWindowTemplate;

use glib::{wrapper, Object};
use gtk4::gio::{ActionGroup, ActionMap};
use gtk4::prelude::*;
use gtk4::{Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager, Widget, Window};
use adw::Application;

wrapper! {
    pub struct MainWindow(ObjectSubclass<MainWindowTemplate>)
        @extends adw::ApplicationWindow, gtk4::ApplicationWindow, Window, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable,
                    ConstraintTarget, Native, Root, ShortcutManager;
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        let window: Self = Object::new::<Self>();
        window.set_application(Some(app));
        window
    }
}