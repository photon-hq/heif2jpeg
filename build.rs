fn main() {
    napi_build::setup();

    let is_windows = cfg!(target_os = "windows");

    // --- Build libde265 (HEVC decoder, static) ---
    let de265_dst = cmake::Config::new("deps/libde265")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_ENCODER", "OFF")
        .define("BUILD_EXAMPLES", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", de265_dst.display());
    println!("cargo:rustc-link-lib=static=de265");

    // --- Build libheif (static, decode only) ---
    let lib_ext = if is_windows { "lib" } else { "a" };
    let lib_prefix = if is_windows { "" } else { "lib" };

    let heif_dst = cmake::Config::new("deps/libheif")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("WITH_EXAMPLES", "OFF")
        .define("WITH_GDK_PIXBUF", "OFF")
        .define("WITH_JPEG_DECODER", "OFF")
        .define("WITH_JPEG_ENCODER", "OFF")
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
        )
        .build();

    println!("cargo:rustc-link-search=native={}/lib", heif_dst.display());
    println!("cargo:rustc-link-lib=static=heif");

    // C++ standard library
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
    // Windows: MSVC links the C++ runtime automatically
}
