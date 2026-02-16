use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[derive(Debug)]
enum PlatformProfile {
    Quiet,
    Balanced,
    Performance,
}

fn main() -> std::io::Result<()> {
    let socket_path = "/run/asus-control-daemon.sock";

    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;
    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o660))?;
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
    let mut parts = cmd.split_whitespace();
    let verb = match parts.next() {
        Some(v) => v,
        None => return "error: empty command".into(),
    };

    match verb {
        "set" => {
            match parts.next() {
                Some("battery-threshold") => {
                    if let Some(arg) = parts.next() {
                        if let Ok(n) = arg.parse::<i32>() {
                            return match set_battery_threshold(n) {
                                Ok(m) => m,
                                Err(e) => format!("error: {}", e),
                            };
                        } else {
                            return "error: invalid battery threshold".into();
                        }
                    }
                    return "error: battery-threshold requires a value".into();
                }
                Some("profile") => {
                    if let Some(arg) = parts.next() {
                        let profile = match arg {
                            "quiet" => Some(PlatformProfile::Quiet),
                            "balanced" => Some(PlatformProfile::Balanced),
                            "performance" => Some(PlatformProfile::Performance),
                            _ => None,
                        };
                        if let Some(p) = profile {
                            return match set_fan_mode(p) {
                                Ok(m) => m,
                                Err(e) => format!("error: {}", e),
                            };
                        } else {
                            return "error: invalid profile".into();
                        }
                    }
                    return "error: profile requires an argument".into();
                }
                Some(other) => return format!("error: unknown set target: {}", other),
                None => return "error: set requires a target".into(),
            }
        }
        "get" => {
            match parts.next() {
                Some("battery-threshold") => {
                    return match get_battery_threshold() {
                        Ok(m) => m,
                        Err(e) => format!("error: {}", e),
                    };
                }
                Some("profile") => {
                    return match get_fan_profile() {
                        Ok(m) => m,
                        Err(e) => format!("error: {}", e),
                    };
                }
                Some(other) => return format!("error: unknown get target: {}", other),
                None => return "error: get requires a target".into(),
            }
        }
        _ => {
            return "error: unknown command - use 'get' or 'set'".into();
        }
    }
}

fn get_battery_threshold() -> Result<String, String> {
    let path = "/sys/class/power_supply/BAT0/charge_control_end_threshold";
    let s = std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
    Ok(format!("battery-threshold {}", s.trim()))
}

fn get_fan_profile() -> Result<String, String> {
    let path = "/sys/firmware/acpi/platform_profile";
    let s = std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
    Ok(format!("profile {}", s.trim()))
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

fn set_fan_mode(profile: PlatformProfile) -> Result<String, String> {
    let desc = match profile {
        PlatformProfile::Quiet => "quiet",
        PlatformProfile::Balanced => "balanced",
        PlatformProfile::Performance => "performance",
    };

    let path = "/sys/firmware/acpi/platform_profile";
    std::fs::write(path, desc)
        .map_err(|e| format!("failed to write {}: {}", path, e))?;

    Ok(format!("Profile set to {}", desc))
}
