use super::MainWindow;

use adw::ApplicationWindow;
use adw::subclass::prelude::AdwApplicationWindowImpl;
use glib::{
    object_subclass,
    subclass::{InitializingObject, types::ObjectSubclass},
};
use gtk4::prelude::ButtonExt;
use gtk4::subclass::prelude::*;
use gtk4::{Button, CompositeTemplate, TemplateChild};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::thread;

#[derive(CompositeTemplate, Default)]
#[template(file = "../../content/main-window.ui")]
pub struct MainWindowTemplate {
    #[template_child]
    pub btn_silent: TemplateChild<Button>,
    #[template_child]
    pub btn_balanced: TemplateChild<Button>,
    #[template_child]
    pub btn_performance: TemplateChild<Button>,
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
        let send = |cmd: &'static str| {
            let cmd = cmd.to_string();
            thread::spawn(
                move || match UnixStream::connect("/run/asus-control-daemon.sock") {
                    Ok(mut stream) => {
                        if let Err(e) = stream.write_all(cmd.as_bytes()) {
                            eprintln!("Failed to write to daemon: {}", e);
                            return;
                        }
                        if let Err(e) = stream.shutdown(std::net::Shutdown::Write) {
                            eprintln!("Failed to shutdown write side of socket: {}", e);
                        }
                        let mut resp = String::new();
                        if let Err(e) = stream.read_to_string(&mut resp) {
                            eprintln!("Failed to read from daemon: {}", e);
                        } else {
                            eprintln!("Daemon response: {}", resp);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to daemon socket: {}", e);
                    }
                },
            );
        };

        let silent = self.btn_silent.get();
        silent.connect_clicked(move |_btn| {
            send("profile quiet\n");
        });

        let balanced = self.btn_balanced.get();
        balanced.connect_clicked(move |_btn| {
            send("profile balanced\n");
        });

        let performance = self.btn_performance.get();
        performance.connect_clicked(move |_btn| {
            send("profile performance\n");
        });
    }
}

impl WidgetImpl for MainWindowTemplate {}
impl WindowImpl for MainWindowTemplate {}
impl ApplicationWindowImpl for MainWindowTemplate {}
impl AdwApplicationWindowImpl for MainWindowTemplate {}
