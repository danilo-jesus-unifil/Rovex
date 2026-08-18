fn main() {
    println!("cargo:rerun-if-changed=ui/main.slint");
    println!("cargo:rerun-if-changed=ui/design_tokens.slint");
    println!("cargo:rerun-if-changed=ui/components.slint");
    println!("cargo:rerun-if-changed=ui/data.slint");
    println!("cargo:rerun-if-changed=ui/overlays.slint");
    println!("cargo:rerun-if-changed=ui/toolbars.slint");
    println!("cargo:rerun-if-changed=assets/rovex.ico");
    println!("cargo:rerun-if-changed=assets/rovex.manifest");
    if let Err(error) = slint_build::compile("ui/main.slint") {
        eprintln!("falha ao compilar a interface Slint: {error}");
        std::process::exit(1);
    }

    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut resource = winres::WindowsResource::new();
        resource
            .set_icon("assets/rovex.ico")
            .set_manifest_file("assets/rovex.manifest");
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
            eprintln!("falha ao embutir os recursos Windows: {error}");
            std::process::exit(1);
        }
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
            let resource_object = std::path::Path::new(
                &std::env::var("OUT_DIR").expect("OUT_DIR deve existir no build script"),
            )
            .join("resource.o");
            println!(
                "cargo:rustc-link-arg-bin=rovex={}",
                resource_object.display()
            );
        }
    }
}
