use crate::error::*;

use std::io::{self, Write};

pub mod env;
pub mod sh;

static mut YES: bool = false;

/// Get the number of decimal digits in the given number.
pub fn digits(mut num: usize) -> usize {
    let mut res = 0;
    loop {
        res += 1;
        num /= 10;
        if num == 0 {
            break res;
        }
    }
}

/// Set the global 'yes' setting.
pub fn set_yes(yes: bool) {
    unsafe { YES = yes };
}

/// Query the global 'yes' setting.
pub fn yes() -> bool {
    unsafe { YES }
}

/// Present an interactive yes/no prompt.
pub fn prompt(
    prompt: &str,
    default: Option<bool>,
    yes_response: Option<&str>,
    no_response: Option<&str>,
) -> Result<bool> {
    if yes() {
        Ok(true)
    } else {
        let yn = match default {
            None => "[y/n]",
            Some(true) => "[Y/n]",
            Some(false) => "[y/N]",
        };

        let mut input = String::new();
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        let res = loop {
            print!("{} {} ", prompt, yn);
            stdout.flush()?;
            stdin.read_line(&mut input)?;

            if input.trim().is_empty() {
                if let Some(def) = default {
                    break Ok::<_, Error>(def);
                }
            } else {
                match input.to_lowercase().trim() {
                    "y" | "yes" => {
                        break Ok(true);
                    }

                    "n" | "no" => {
                        break Ok(false);
                    }

                    _ => {}
                }
            }

            input.clear();
        }?;

        if res {
            if let Some(s) = yes_response {
                println!("{}", s);
            }
        } else {
            if let Some(s) = no_response {
                println!("{}", s);
            }
        }

        Ok(res)
    }
}
