fn main() {
    let mut dst = cmake::Config::new("native").build();
    dst.push("lib");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=sensorevalgui_native");
    println!("cargo:rustc-link-lib=stdc++");

    pkg_config::probe_library("Qt5Core").unwrap();
    pkg_config::probe_library("Qt5Quick").unwrap();
    pkg_config::probe_library("Qt5Multimedia").unwrap();
    pkg_config::probe_library("glu").unwrap();

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    for entry in walkdir::WalkDir::new("native")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }
    println!("cargo:rerun-if-changed=wrapper.h");
}
