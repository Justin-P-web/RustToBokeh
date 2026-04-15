use std::path::{Path, PathBuf};

fn main() {
    // Re-run if Bokeh vendor assets change (used by the `bokeh-inline` feature).
    // Emitted unconditionally so Cargo notices updates on all platforms.
    // Bokeh 3.x bundles CSS inside JS — only JS files are needed.
    println!("cargo:rerun-if-changed=vendor/bokeh/bokeh-3.9.0.min.js");
    println!("cargo:rerun-if-changed=vendor/bokeh/bokeh-widgets-3.9.0.min.js");

    // Declare the custom cfg so Rust doesn't warn about an unexpected key.
    println!("cargo:rustc-check-cfg=cfg(bokeh_vendor_present)");

    // Emit the cfg only when both vendor JS files are actually present.
    // html::bokeh_inline_resources() uses include_str! gated on this cfg, so
    // the build succeeds even when setup_vendor.sh hasn't been run yet.
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let js = manifest.join("vendor/bokeh/bokeh-3.9.0.min.js");
    let js_widgets = manifest.join("vendor/bokeh/bokeh-widgets-3.9.0.min.js");
    if js.exists() && js_widgets.exists() {
        println!("cargo:rustc-cfg=bokeh_vendor_present");
    }

    // Python DLL copying is only needed when the `python` feature is enabled.
    if std::env::var("CARGO_FEATURE_PYTHON").is_err() {
        return;
    }

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    // Copy Python DLLs from vendor/python/ to the target directory so the
    // OS loader can find them next to the executable at runtime.
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let vendor_dir = manifest_dir.join("vendor").join("python");

    if !vendor_dir.exists() {
        println!(
            "cargo:warning=vendor/python/ not found — run `bash scripts/setup_vendor.sh` first"
        );
        return;
    }

    // OUT_DIR is something like target/release/build/<crate>-<hash>/out/
    // Walk up to find the profile directory (target/release/ or target/debug/).
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    let target_dir = out_dir
        .ancestors()
        .find(|p| {
            p.file_name()
                .is_some_and(|n| n == "release" || n == "debug")
        })
        .map(Path::to_path_buf);

    let Some(target_dir) = target_dir else {
        println!("cargo:warning=Could not determine target directory from OUT_DIR");
        return;
    };

    let dlls = ["python3.dll", "python312.dll"];
    for dll in &dlls {
        let src = vendor_dir.join(dll);
        let dst = target_dir.join(dll);
        if src.exists() && !dst.exists() {
            std::fs::copy(&src, &dst).unwrap_or_else(|e| {
                panic!(
                    "Failed to copy {} to {}: {}",
                    src.display(),
                    dst.display(),
                    e
                )
            });
            println!("cargo:warning=Copied {} to {}", dll, target_dir.display());
        }
    }

    // Re-run if the vendor directory changes.
    println!("cargo:rerun-if-changed=vendor/python/python3.dll");
    println!("cargo:rerun-if-changed=vendor/python/python312.dll");
}
