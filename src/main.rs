use rovex_core::{EntryKind, FileSystem};
use std::env;
use std::path::PathBuf;

fn main() {
    let path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

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
        }
        Err(error) => {
            eprintln!("erro: {error}");
            std::process::exit(1);
        }
    }
}
