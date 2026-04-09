use std::env;
use std::path::Path;

fn main() {
    napi_build::setup();

    let target = env::var("TARGET").unwrap();
    let is_windows = target.contains("windows");

    // --- Build libde265 (HEVC decoder, static) ---
    let de265_dst = cmake::Config::new("deps/libde265")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_ENCODER", "OFF")
        .define("BUILD_EXAMPLES", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .build();

    // Find the actual libde265 library file (needed by libheif cmake)
    let de265_lib = find_lib(&de265_dst, &["libde265.lib", "libde265.a"]);

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
        .define("CMAKE_DISABLE_FIND_PACKAGE_JPEG", "ON")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define(
            "LIBDE265_INCLUDE_DIR",
            format!("{}/include", de265_dst.display()),
        )
        .define("LIBDE265_LIBRARY", &de265_lib);

    if is_windows {
        // Tell libheif that libde265 is static (avoid __declspec(dllimport))
        heif_cfg.cflag("-DLIBDE265_STATIC_BUILD");
        heif_cfg.cxxflag("-DLIBDE265_STATIC_BUILD");
    }

    let heif_dst = heif_cfg.build();

    // --- Link order matters: heif depends on de265, so heif must come first ---
    // The linker resolves symbols left-to-right. If de265 comes first, the linker
    // doesn't yet know what symbols heif will need and may not pull them from the
    // static archive. Putting heif first ensures its unresolved symbols (like
    // de265_flush_data) are satisfied when de265 is processed next.
    println!("cargo:rustc-link-search=native={}/lib", heif_dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", heif_dst.display());
    println!("cargo:rustc-link-lib=static=heif");

    println!("cargo:rustc-link-search=native={}/lib", de265_dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", de265_dst.display());
    if is_windows {
        println!("cargo:rustc-link-lib=static=libde265");
    } else {
        println!("cargo:rustc-link-lib=static=de265");
    }

    // C++ standard library
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
}

/// Search for a library file under `base/lib/` or `base/lib64/`.
fn find_lib(base: &Path, names: &[&str]) -> String {
    for dir in &["lib", "lib64"] {
        for name in names {
            let path = base.join(dir).join(name);
            if path.exists() {
                return path.to_str().unwrap().to_string();
            }
        }
    }
    panic!(
        "Could not find library in {}/lib/ or {}/lib64/. Tried: {:?}",
        base.display(),
        base.display(),
        names
    );
}
