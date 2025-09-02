#![windows_subsystem = "windows"]
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), slint::PlatformError> {
    slindex::start_desktop()
}

// Para wasm32, `start_wasm` está em lib.rs com #[wasm_bindgen(start)]
#[cfg(target_arch = "wasm32")]
fn main() {}
