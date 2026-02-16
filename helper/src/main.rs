use evdev::{Device, EventType};
use std::process::Command;
use std::env;
use anyhow::Result;

fn main() -> Result<()> {
    let mut dev = Device::open("/dev/input/event10")?;

    println!("Listening for MyASUS button (KEY_PROG1)...");

    loop {
        for ev in dev.fetch_events()? {
            if ev.event_type() == EventType::KEY {
                if ev.code() == 148 && ev.value() == 1 {
                    println!("MyASUS button pressed! Launching GUI...");

                    let mut cmd = Command::new("/home/user/Projects/asus-control/gui-gtk4/target/debug/asus-control-gui");
                    cmd.envs(env::vars());
                    cmd.spawn().expect("failed to launch GUI");
                }
            }
        }
    }
}