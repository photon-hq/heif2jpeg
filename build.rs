use std::env;
use std::path::Path;

fn main() {
    napi_build::setup();

    let target = env::var("TARGET").unwrap();
    let is_windows = target.contains("windows");
    let is_cross_linux_arm64 = target == "aarch64-unknown-linux-gnu";

    // --- Build libde265 (HEVC decoder, static) ---
    let mut de265_cfg = cmake::Config::new("deps/libde265");
    de265_cfg
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_ENCODER", "OFF")
        .define("BUILD_EXAMPLES", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON");

    if is_cross_linux_arm64 {
        de265_cfg
            .define("CMAKE_C_COMPILER", "aarch64-linux-gnu-gcc")
            .define("CMAKE_CXX_COMPILER", "aarch64-linux-gnu-g++")
            .define("CMAKE_SYSTEM_NAME", "Linux")
            .define("CMAKE_SYSTEM_PROCESSOR", "aarch64");
    }

    let de265_dst = de265_cfg.build();

    println!("cargo:rustc-link-search=native={}/lib", de265_dst.display());
    if is_windows {
        println!("cargo:rustc-link-lib=static=libde265");
    } else {
        println!("cargo:rustc-link-lib=static=de265");
    }

    // Find the actual libde265 library file for passing to libheif
    let de265_lib = if is_windows {
        format!("{}/lib/libde265.lib", de265_dst.display())
    } else {
        format!("{}/lib/libde265.a", de265_dst.display())
    };

    // Sanity check
    if !Path::new(&de265_lib).exists() {
        // Try alternate location
        let alt = format!("{}/lib/de265.lib", de265_dst.display());
        if Path::new(&alt).exists() {
            eprintln!("cargo:warning=Using alternate libde265 path: {}", alt);
        } else {
            panic!(
                "libde265 library not found at {} or {}",
                de265_lib, alt
            );
        }
    }

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
        .define("WITH_REDUCED_VISIBILITY", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define(
            "LIBDE265_INCLUDE_DIR",
            format!("{}/include", de265_dst.display()),
        )
        .define("LIBDE265_LIBRARY", &de265_lib);

    if is_cross_linux_arm64 {
        heif_cfg
            .define("CMAKE_C_COMPILER", "aarch64-linux-gnu-gcc")
            .define("CMAKE_CXX_COMPILER", "aarch64-linux-gnu-g++")
            .define("CMAKE_SYSTEM_NAME", "Linux")
            .define("CMAKE_SYSTEM_PROCESSOR", "aarch64");
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
