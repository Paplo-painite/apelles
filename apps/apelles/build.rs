//! Windows only: embed the app icon and version info (VERSIONINFO) into `apelles.exe`.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `APELLES_REQUIRE_WINRES=1` (set by the
//! release workflow) turns it into an error.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/apelles.ico");
    println!("cargo:rerun-if-env-changed=APELLES_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/apelles.ico")
        .set("ProductName", "Apelles")
        .set("FileDescription", "Apelles image editor")
        .set("CompanyName", "Learning Machines LLC")
        .set("LegalCopyright", "Copyright (c) the Apelles authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "apelles.exe")
        .set("InternalName", "apelles");
    if let Err(e) = res.compile() {
        if std::env::var_os("APELLES_REQUIRE_WINRES").is_some() {
            panic!("embedding Windows resources failed: {e}");
        }
        println!("cargo:warning=apelles.exe built without icon/version resources: {e}");
    }
}
