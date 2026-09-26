fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    if let Ok(library) = pkg_config::Config::new()
        .atleast_version("1.5.0")
        .probe("ibus-1.0")
    {
        let mut builder = bindgen::Builder::default()
            .header("wrapper.h")
            .allowlist_function("ibus_.*")
            .allowlist_type("IBus.*")
            .allowlist_var("IBUS_.*");

        for path in &library.include_paths {
            builder = builder.clang_arg(format!("-I{}", path.display()));
        }

        let bindings = builder
            .generate()
            .expect("Unable to generate IBus bindings");
        let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .unwrap();
    } else {
        // Fallback placeholder when dev headers are not installed during initial offline test
        println!(
            "cargo:warning=ibus-1.0 pkg-config not found, using fallback/dummy definitions if needed"
        );
    }
}
