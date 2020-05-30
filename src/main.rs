use std::{env, fs, process};

use anyhow::{Context, Result};

const BRIGHTNESS_FILE: &'static str = "/sys/class/backlight/intel_backlight/brightness";

const SMART_STEPS: &'static [u32] = &[
    0, 1, 100, // low end
    200, 300, 400, // 100 steps
    600, 800, 1000, 1200, 1400, 1600, 1800, 2000, // 200 steps
    3000, 4000, 5000, 6000, // 1000 steps
    8000, 10000, 12000, 14000, 16000, 18000, 20000, // 2000 steps
    25000, 30000, 35000, 40000, 45000, 50000, 55000, 60000, // 5000 steps
];

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
    let brightness = if brightness <= 60000 {
        brightness
    } else {
        60000
    };
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
                let index = SMART_STEPS.binary_search(&brightness);
                let index = index.map_or_else(
                    // the insert index is to the right of the current value
                    |insert_index| insert_index,
                    // take the index of the value to the right
                    |index| index + 1,
                );
                // make sure that the index is not out of bounds
                let index = std::cmp::min(index, SMART_STEPS.len() - 1);
                SMART_STEPS[index]
            }
            Adjustment::SmartDec => {
                let index = SMART_STEPS.binary_search(&brightness);
                let index = index.map_or_else(
                    // the insert index is to the right of the current value; go one to the left
                    |insert_index| insert_index.saturating_sub(1),
                    // take the index of the value to the left
                    |index| index.saturating_sub(1),
                );
                SMART_STEPS[index]
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
