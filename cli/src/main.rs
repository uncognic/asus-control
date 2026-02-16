use std::env;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;

enum PlatformProfile {
    Quiet,
    Balanced,
    Performance,
}

impl PlatformProfile {
    fn as_str(&self) -> &str {
        match self {
            PlatformProfile::Quiet => "quiet",
            PlatformProfile::Balanced => "balanced",
            PlatformProfile::Performance => "performance",
        }
    }
}

enum Command {
    SetBatteryThreshold(i32),
    SetProfile(PlatformProfile),
    GetBatteryThreshold,
    GetProfile,
}

impl Command {
    fn variants() -> &'static [&'static str] {
        &[
            "set battery-threshold <num>",
            "set profile <quiet|balanced|performance>",
            "get profile",
            "get battery-threshold",
        ]
    }

    fn parse(input: &str) -> Result<Command, ()> {
        let mut parts = input.split_whitespace();
        let verb = parts.next().unwrap_or("");

        match verb {
            "set" => match parts.next() {
                Some("battery-threshold") => {
                    if let Some(nstr) = parts.next() {
                        if let Ok(n) = nstr.parse::<i32>() {
                            return Ok(Command::SetBatteryThreshold(n));
                        }
                    }
                    Err(())
                }
                Some("profile") => {
                    if let Some(arg) = parts.next() {
                        let profile = match arg {
                            "quiet" => PlatformProfile::Quiet,
                            "balanced" => PlatformProfile::Balanced,
                            "performance" => PlatformProfile::Performance,
                            _ => return Err(()),
                        };
                        return Ok(Command::SetProfile(profile));
                    }
                    Err(())
                }
                _ => Err(()),
            },
            "get" => match parts.next() {
                Some("battery-threshold") => Ok(Command::GetBatteryThreshold),
                Some("profile") => Ok(Command::GetProfile),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }

    fn to_string(&self) -> String {
        match self {
            Command::SetBatteryThreshold(n) => format!("set battery-threshold {}", n),
            Command::SetProfile(profile) => format!("set profile {}", profile.as_str()),
            Command::GetBatteryThreshold => "get battery-threshold".into(),
            Command::GetProfile => "get profile".into(),
        }
    }
}

fn main() -> io::Result<()> {
    let input = {
        let args = env::args().skip(1).collect::<Vec<_>>();
        if !args.is_empty() {
            args.join(" ")
        } else {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            while buf.ends_with('\n') || buf.ends_with('\r') {
                buf.pop();
            }
            buf
        }
    };

    if input.is_empty() {
        eprintln!(
            "Usage: {} <command> or provide command on stdin",
            env::args().next().unwrap_or_else(|| "asus-control".into())
        );
        std::process::exit(2);
    }

    let cmd = match Command::parse(&input) {
        Ok(c) => c,
        Err(_) => {
            let first_token = input.split_whitespace().next().unwrap_or("");
            eprintln!("Invalid command: {}", first_token);
            eprintln!("Valid commands: {}", Command::variants().join(", "));
            std::process::exit(2);
        }
    };

    let mut stream = UnixStream::connect("/run/asus-control-daemon.sock")?;
    stream.write_all(cmd.to_string().as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("{}", response);

    Ok(())
}
