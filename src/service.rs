use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PokemonTypeEntry { #[serde(rename = "type")] typ: NamedResource }
#[derive(Debug, Deserialize)]
struct NamedResource { name: String}

#[derive(Debug, Deserialize)]
struct StatEntry { base_stat: u32, stat: NamedResource }

#[derive(Debug, Deserialize)]
struct PokemonApiDetail {
    id: u32,
    name: String,
    height: u32,
    weight: u32,
    types: Vec<PokemonTypeEntry>,
    stats: Vec<StatEntry>,
    sprites: Sprites,
}

#[derive(Debug, Deserialize)]
struct Sprites { front_default: Option<String> }

#[derive(Debug, Clone)]
pub struct Detail {
    pub id: u32,
    pub name: String,
    pub height: u32,
    pub weight: u32,
    pub types: Vec<String>,
    pub stats: Vec<(String, u32)>,
    pub sprite_url: Option<String>,
}

impl From<PokemonApiDetail> for Detail {
    fn from(v: PokemonApiDetail) -> Self {
        Self {
            id: v.id,
            name: v.name,
            height: v.height,
            weight: v.weight,
            types: v.types.into_iter().map(|t| t.typ.name).collect(),
            stats: v.stats.into_iter().map(|s| (s.stat.name, s.base_stat)).collect(),
            sprite_url: v.sprites.front_default,
        }
    }
}

const BASE: &str = "https://pokeapi.co/api/v2";

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_pokemon_list_blocking() -> Result<Vec<String>, String> {
    let url = "https://pokeapi.co/api/v2/pokemon?limit=10000"; // todos
    let resp = reqwest::blocking::get(url).map_err(|e| e.to_string())?;
    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let results = json["results"]
        .as_array()
        .ok_or("lista inválida")?;
    Ok(results
        .iter()
        .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
        .collect())
}


#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_pokemon_detail_blocking(name: &str) -> Result<Detail, String> {
    let url = format!("{BASE}/pokemon/{name}");
    let resp = reqwest::blocking::get(&url).map_err(|e| e.to_string())?;
    let data: PokemonApiDetail = resp.json().map_err(|e| e.to_string())?;
    Ok(data.into())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pokemon_list() -> Result<Vec<String>, String> {
    let url = "https://pokeapi.co/api/v2/pokemon?limit=10000"; // todos
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let results = json["results"]
        .as_array()
        .ok_or("lista inválida")?;
    Ok(results
        .iter()
        .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
        .collect())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_pokemon_detail(name: &str) -> Result<Detail, String> {
    let url = format!("{BASE}/pokemon/{name}");
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let data: PokemonApiDetail = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.into())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_image_blocking(url: &str) -> Result<Vec<u8>, String> {
    let resp = reqwest::blocking::get(url).map_err(|e| e.to_string())?;
    resp.bytes().map(|b| b.to_vec()).map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_image(url: &str) -> Result<Vec<u8>, String> {
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let bytes = resp.bytes().await.map_err(|e| e.to_string()).map(|b| b.to_vec());
    bytes
}