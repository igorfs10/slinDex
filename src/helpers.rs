use slint::{Brush, Color};
/// Capitaliza palavras e substitui hífens por espaço
pub fn cap_words_and_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut cap_next = true;
    for ch in s.chars() {
        if ch == '-' {
            out.push(' ');
            cap_next = true;
        } else if ch.is_whitespace() {
            out.push(ch);
            cap_next = true;
        } else if cap_next {
            out.extend(ch.to_uppercase());
            cap_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Cor por tipo
pub fn type_color(t: &str) -> Brush {
    let c = match t {
        "normal" => Color::from_rgb_u8(145, 154, 162),
        "fire" => Color::from_rgb_u8(255, 157, 85),
        "water" => Color::from_rgb_u8(80, 144, 214),
        "electric" => Color::from_rgb_u8(244, 210, 60),
        "grass" => Color::from_rgb_u8(99, 188, 90),
        "ice" => Color::from_rgb_u8(115, 206, 192),
        "fighting" => Color::from_rgb_u8(206, 65, 107),
        "poison" => Color::from_rgb_u8(170, 107, 200),
        "ground" => Color::from_rgb_u8(217, 120, 69),
        "flying" => Color::from_rgb_u8(143, 169, 222),
        "psychic" => Color::from_rgb_u8(250, 113, 121),
        "bug" => Color::from_rgb_u8(145, 193, 47),
        "rock" => Color::from_rgb_u8(197, 183, 140),
        "ghost" => Color::from_rgb_u8(82, 105, 173),
        "dragon" => Color::from_rgb_u8(11, 109, 195),
        "dark" => Color::from_rgb_u8(90, 84, 101),
        "steel" => Color::from_rgb_u8(90, 142, 162),
        "fairy" => Color::from_rgb_u8(236, 143, 230),
        _ => Color::from_rgb_u8(145, 154, 162),
    };
    Brush::from(c)
}

/// Rótulo PT-BR dos tipos
pub fn type_label_pt(t: &str) -> &'static str {
    match t {
        "normal" => "Normal",
        "fire" => "Fogo",
        "water" => "Água",
        "electric" => "Elétrico",
        "grass" => "Grama",
        "ice" => "Gelo",
        "fighting" => "Lutador",
        "poison" => "Venenoso",
        "ground" => "Terrestre",
        "flying" => "Voador",
        "psychic" => "Psíquico",
        "bug" => "Inseto",
        "rock" => "Pedra",
        "ghost" => "Fantasma",
        "dragon" => "Dragão",
        "dark" => "Noturno",
        "steel" => "Aço",
        "fairy" => "Fada",
        _ => "Desconhecido",
    }
}

/// Decodifica PNG dos bytes embutidos
pub fn load_icon(bytes: &'static [u8]) -> slint::Image {
    let img = image::load_from_memory(bytes).unwrap();
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
    buf.make_mut_bytes().copy_from_slice(rgba.as_raw());
    slint::Image::from_rgba8(buf)
}

/// Ícone por tipo
pub fn type_icon(t: &str) -> slint::Image {
    match t {
        "normal" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/normal.png"
        ))),
        "fire" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/fire.png"
        ))),
        "water" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/water.png"
        ))),
        "electric" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/electric.png"
        ))),
        "grass" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/grass.png"
        ))),
        "ice" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/ice.png"
        ))),
        "fighting" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/fighting.png"
        ))),
        "poison" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/poison.png"
        ))),
        "ground" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/ground.png"
        ))),
        "flying" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/flying.png"
        ))),
        "psychic" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/psychic.png"
        ))),
        "bug" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/bug.png"
        ))),
        "rock" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/rock.png"
        ))),
        "ghost" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/ghost.png"
        ))),
        "dragon" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/dragon.png"
        ))),
        "dark" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/dark.png"
        ))),
        "steel" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/steel.png"
        ))),
        "fairy" => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/fairy.png"
        ))),
        _ => load_icon(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/imagens/tipos/normal.png"
        ))),
    }
}

/// Cor por stat
pub fn stat_color(k: &str) -> Brush {
    let c = match k {
        "hp" => Color::from_rgb_u8(105, 220, 18),
        "attack" => Color::from_rgb_u8(239, 204, 24),
        "defense" => Color::from_rgb_u8(232, 100, 18),
        "special-attack" => Color::from_rgb_u8(20, 195, 241),
        "special-defense" => Color::from_rgb_u8(74, 106, 223),
        "speed" => Color::from_rgb_u8(239, 99, 200),
        _ => Color::from_rgb_u8(213, 29, 173),
    };
    Brush::from(c)
}

/// Converte bytes PNG -> Image
pub fn png_to_image(bytes: &[u8]) -> Result<slint::Image, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
    buf.make_mut_bytes().copy_from_slice(rgba.as_raw());
    Ok(slint::Image::from_rgba8(buf))
}
