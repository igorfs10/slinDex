use serde::Deserialize;

const BASE: &str = "https://pokeapi.co/api/v2";

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
    abilities: Vec<Ability>
}

#[derive(Debug, Deserialize)]
pub struct Ability{
    ability: Option<AbilityInfo>,
    is_hidden: bool,
}

#[derive(Debug, Deserialize)]
pub struct AbilityInfo {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Sprites { other: Option<Other> }

#[derive(Debug, Deserialize)]
struct Other {
    #[serde(rename = "official-artwork")]
    official_artwork: Option<OfficialArtwork>
}

#[derive(Debug, Deserialize)]
struct OfficialArtwork { front_default: Option<String> }

#[derive(Debug, Clone)]
pub struct Detail {
    pub id: u32,
    pub name: String,
    pub height: u32,
    pub weight: u32,
    pub types: Vec<String>,
    pub stats: Vec<(String, u32)>,
    pub artwork_url: Option<String>,
    pub ability1: String,
    pub ability2: String,
    pub hidden_ability: String,
}

impl From<PokemonApiDetail> for Detail {
    fn from(v: PokemonApiDetail) -> Self {
        let (ab1, ab2, hidden) = split_abilities_str(&v);
        Self {
            id: v.id,
            name: v.name,
            height: v.height,
            weight: v.weight,
            types: v.types.into_iter().map(|t| t.typ.name).collect(),
            stats: v.stats.into_iter().map(|s| (s.stat.name, s.base_stat)).collect(),
            artwork_url: v.sprites.other.as_ref()
                .and_then(|o| o.official_artwork.as_ref())
                .and_then(|oa| oa.front_default.clone()),
            ability1: ab1,
            ability2: ab2,
            hidden_ability: hidden
        }
    }
}

fn split_abilities_str(v: &PokemonApiDetail) -> (String, String, String) {
    let mut normals = v.abilities
        .iter()
        .filter(|a| !a.is_hidden)
        .filter_map(|a| a.ability.as_ref().map(|abi| abi.name.as_str()));

    let ab1 = normals.next().unwrap_or("").to_string();
    let ab2 = normals.next().unwrap_or("").to_string();

    let hidden = v.abilities
        .iter()
        .find(|a| a.is_hidden)
        .and_then(|a| a.ability.as_ref().map(|abi| abi.name.as_str()))
        .unwrap_or("")
        .to_string();

    (ab1, ab2, hidden)
}


pub async fn fetch_pokemon_list() -> Result<Vec<String>, String> {
    let url = format!("{BASE}/pokemon?limit=10000");
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

pub async fn fetch_pokemon_detail(name: &str) -> Result<Detail, String> {
    let url = format!("{BASE}/pokemon/{name}");
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let data: PokemonApiDetail = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.into())
}

pub async fn fetch_image(url: &str) -> Result<Vec<u8>, String> {
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let bytes = resp.bytes().await.map_err(|e| e.to_string()).map(|b| b.to_vec());
    bytes
}