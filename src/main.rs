#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use rovex_core::{EntryKind, FileSystem};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

fn run_cli(path: PathBuf) -> i32 {
    println!(
        "Rovex core — listagem de desenvolvimento: {}",
        path.display()
    );
    match FileSystem.list_directory(&path) {
        Ok(entries) => {
            for entry in entries {
                let marker = match entry.kind {
                    EntryKind::Directory => "[DIR]",
                    EntryKind::File => "[FILE]",
                    EntryKind::Symlink => "[LINK]",
                    EntryKind::Other => "[OTHER]",
                };
                println!("{marker:>6} {}", entry.display_name());
            }
            0
        }
        Err(error) => {
            eprintln!("erro: {error}");
            1
        }
    }
}

fn main() {
    let mut args = env::args_os();
    let first = args.nth(1);

    if first.as_deref() == Some(OsString::from("--cli").as_os_str()) {
        let path = args
            .next()
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        std::process::exit(run_cli(path));
    }

    if let Err(error) = rovex_core::desktop::run() {
        eprintln!("falha ao iniciar a interface: {error}");
        std::process::exit(1);
    }
}
