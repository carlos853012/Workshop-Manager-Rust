fn main() {
    #[cfg(target_os = "windows")]
    {
        winresource::WindowsResource::new()
            .set_manifest_file("manifest.xml")
            .compile()
            .expect("failed to compile Windows resource");
    }
}
