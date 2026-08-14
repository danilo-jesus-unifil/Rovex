fn main() {
    println!("cargo:rerun-if-changed=ui/main.slint");
    if let Err(error) = slint_build::compile("ui/main.slint") {
        eprintln!("falha ao compilar a interface Slint: {error}");
        std::process::exit(1);
    }
}
