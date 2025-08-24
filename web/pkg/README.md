# Slint Pokédex — Rust (desktop + WASM)

Pequena Pokédex com **Rust + Slint** que roda em desktop e WebAssembly, consumindo a **PokeAPI**.

## Pré-requisitos
- Rust estável (via rustup)
- Para **web (WASM)**: `wasm-pack` (`cargo install wasm-pack`) e qualquer servidor HTTP estático

## Executar (Desktop)
```bash
# Linux/macOS/Windows
cargo run
```

## Executar (WebAssembly)
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