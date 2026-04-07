use std::env;
use std::path::{Path, PathBuf};

fn main() {
    napi_build::setup();

    let target = env::var("TARGET").unwrap();
    let is_windows = target.contains("windows");
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Cross-compilation toolchain file
    let toolchain_file = if target == "aarch64-unknown-linux-gnu" {
        Some(manifest_dir.join("cmake/aarch64-linux-gnu.cmake"))
    } else {
        None
    };

    // --- Build libde265 (HEVC decoder, static) ---
    let mut de265_cfg = cmake::Config::new("deps/libde265");
    de265_cfg
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_ENCODER", "OFF")
        .define("BUILD_EXAMPLES", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON");

    if let Some(ref tc) = toolchain_file {
        de265_cfg.define("CMAKE_TOOLCHAIN_FILE", tc.to_str().unwrap());
    }

    let de265_dst = de265_cfg.build();

    println!("cargo:rustc-link-search=native={}/lib", de265_dst.display());
    if is_windows {
        println!("cargo:rustc-link-lib=static=libde265");
    } else {
        println!("cargo:rustc-link-lib=static=de265");
    }

    // Find the actual libde265 library file
    let de265_lib = find_lib(&de265_dst, &["libde265.lib", "libde265.a", "de265.lib"]);

    // --- Build libheif (static, decode only) ---
    let mut heif_cfg = cmake::Config::new("deps/libheif");
    heif_cfg
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("WITH_EXAMPLES", "OFF")
        .define("WITH_GDK_PIXBUF", "OFF")
        .define("WITH_JPEG_DECODER", "OFF")
        .define("WITH_JPEG_ENCODER", "OFF")
        .define("BUILD_DOCUMENTATION", "OFF")
        .define("BUILD_TESTING", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define(
            "LIBDE265_INCLUDE_DIR",
            format!("{}/include", de265_dst.display()),
        )
        .define("LIBDE265_LIBRARY", &de265_lib);

    if is_windows {
        // Tell libheif that libde265 is a static library (avoid __declspec(dllimport))
        heif_cfg.cflag("-DLIBDE265_STATIC_BUILD");
        heif_cfg.cxxflag("-DLIBDE265_STATIC_BUILD");
    }

    if let Some(ref tc) = toolchain_file {
        heif_cfg.define("CMAKE_TOOLCHAIN_FILE", tc.to_str().unwrap());
    }

    let heif_dst = heif_cfg.build();

    println!("cargo:rustc-link-search=native={}/lib", heif_dst.display());
    println!("cargo:rustc-link-lib=static=heif");

    // C++ standard library
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
    // Windows: MSVC links the C++ runtime automatically
}

/// Search for a library file under `base/lib/`, trying multiple names.
fn find_lib(base: &Path, names: &[&str]) -> String {
    for name in names {
        let path = base.join("lib").join(name);
        if path.exists() {
            return path.to_str().unwrap().to_string();
        }
    }
    panic!(
        "Could not find library in {}/lib/. Tried: {:?}",
        base.display(),
        names
    );
}
