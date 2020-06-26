fn main() {
    let mut dst = cmake::Config::new("dataviewer").build();
    dst.push("lib");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=dataviewer");
    println!("cargo:rustc-link-lib=stdc++");

    pkg_config::probe_library("Qt5Core").unwrap();
    pkg_config::probe_library("Qt5Quick").unwrap();
    pkg_config::probe_library("Qt5Multimedia").unwrap();
    pkg_config::probe_library("glu").unwrap();

    for entry in walkdir::WalkDir::new("dataviewer")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }
}
