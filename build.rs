#[cfg(target_os = "windows")]
fn main() {
    vcpkg::Config::new()
        .emit_includes(true)
        .copy_dlls(true)
        .find_package("libpq")
        .unwrap();
}

#[cfg(not(target_os = "windows"))]
fn main() {}
