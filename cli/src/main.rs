use std::env;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;

enum FanProfile {
    Quiet,
    Balanced,
    Performance,
}

impl FanProfile {
    fn as_str(&self) -> &str {
        match self {
            FanProfile::Quiet => "quiet",
            FanProfile::Balanced => "balanced",
            FanProfile::Performance => "performance",
        }
    }
}

enum Command {
    BatteryThreshold(i32),
    FanMode(FanProfile),
}

impl Command {
    fn variants() -> &'static [&'static str] {
        &[
            "battery-threshold <num>",
            "fanmode <quiet|balanced|performance>",
        ]
    }

    fn parse(input: &str) -> Result<Command, ()> {
        let mut parts = input.split_whitespace();
        let first = parts.next().unwrap_or("");

        if first.starts_with("battery-threshold") {
            let suffix = &first["battery-threshold".len()..];
            if !suffix.is_empty() {
                if let Ok(n) = suffix.parse::<i32>() {
                    return Ok(Command::BatteryThreshold(n));
                } else {
                    return Err(());
                }
            }
            if let Some(nstr) = parts.next() {
                if let Ok(n) = nstr.parse::<i32>() {
                    return Ok(Command::BatteryThreshold(n));
                }
            }
            return Err(());
        }
        if first == "fanmode" {
            if let Some(arg) = parts.next() {
                let profile = match arg {
                    "quiet" => FanProfile::Quiet,
                    "balanced" => FanProfile::Balanced,
                    "performance" => FanProfile::Performance,
                    _ => return Err(()),
                };
                return Ok(Command::FanMode(profile));
            }
            return Err(());
        }

        Err(())
    }

    fn to_string(&self) -> String {
        match self {
            Command::BatteryThreshold(n) => format!("battery-threshold {}", n),
            Command::FanMode(profile) => format!("fanmode {}", profile.as_str()),
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
