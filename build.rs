fn main() {
    println!("cargo:rerun-if-changed=ui/main.slint");
    println!("cargo:rerun-if-changed=ui/design_tokens.slint");
    println!("cargo:rerun-if-changed=ui/components.slint");
    println!("cargo:rerun-if-changed=ui/data.slint");
    println!("cargo:rerun-if-changed=ui/overlays.slint");
    println!("cargo:rerun-if-changed=assets/rovex.ico");
    if let Err(error) = slint_build::compile("ui/main.slint") {
        eprintln!("falha ao compilar a interface Slint: {error}");
        std::process::exit(1);
    }

    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut resource = winres::WindowsResource::new();
        resource.set_icon("assets/rovex.ico");
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") && cfg!(unix) {
            let prefix = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
                Ok("x86") => "i686-w64-mingw32-",
                _ => "x86_64-w64-mingw32-",
            };
            resource
                .set_windres_path(&format!("{prefix}windres"))
                .set_ar_path(&format!("{prefix}ar"));
        }
        if let Err(error) = resource.compile() {
            eprintln!("falha ao embutir o ícone Windows: {error}");
            std::process::exit(1);
        }
    }
}
