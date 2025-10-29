fn main() {
    // Force static linking of OpenSSL for macOS builds
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-env=OPENSSL_STATIC=1");
    }
    tauri_build::build()
}
