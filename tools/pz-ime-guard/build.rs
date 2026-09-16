fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/tray-32.rgba");
    winresource::WindowsResource::new()
        .set_icon("assets/icon.ico")
        .set("ProductName", "pz-ime-guard")
        .set("FileDescription", "Companion tool for MinidoracatIMEGuardFor42 (Project Zomboid)")
        .compile()
        .expect("embed icon resource");
}
