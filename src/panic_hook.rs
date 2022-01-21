use std::env::consts::{ARCH, FAMILY, OS};
use std::fmt::Write as StrWrite;
use std::fs::{create_dir_all, rename, OpenOptions};
use std::io::{stderr, Write as IoWrite};
use std::panic;
use std::panic::PanicInfo;
use std::path::PathBuf;

use backtrace::Backtrace;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::utilities;
use crate::utilities::{PKG_NAME, PKG_VERSION};

pub const CRASH_LOG_FILE_NAME: &str = "crash.log";

pub fn install() {
    panic::set_hook(Box::new(|info: &PanicInfo| {
        if let Err(e) = report_crash(info) {
            eprintln!("Panic hook failure: {}.", e);
        }
    }));
}

pub fn archive_last_crash() -> Option<PathBuf> {
    if let Some(dirs) = utilities::project_dirs() {
        let mut path = dirs.data_local_dir().to_path_buf();
        path.push(CRASH_LOG_FILE_NAME);
        if path.exists() {
            let archive_name = format!(
                "{}.{}",
                CRASH_LOG_FILE_NAME,
                OffsetDateTime::now_utc().format(&Rfc3339).ok()?
            );
            let log_path = path.clone();
            path.pop();
            path.push(archive_name);
            rename(log_path, &path).unwrap();
            return Some(path);
        }
    }
    None
}

fn report_crash(info: &PanicInfo) -> std::fmt::Result {
    let mut msg = String::new();

    writeln!(msg)?;
    writeln!(msg)?;
    writeln!(
        msg,
        "Panic in {} {}, {}.",
        PKG_NAME,
        PKG_VERSION,
        OffsetDateTime::now_utc()
    )?;
    writeln!(msg, "System: {} ({}), {}.", OS, FAMILY, ARCH)?;

    if let Some(payload) = info.payload().downcast_ref::<&str>() {
        writeln!(msg, "Cause: {}", payload)?;
    }

    if let Some(location) = info.location() {
        writeln!(msg, "Location: {}.", location)?;
    }

    writeln!(msg)?;
    writeln!(msg, "{:#?}", Backtrace::new())?;

    if let Some(dirs) = utilities::project_dirs() {
        let mut path = dirs.data_local_dir().to_path_buf();
        if let Err(e) = create_dir_all(&path) {
            writeln!(
                msg,
                "[PANIC] Failed to create crash report directory {:?}: {}.",
                path, e
            )?;
        }
        path.push(CRASH_LOG_FILE_NAME);
        let open_result = OpenOptions::new().create(true).append(true).open(path);
        match open_result {
            Ok(mut log_writer) => {
                if let Err(e) = log_writer.write(msg.as_bytes()) {
                    writeln!(
                        msg,
                        "[PANIC] Failed to write panic message to crash.log: {}.",
                        e
                    )?;
                }
            }
            Err(e) => writeln!(msg, "[PANIC] Failed to open crash.log for writing: {}.", e)?,
        }
    }

    // TUI libraries may place the terminal in a mode where line breaks don't reset the column
    // position. So before printing to stderr, we have to replace all newlines with
    // carriage-return + newline.
    let msg = msg.replace("\n", "\r\n");
    if let Err(e) = stderr().write_all(msg.as_bytes()) {
        eprintln!("Failed to write panic message to stderr: {}.", e);
    }
    Ok(())
}
