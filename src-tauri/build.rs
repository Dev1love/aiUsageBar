use std::process::Command;
use std::path::Path;
use std::env;
use std::fs;

fn main() {
    let swift_src = Path::new("swift/SystemMonitor.swift");
    if swift_src.exists() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let dylib_path = format!("{}/libsystem_monitor.dylib", out_dir);

        // Build with @rpath/libsystem_monitor.dylib as install_name so the .app
        // can be moved/redistributed without baking absolute paths into the
        // binary's load commands.
        let status = Command::new("swiftc")
            .args([
                "-emit-library",
                "-o", &dylib_path,
                "-module-name", "SystemMonitor",
                "swift/SystemMonitor.swift",
                "-framework", "IOKit",
                "-framework", "Foundation",
                "-framework", "AppKit",
                "-Xlinker", "-install_name",
                "-Xlinker", "@rpath/libsystem_monitor.dylib",
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:rustc-link-search=native={}", out_dir);
                println!("cargo:rustc-link-lib=dylib=system_monitor");
                println!("cargo:rustc-cfg=has_swift_dylib");

                // Binary must know where to find the dylib at runtime. Tauri
                // copies dylib resources into Contents/Frameworks/, so look
                // there relative to the binary inside Contents/MacOS/.
                println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");

                // Mirror the dylib to a stable relative path so Tauri's
                // bundler (and manual install scripts) can find it without
                // chasing the per-build OUT_DIR hash.
                let mirror_dir = Path::new("dylib");
                let _ = fs::create_dir_all(mirror_dir);
                let mirror_path = mirror_dir.join("libsystem_monitor.dylib");
                if let Err(e) = fs::copy(&dylib_path, &mirror_path) {
                    eprintln!("Warning: failed to mirror dylib to {mirror_path:?}: {e}");
                }
                println!("cargo:rerun-if-changed=swift/SystemMonitor.swift");
            }
            Ok(_) => {
                eprintln!("Warning: Swift dylib compilation failed.");
            }
            Err(e) => {
                eprintln!("Warning: swiftc not found ({e}).");
            }
        }
    }

    println!("cargo:rerun-if-changed=swift/SystemMonitor.swift");
    println!("cargo:rustc-check-cfg=cfg(has_swift_dylib)");
    tauri_build::build()
}
