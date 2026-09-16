fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/tray-32.rgba");
    winresource::WindowsResource::new()
        .set_icon("assets/icon.ico")
        // SignPath 要求 ProductName＝專案名、ProductVersion 每次建置一致（winresource 預設帶 CARGO_PKG_VERSION）
        .set("ProductName", "MinidoracatIMEGuardFor42")
        .set("FileDescription", "pz-ime-guard — companion tool for MinidoracatIMEGuardFor42 (Project Zomboid)")
        .set("CompanyName", "Minidoracat")
        .set("LegalCopyright", "MIT License — Minidoracat")
        .compile()
        .expect("embed icon resource");
}
