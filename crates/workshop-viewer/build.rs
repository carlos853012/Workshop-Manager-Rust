fn main() {
    #[cfg(target_os = "windows")]
    {
        winresource::WindowsResource::new()
            .set_icon("icon.ico")
            .compile()
            .expect("failed to compile Windows resource");
    }
}
