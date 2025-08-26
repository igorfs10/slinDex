# SlinDex — Rust (desktop + WASM + Android)

Pequena Pokédex com **Rust + Slint** que roda em Desktop, WebAssembly e Android, consumindo a **PokeAPI**.  
Versão web: https://igorfs10.github.io/slinDex/web/

## Pré-requisitos
- Rust estável (via `rustup`)
- **Web (WASM)**: `wasm-pack` (`cargo install wasm-pack`) e um servidor HTTP estático
- **Android**:
  - Android Studio (ou apenas SDK + NDK)
  - `cargo-apk`: `cargo install cargo-apk`
  - Targets do Rust para Android (instale os que você precisa):
    ```bash
    rustup target add x86_64-linux-android aarch64-linux-android armv7-linux-androideabi
    ```
  - Variáveis de ambiente (o `cargo-apk` costuma detectar sozinho se você instalou o SDK/NDK pelo Android Studio, mas se precisar):
    - `ANDROID_SDK_ROOT` = caminho do SDK (ex.: `~/Android/Sdk`)
    - `ANDROID_NDK_HOME` = caminho do NDK dentro do SDK

---

## Executar (Desktop)
```bash
cargo run --bin slindex_app
```
## Build (Desktop)
```bash
cargo build --release --bin slindex_app
```

## Executar-build (WebAssembly)
```bash
# 1) Adicione o target wasm32
rustup target add wasm32-unknown-unknown

# 2) Gere os artefatos com wasm-pack
wasm-pack build --release --target web --out-dir web/pkg

# 3) Sirva a pasta web/ em um servidor estático
cd web
python -m http.server 5173
# Abra http://localhost:5173
```
> Dica: Você pode usar outro servidor (vite, serve, http-server, live-server, etc.).

## Executar (Android)
## Rodar no emulador (x86_64)
> Inicie um AVD no Android Studio antes de rodar estes comandos.
```bash
# instala e executa no emulador x86_64
cargo apk run --lib --target x86_64-linux-android
```
## Rodar em dispositivo físico (ARM64)
> Ative a Depuração USB e plugue o aparelho (verifique com adb devices).
```bash
# instala e executa no dispositivo
cargo apk run --lib --target aarch64-linux-android
```

## Build (Android)
```bash
# Release (gera APK assinada com keystore de release se configurada; caso contrário remova --release)
cargo apk build --lib --release --target aarch64-linux-android
```