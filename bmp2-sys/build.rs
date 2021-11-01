fn main() {
    cc::Build::new()
        .file("native/bmp2.c")
        .warnings_into_errors(true)
        .compile("bmp2");

    let bindings = bindgen::Builder::default()
        .header("native/bmp2.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
