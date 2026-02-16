mod main_window;

use main_window::MainWindow;

use adw::prelude::*;
use gtk4::prelude::GtkWindowExt;
use adw::Application;

fn build_ui(application: &Application) {
    let window = MainWindow::new(application);
    GtkWindowExt::present(&window);
}

pub fn main() {
    let application = Application::builder()
        .application_id("dev.uncognic.asus-control-gui")
        .build();

    application.connect_activate(build_ui);
    application.run();
}