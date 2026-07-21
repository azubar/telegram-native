fn main() {
    if let Ok(td_lib_dir) = std::env::var("TD_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", td_lib_dir);
    }

    let link_static = std::env::var("TD_STATIC").is_ok();
    if link_static {
        println!("cargo:rustc-link-lib=static=tdjson_static");
        println!("cargo:rustc-link-lib=static=tdclient");
        println!("cargo:rustc-link-lib=static=tdcore");
        println!("cargo:rustc-link-lib=static=tdapi");
        println!("cargo:rustc-link-lib=static=tdactor");
        println!("cargo:rustc-link-lib=static=tdutils");
    } else {
        println!("cargo:rustc-link-lib=tdjson");
    }
}
