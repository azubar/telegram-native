use std::path::Path;

fn main() {
    if let Ok(td_lib_dir) = std::env::var("TD_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", td_lib_dir);
        let path = Path::new(&td_lib_dir);
        println!("cargo:rustc-link-search=native={}", path.join("Release").display());
        println!("cargo:rustc-link-search=native={}", path.join("Debug").display());
        println!("cargo:rustc-link-search=native={}", path.join("static").display());
    }

    if let Ok(vcpkg_lib_dir) = std::env::var("VCPKG_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", vcpkg_lib_dir);
    }

    if std::env::var("TD_STATIC").is_ok() {
        println!("cargo:rustc-link-lib=static=tdjson_static");
        println!("cargo:rustc-link-lib=static=tdclient");
        println!("cargo:rustc-link-lib=static=tdcore");
        println!("cargo:rustc-link-lib=static=tdapi");
        println!("cargo:rustc-link-lib=static=tdactor");
        println!("cargo:rustc-link-lib=static=tdutils");
        println!("cargo:rustc-link-lib=static=tdnet");
        println!("cargo:rustc-link-lib=static=tddb");
        println!("cargo:rustc-link-lib=static=tdsqlite");

        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
        if target_os == "windows" {
            println!("cargo:rustc-link-lib=ws2_32");
            println!("cargo:rustc-link-lib=crypt32");
            println!("cargo:rustc-link-lib=normaliz");
            println!("cargo:rustc-link-lib=psapi");
            println!("cargo:rustc-link-lib=user32");
            println!("cargo:rustc-link-lib=advapi32");
            println!("cargo:rustc-link-lib=libssl");
            println!("cargo:rustc-link-lib=libcrypto");
            println!("cargo:rustc-link-lib=zlib");
        } else if target_os == "linux" {
            println!("cargo:rustc-link-lib=ssl");
            println!("cargo:rustc-link-lib=crypto");
            println!("cargo:rustc-link-lib=z");
            println!("cargo:rustc-link-lib=stdc++");
        }
    } else {
        println!("cargo:rustc-link-lib=tdjson");
    }
}
