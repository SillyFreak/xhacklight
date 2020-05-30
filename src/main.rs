use std::{env, fs, process};

use anyhow::{Context, Result};

const BRIGHTNESS_FILE: &'static str = "/sys/class/backlight/intel_backlight/brightness";

fn usage() -> ! {
    eprintln!("Usage: xhacklight [=N|+N|-N|inc|dec]");
    process::exit(1);
}

fn get_brightness() -> Result<u32> {
    let brightness = fs::read_to_string(BRIGHTNESS_FILE)
        .context(format!("could not read file {}", BRIGHTNESS_FILE))?;
    let brightness = brightness
        .trim()
        .parse::<u32>()
        .context(format!("could not parse {} as a number", brightness))?;
    Ok(brightness)
}

fn set_brightness(brightness: u32) -> Result<()> {
    let brightness = if brightness <= 60000 { brightness } else { 60000 };
    let brightness = brightness.to_string();
    fs::write(BRIGHTNESS_FILE, brightness)
        .context(format!("could not write file {}", BRIGHTNESS_FILE))?;
    Ok(())
}

enum Adjustment {
    Set(u32),
    Inc(u32),
    Dec(u32),
    SmartInc,
    SmartDec,
}

fn adjust_brightness(adjustment: Adjustment) -> Result<()> {
    let brightness = if let Adjustment::Set(brightness) = adjustment {
        brightness
    } else {
        let brightness = get_brightness()?;
        match adjustment {
            Adjustment::Set(_) => {
                // we handled this before already,
                // so that reading the brightness is avoided
                panic!("unreachable");
            }
            Adjustment::Inc(change) => brightness.saturating_add(change),
            Adjustment::Dec(change) => brightness.saturating_sub(change),
            Adjustment::SmartInc => {
                let change = match brightness {
                    0..=199 => 50,
                    200..=1999 => 200,
                    2000..=19999 => 2000,
                    _ => 5000,
                };
                brightness.saturating_add(change)
            }
            Adjustment::SmartDec => {
                let change = match brightness {
                    0..=200 => 50,
                    201..=2000 => 200,
                    2001..=20000 => 2000,
                    _ => 5000,
                };
                brightness.saturating_sub(change)
            }
        }
    };

    set_brightness(brightness)?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();

    match args.len() {
        1 => {
            let brightness = get_brightness()?;
            println!("{}", brightness);
        }
        2 => {
            let adjustment = match args[1].as_str() {
                "inc" => Adjustment::SmartInc,
                "dec" => Adjustment::SmartDec,
                "" => usage(),
                arg => {
                    let (mode, number) = arg.split_at(1);
                    let number = number.parse::<u32>();
                    match mode {
                        "=" => Adjustment::Set(number?),
                        "+" => Adjustment::Inc(number?),
                        "-" => Adjustment::Dec(number?),
                        _ => usage(),
                    }
                }
            };
            adjust_brightness(adjustment)?;
        }
        _ => usage(),
    }

    Ok(())
}
