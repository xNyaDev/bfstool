fn main() {
    #[cfg(feature = "cli")]
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        tauri_winres::WindowsResource::new()
            .set("FileDescription", "bfstool-cli")
            .set(
                "LegalCopyright",
                "Licensed under MIT or Apache-2.0, xNyaDev 2023",
            )
            .set("OriginalFilename", "bfstool-cli.exe")
            .set("ProductName", "bfstool-cli")
            .compile_for("bfstool-cli")
            .unwrap();
    }
}
