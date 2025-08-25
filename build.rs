fn main() {
    let cfg = slint_build::CompilerConfiguration::new()
        .with_style("cosmic-dark".into());
    slint_build::compile_with_config("ui/app.slint", cfg).expect("Failed to compile Slint UI");
}
