fn main() {
    // Descobre o SO de destino (não o do host!)
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    // Só embute ícone quando o TARGET for Windows (.exe)
    if target_os == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico"); // ajuste o caminho do seu .ico
        // Não derruba builds de outras plataformas caso falhe por algum motivo
        if let Err(e) = res.compile() {
            eprintln!("winres falhou: {e}");
        }
    }

    let cfg = slint_build::CompilerConfiguration::new()
        .with_style("cosmic-dark".into());
    slint_build::compile_with_config("ui/app.slint", cfg).expect("Failed to compile Slint UI");
}
