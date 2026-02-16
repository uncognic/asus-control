use super::MainWindow;

use adw::ApplicationWindow;
use adw::subclass::prelude::AdwApplicationWindowImpl;
use glib::{
    object_subclass,
    subclass::{InitializingObject, types::ObjectSubclass},
};
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{CompositeTemplate, Label, Scale, TemplateChild, ToggleButton};
use std::cell::RefCell;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::thread;

#[derive(CompositeTemplate, Default)]
#[template(file = "../../content/main-window.ui")]
pub struct MainWindowTemplate {
    #[template_child]
    pub btn_silent: TemplateChild<ToggleButton>,
    #[template_child]
    pub btn_balanced: TemplateChild<ToggleButton>,
    #[template_child]
    pub btn_performance: TemplateChild<ToggleButton>,
    #[template_child]
    pub battery_slider: TemplateChild<Scale>,
    #[template_child]
    pub battery_value: TemplateChild<Label>,
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
        let send_cmd: std::sync::Arc<dyn Fn(String) + Send + Sync + 'static> =
            std::sync::Arc::new(move |cmd: String| {
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
            });

        let slider = self.battery_slider.get();

        let (tx, rx) = std::sync::mpsc::channel::<i32>();
        thread::spawn(
            move || match UnixStream::connect("/run/asus-control-daemon.sock") {
                Ok(mut stream) => {
                    if let Err(e) = stream.write_all(b"get battery-threshold\n") {
                        eprintln!("Failed to write to daemon: {}", e);
                        return;
                    }
                    let _ = stream.shutdown(std::net::Shutdown::Write);
                    let mut resp = String::new();
                    if let Err(e) = stream.read_to_string(&mut resp) {
                        eprintln!("Failed to read from daemon: {}", e);
                    } else if let Ok(n) = resp.trim().parse::<i32>() {
                        let _ = tx.send(n);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to daemon socket: {}", e);
                }
            },
        );

        let slider_clone_for_init = slider.clone();
        let (value_tx, value_rx) = std::sync::mpsc::channel::<i32>();
        glib::idle_add_local(move || match rx.try_recv() {
            Ok(n) => {
                let _ = value_tx.send(n);
                true.into()
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => true.into(),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => false.into(),
        });

        let slider_for_label = slider_clone_for_init.clone();
        let _value_setter = glib::idle_add_local(move || match value_rx.try_recv() {
            Ok(n) => {
                slider_for_label.set_value(n as f64);
                false.into()
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => true.into(),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => false.into(),
        });
        let slider_for_send = self.battery_slider.get();
        let silent_btn = self.btn_silent.get();
        let balanced_btn = self.btn_balanced.get();
        let performance_btn = self.btn_performance.get();

        let (ptx, prx) = std::sync::mpsc::channel::<String>();
        thread::spawn(
            move || match UnixStream::connect("/run/asus-control-daemon.sock") {
                Ok(mut stream) => {
                    if let Err(e) = stream.write_all(b"get profile\n") {
                        eprintln!("Failed to write to daemon: {}", e);
                        return;
                    }
                    let _ = stream.shutdown(std::net::Shutdown::Write);
                    let mut resp = String::new();
                    if let Err(e) = stream.read_to_string(&mut resp) {
                        eprintln!("Failed to read from daemon: {}", e);
                    } else {
                        let _ = ptx.send(resp);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to daemon socket: {}", e);
                }
            },
        );

        let silent_for_idle = silent_btn.clone();
        let balanced_for_idle = balanced_btn.clone();
        let performance_for_idle = performance_btn.clone();
        let suppress_profile_signals = Rc::new(RefCell::new(false));
        let suppress_for_idle = suppress_profile_signals.clone();
        let _profile_setter = glib::idle_add_local(move || match prx.try_recv() {
            Ok(s) => {
                *suppress_for_idle.borrow_mut() = true;
                match s.trim() {
                    "quiet" => {
                        silent_for_idle.set_active(true);
                        balanced_for_idle.set_active(false);
                        performance_for_idle.set_active(false);
                    }
                    "balanced" => {
                        silent_for_idle.set_active(false);
                        balanced_for_idle.set_active(true);
                        performance_for_idle.set_active(false);
                    }
                    "performance" => {
                        silent_for_idle.set_active(false);
                        balanced_for_idle.set_active(false);
                        performance_for_idle.set_active(true);
                    }
                    _ => {}
                }
                *suppress_for_idle.borrow_mut() = false;
                false.into()
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => true.into(),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => false.into(),
        });
        let value_label = self.battery_value.get();
        let send_cmd_for_slider = send_cmd.clone();
        let pending: Rc<RefCell<Option<glib::source::SourceId>>> = Rc::new(RefCell::new(None));
        let pending_clone = pending.clone();
        slider_for_send.connect_value_changed(move |s| {
            let v = s.value() as i32;
            value_label.set_label(&format!("{}%", v));

            if let Some(id) = pending_clone.borrow_mut().take() {
                id.remove();
            }

            let sc = send_cmd_for_slider.clone();
            let pending_for_timeout = pending_clone.clone();
            let id = glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
                sc(format!("set battery-threshold {}\n", v));
                *pending_for_timeout.borrow_mut() = None;
                false.into()
            });
            *pending_clone.borrow_mut() = Some(id);
        });

        let silent = silent_btn.clone();
        let send_cmd_s = send_cmd.clone();
        let suppress_for_silent = suppress_profile_signals.clone();
        let balanced_for_s = balanced_btn.clone();
        let perf_for_s = performance_btn.clone();
        silent.connect_toggled(move |btn| {
            if *suppress_for_silent.borrow() {
                return;
            }
            if btn.is_active() {
                balanced_for_s.set_active(false);
                perf_for_s.set_active(false);
                let sc = send_cmd_s.clone();
                sc("set profile quiet\n".to_string());
            }
        });

        let balanced = balanced_btn.clone();
        let send_cmd_b = send_cmd.clone();
        let suppress_for_balanced = suppress_profile_signals.clone();
        let silent_for_b = silent.clone();
        let perf_for_b = performance_btn.clone();
        balanced.connect_toggled(move |btn| {
            if *suppress_for_balanced.borrow() {
                return;
            }
            if btn.is_active() {
                silent_for_b.set_active(false);
                perf_for_b.set_active(false);
                let sc = send_cmd_b.clone();
                sc("set profile balanced\n".to_string());
            }
        });

        let performance = performance_btn.clone();
        let send_cmd_p = send_cmd.clone();
        let suppress_for_perf = suppress_profile_signals.clone();
        let silent_for_p = silent.clone();
        let balanced_for_p = balanced.clone();
        performance.connect_toggled(move |btn| {
            if *suppress_for_perf.borrow() {
                return;
            }
            if btn.is_active() {
                silent_for_p.set_active(false);
                balanced_for_p.set_active(false);
                let sc = send_cmd_p.clone();
                sc("set profile performance\n".to_string());
            }
        });
    }
}

impl WidgetImpl for MainWindowTemplate {}
impl WindowImpl for MainWindowTemplate {}
impl ApplicationWindowImpl for MainWindowTemplate {}
impl AdwApplicationWindowImpl for MainWindowTemplate {}
