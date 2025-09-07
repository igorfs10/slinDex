use rust_embed::Embed;
use slint::{Brush, Color};

// ÍCONES DE TIPOS
#[derive(Embed)]
#[folder = "imagens/tipos/"] // embute toda a pasta
struct TypeIcons;

// Artwork dos Pokémons
#[derive(Embed)]
#[folder = "imagens/official-artwork/"] // embute toda a pasta
struct OfficialArtworks;

/// Cor por tipo
pub fn type_color(t: &str) -> Brush {
    let c = match t {
        "Normal" => Color::from_rgb_u8(145, 154, 162),
        "Fire" => Color::from_rgb_u8(255, 157, 85),
        "Water" => Color::from_rgb_u8(80, 144, 214),
        "Electric" => Color::from_rgb_u8(244, 210, 60),
        "Grass" => Color::from_rgb_u8(99, 188, 90),
        "Ice" => Color::from_rgb_u8(115, 206, 192),
        "Fighting" => Color::from_rgb_u8(206, 65, 107),
        "Poison" => Color::from_rgb_u8(170, 107, 200),
        "Ground" => Color::from_rgb_u8(217, 120, 69),
        "Flying" => Color::from_rgb_u8(143, 169, 222),
        "Psychic" => Color::from_rgb_u8(250, 113, 121),
        "Bug" => Color::from_rgb_u8(145, 193, 47),
        "Rock" => Color::from_rgb_u8(197, 183, 140),
        "Ghost" => Color::from_rgb_u8(82, 105, 173),
        "Dragon" => Color::from_rgb_u8(11, 109, 195),
        "Dark" => Color::from_rgb_u8(90, 84, 101),
        "Steel" => Color::from_rgb_u8(90, 142, 162),
        "Fairy" => Color::from_rgb_u8(236, 143, 230),
        _ => Color::from_rgb_u8(145, 154, 162),
    };
    Brush::from(c)
}

/// Rótulo PT-BR dos tipos
pub fn type_label_pt(t: &str) -> &'static str {
    match t {
        "Normal" => "Normal",
        "Fire" => "Fogo",
        "Water" => "Água",
        "Electric" => "Elétrico",
        "Grass" => "Grama",
        "Ice" => "Gelo",
        "Fighting" => "Lutador",
        "Poison" => "Venenoso",
        "Ground" => "Terrestre",
        "Flying" => "Voador",
        "Psychic" => "Psíquico",
        "Bug" => "Inseto",
        "Rock" => "Pedra",
        "Ghost" => "Fantasma",
        "Dragon" => "Dragão",
        "Dark" => "Noturno",
        "Steel" => "Aço",
        "Fairy" => "Fada",
        _ => "Desconhecido",
    }
}

fn load_embedded_image(bytes: &[u8]) -> slint::Image {
    // usa seu png_to_image; se quiser suportar .webp também, o `image` já lida
    png_to_image(bytes).unwrap_or_default()
}

// carrega um ícone de tipo pelo nome (ex.: "poison" -> "poison.png")
pub fn type_icon(t: &str) -> slint::Image {
    if let Some(embeded_file) = TypeIcons::get(&format!("{t}.png")) {
        load_embedded_image(embeded_file.data.as_ref())
    } else {
        slint::Image::default() // fallback
    }
}

// carrega um ícone de tipo pelo nome (ex.: "poison" -> "poison.png")
pub fn artwork_img(species_id: u32) -> slint::Image {
    if let Some(embeded_file) = OfficialArtworks::get(&format!("{species_id}.png")) {
        load_embedded_image(embeded_file.data.as_ref())
    } else {
        slint::Image::default() // fallback
    }
}

/// Cor pokemon
pub fn pokemon_color(k: &str) -> Brush {
    let c = match k {
        "black" => Color::from_rgb_u8(43, 43, 43),    // Black
        "blue" => Color::from_rgb_u8(0, 149, 217),    // Blue
        "brown" => Color::from_rgb_u8(150, 80, 66),   // Brown
        "gray" => Color::from_rgb_u8(125, 125, 125),  // Gray
        "green" => Color::from_rgb_u8(62, 179, 112),  // Green
        "pink" => Color::from_rgb_u8(227, 134, 152),  // Pink
        "purple" => Color::from_rgb_u8(136, 72, 152), // Purple
        "red" => Color::from_rgb_u8(230, 0, 51),      // Red
        "white" => Color::from_rgb_u8(255, 255, 255), // White
        "yellow" => Color::from_rgb_u8(255, 217, 0),  // Yellow
        _ => Color::from_rgb_u8(0, 0, 0),             // Default: Black
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
