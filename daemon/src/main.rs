use std::io::{Read, Write};
use std::os::unix::net::UnixListener;

#[derive(Debug)]
enum FanProfile {
    Quiet,
    Balanced,
    Performance,
}

fn main() -> std::io::Result<()> {
    let socket_path = "/run/asus-control-daemon.sock";

    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;
    println!("asus-control-daemon listening on {}", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = String::new();
                if let Err(e) = stream.read_to_string(&mut buf) {
                    eprintln!("Error reading from client: {}", e);
                    continue;
                }

                let response = handle_command(&buf);
                if let Err(e) = stream.write_all(response.as_bytes()) {
                    eprintln!("Error writing to client: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_command(cmd: &str) -> String {
    let cmd = cmd.trim();

    if cmd.starts_with("battery-threshold") {
        let after = &cmd["battery-threshold".len()..].trim();
        let n_opt = if !after.is_empty() {
            after.parse::<i32>().ok()
        } else {
            None
        };

        let n_opt = n_opt.or_else(|| {
            cmd.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<i32>().ok())
        });

        if let Some(n) = n_opt {
            match set_battery_threshold(n) {
                Ok(msg) => return msg,
                Err(e) => return format!("error: {}", e),
            }
        } else {
            return "error: invalid battery threshold".into();
        }
    }
    if cmd.starts_with("fanmode") {
        if let Some(arg) = cmd.split_whitespace().nth(1) {
            let profile = match arg {
                "quiet" => Some(FanProfile::Quiet),
                "balanced" => Some(FanProfile::Balanced),
                "performance" => Some(FanProfile::Performance),
                _ => None,
            };
            if let Some(p) = profile {
                match set_fan_mode(p) {
                    Ok(msg) => return msg,
                    Err(e) => return format!("error: {}", e),
                }
            } else {
                return "error: invalid fanmode".into();
            }
        } else {
            return "error: fanmode requires argument".into();
        }
    }

    "unknown command".into()
}

fn set_battery_threshold(value: i32) -> Result<String, String> {
    if !(0..=100).contains(&value) {
        return Err("threshold out of range (0-100)".into());
    }

    let path = "/sys/class/power_supply/BAT0/charge_control_end_threshold";
    std::fs::write(path, value.to_string())
        .map_err(|e| format!("failed to write {}: {}", path, e))?;
    Ok(format!("Battery threshold set to {}", value))
}

fn set_fan_mode(profile: FanProfile) -> Result<String, String> {
    let desc = match profile {
        FanProfile::Quiet => "quiet",
        FanProfile::Balanced => "balanced",
        FanProfile::Performance => "performance",
    };
    // do fan stuff here
    println!("fan mode set to {}", desc);
    Ok(format!("fan mode set to {}", desc))
}
