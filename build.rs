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
        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

        if let Ok(td_lib_dir) = std::env::var("TD_LIB_DIR") {
            let p = std::path::Path::new(&td_lib_dir);
            if let Ok(entries) = std::fs::read_dir(p) {
                let mut libs = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "lib" || ext == "a" {
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    let lib_name = if stem.starts_with("lib") && ext == "a" {
                                        &stem[3..]
                                    } else {
                                        stem
                                    };
                                    // On Windows, "tdjson.lib" is an import library for tdjson.dll.
                                    // Exclude "tdjson" so cargo links tdjson_static (and all static libs) instead!
                                    if target_os == "windows" && lib_name == "tdjson" {
                                        continue;
                                    }
                                    libs.push(lib_name.to_string());
                                }
                            }
                        }
                    }
                }
                libs.sort();
                libs.dedup();
                
                // Repeat link passes for Windows MSVC to resolve circular dependencies between static libraries
                let passes = if target_os == "windows" { 2 } else { 1 };
                for _ in 0..passes {
                    for lib in &libs {
                        println!("cargo:rustc-link-lib=static={}", lib);
                    }
                }
            }
        } else {
            println!("cargo:rustc-link-lib=static=tdjson_static");
            println!("cargo:rustc-link-lib=static=tdclient");
            println!("cargo:rustc-link-lib=static=tdcore");
            println!("cargo:rustc-link-lib=static=tdapi");
            println!("cargo:rustc-link-lib=static=tdactor");
            println!("cargo:rustc-link-lib=static=tdutils");
            println!("cargo:rustc-link-lib=static=tdnet");
            println!("cargo:rustc-link-lib=static=tddb");
            println!("cargo:rustc-link-lib=static=tdsqlite");
        }

        if target_os == "windows" {
            println!("cargo:rustc-link-lib=ws2_32");
            println!("cargo:rustc-link-lib=crypt32");
            println!("cargo:rustc-link-lib=normaliz");
            println!("cargo:rustc-link-lib=psapi");
            println!("cargo:rustc-link-lib=user32");
            println!("cargo:rustc-link-lib=advapi32");
            println!("cargo:rustc-link-lib=shell32");

            if let Ok(vcpkg_dir) = std::env::var("VCPKG_LIB_DIR") {
                let p = std::path::Path::new(&vcpkg_dir);

                // Auto alias zlibstatic <-> zlib in VCPKG_LIB_DIR and TD_LIB_DIR
                let zs = p.join("zlibstatic.lib");
                let zl = p.join("zlib.lib");

                if zs.exists() {
                    let _ = std::fs::copy(&zs, p.join("zlib.lib"));
                }
                if zl.exists() {
                    let _ = std::fs::copy(&zl, p.join("zlibstatic.lib"));
                }

                if let Ok(td_lib_dir) = std::env::var("TD_LIB_DIR") {
                    let tdp = std::path::Path::new(&td_lib_dir);
                    if zs.exists() {
                        let _ = std::fs::copy(&zs, tdp.join("zlib.lib"));
                        let _ = std::fs::copy(&zs, tdp.join("zlibstatic.lib"));
                    }
                    if zl.exists() {
                        let _ = std::fs::copy(&zl, tdp.join("zlib.lib"));
                        let _ = std::fs::copy(&zl, tdp.join("zlibstatic.lib"));
                    }
                }

                if p.join("libssl.lib").exists() {
                    println!("cargo:rustc-link-lib=static=libssl");
                } else if p.join("ssl.lib").exists() {
                    println!("cargo:rustc-link-lib=static=ssl");
                } else {
                    println!("cargo:rustc-link-lib=static=libssl");
                }

                if p.join("libcrypto.lib").exists() {
                    println!("cargo:rustc-link-lib=static=libcrypto");
                } else if p.join("crypto.lib").exists() {
                    println!("cargo:rustc-link-lib=static=crypto");
                } else {
                    println!("cargo:rustc-link-lib=static=libcrypto");
                }

                let has_zs = p.join("zlibstatic.lib").exists();
                let has_zl = p.join("zlib.lib").exists();
                if has_zs {
                    println!("cargo:rustc-link-lib=static=zlibstatic");
                }
                if has_zl {
                    println!("cargo:rustc-link-lib=static=zlib");
                }
                if !has_zs && !has_zl {
                    println!("cargo:rustc-link-lib=static=zlib");
                }
            } else {
                println!("cargo:rustc-link-lib=static=libssl");
                println!("cargo:rustc-link-lib=static=libcrypto");
                println!("cargo:rustc-link-lib=static=zlibstatic");
            }
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
