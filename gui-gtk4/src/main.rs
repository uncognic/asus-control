use adw::{
    Application, ApplicationWindow, HeaderBar, WindowTitle,
    gtk::Orientation,
    prelude::{ApplicationExt, ApplicationExtManual, BoxExt, WidgetExt},
};
fn build_ui(application: &Application) {
    let content = adw::gtk::Box::new(Orientation::Vertical, 0);
    content.append(
        &HeaderBar::builder()
            .title_widget(&WindowTitle::new("asus-control-gui", ""))
            .build(),
    );
    let window = ApplicationWindow::builder()
        .application(application)
        .title("asus-control-gui")
        .default_height(600)
        .default_width(400)
        .content(&content)
        .build();
    window.show();
}
pub fn main() {
    let application = Application::new(Some("dev.uncognic.asus-control-gui"), Default::default());

    application.connect_activate(build_ui);
    application.run();
}
