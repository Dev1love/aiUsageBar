use std::process::Command;
use std::path::Path;
use std::env;

fn main() {
    let swift_src = Path::new("swift/SystemMonitor.swift");
    if swift_src.exists() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let dylib_path = format!("{}/libsystem_monitor.dylib", out_dir);

        let status = Command::new("swiftc")
            .args([
                "-emit-library",
                "-o", &dylib_path,
                "-module-name", "SystemMonitor",
                "swift/SystemMonitor.swift",
                "-framework", "IOKit",
                "-framework", "Foundation",
                "-framework", "AppKit",
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:rustc-link-search=native={}", out_dir);
                println!("cargo:rustc-link-lib=dylib=system_monitor");
                println!("cargo:rustc-cfg=has_swift_dylib");
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
