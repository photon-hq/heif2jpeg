use std::env;

fn main() {
    napi_build::setup();

    let target = env::var("TARGET").unwrap();
    let is_windows = target.contains("windows");

    // For cross-compilation, the cmake crate reads CC/CXX env vars
    // but they need to be set with the target-specific prefix.
    // We handle this by setting CMAKE_C_COMPILER/CMAKE_CXX_COMPILER directly.
    let (c_compiler, cxx_compiler) = if target == "aarch64-unknown-linux-gnu" {
        (
            Some("aarch64-linux-gnu-gcc"),
            Some("aarch64-linux-gnu-g++"),
        )
    } else {
        (None, None)
    };

    // --- Build libde265 (HEVC decoder, static) ---
    let mut de265_cfg = cmake::Config::new("deps/libde265");
    de265_cfg
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_ENCODER", "OFF")
        .define("BUILD_EXAMPLES", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON");
    if let Some(cc) = c_compiler {
        de265_cfg.define("CMAKE_C_COMPILER", cc);
    }
    if let Some(cxx) = cxx_compiler {
        de265_cfg.define("CMAKE_CXX_COMPILER", cxx);
    }
    let de265_dst = de265_cfg.build();

    println!("cargo:rustc-link-search=native={}/lib", de265_dst.display());
    println!("cargo:rustc-link-lib=static=de265");

    // --- Build libheif (static, decode only) ---
    let lib_ext = if is_windows { "lib" } else { "a" };
    let lib_prefix = if is_windows { "" } else { "lib" };

    let mut heif_cfg = cmake::Config::new("deps/libheif");
    heif_cfg
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("WITH_EXAMPLES", "OFF")
        .define("WITH_GDK_PIXBUF", "OFF")
        .define("WITH_JPEG_DECODER", "OFF")
        .define("WITH_JPEG_ENCODER", "OFF")
        .define("BUILD_DOCUMENTATION", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define(
            "LIBDE265_INCLUDE_DIR",
            format!("{}/include", de265_dst.display()),
        )
        .define(
            "LIBDE265_LIBRARY",
            format!(
                "{}/lib/{prefix}de265.{ext}",
                de265_dst.display(),
                prefix = lib_prefix,
                ext = lib_ext
            ),
        );
    if let Some(cc) = c_compiler {
        heif_cfg.define("CMAKE_C_COMPILER", cc);
    }
    if let Some(cxx) = cxx_compiler {
        heif_cfg.define("CMAKE_CXX_COMPILER", cxx);
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
