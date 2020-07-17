use regex::Regex;
use std::path::{Path, PathBuf};

pub mod args;
mod term_utils;

use term_utils::{ask_for_confirmation, log};

pub fn run(opts: args::Options) -> Result<(), String> {
    if opts.force && opts.interactive {
        return Err(
            "Received --force and --interactive. Not sure how to continue. Exiting.".to_string(),
        );
    }

    let patterns = {
        let mut p = Vec::with_capacity(1 + opts.patterns.len());

        let from = Regex::new(&opts.pattern.0).expect("Invalid regex.");

        p.push((from, &opts.pattern.1));
        p.extend(
            opts.patterns
                .iter()
                .map(|(f, v)| (Regex::new(f).expect("Invalid regex."), v)),
        );

        p
    };

    for path in &opts.files {
        if path.is_file() {
            let renamed = get_renamed_path(path, opts.global, &patterns);

            if path == &renamed {
                if opts.verbose {
                    log(opts.dry_run, format!("No patterns match {:?}", path));
                }
                continue;
            }

            if renamed.is_dir() {
                return Err(format!(
                    "Cannot rename {:?}. {:?} is already a directory.",
                    path, renamed
                ));
            }

            if renamed.is_file() {
                if opts.interactive {
                    if !ask_for_confirmation(format!("Overwrite {:?}?", renamed)) {
                        continue;
                    }
                } else if !opts.force {
                    return Err(format!(
                        "Not overwriting {:?} without --interactive or --force",
                        renamed,
                    ));
                }
            }

            if opts.verbose || opts.dry_run {
                log(opts.dry_run, format!("{:?} -> {:?}", path, renamed));
            }

            if !opts.dry_run {
                std::fs::rename(path, renamed).expect("Failed to rename file");
            }
        } else if opts.ignore_dir {
            if opts.verbose {
                log(opts.dry_run, format!("Ignoring directory {:?}", path));
            }
        } else {
            let current = std::env::current_dir().unwrap().join(path);
            return Err(format!(
                "{:?} is not a file. If this is intentional, pass --ignore-dir.",
                current
            ));
        }
    }

    Ok(())
}

pub fn get_renamed_path<P: AsRef<Path>>(
    path: P,
    replace_all: bool,
    patterns: &[(Regex, &String)],
) -> PathBuf {
    let file = path.as_ref().file_name().unwrap();
    let dir = path.as_ref().parent();
    let mut filename = file.to_str().unwrap().to_string(); // There's got to be a nicer way to do this.

    let replace = if replace_all {
        Regex::replace_all
    } else {
        Regex::replace
    };

    for (regex, replacement) in patterns {
        filename = replace(regex, &filename, replacement.as_str()).to_string();
    }

    dir.unwrap().join(filename)
}
