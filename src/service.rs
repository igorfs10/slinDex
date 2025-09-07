#[derive(Clone)]
pub struct PokemonService {
    client: reqwest::Client,
}

impl PokemonService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder().build().expect("reqwest client"),
        }
    }

    pub async fn fetch_image(&self, id: u32) -> Result<Vec<u8>, String> {
        let url = format!(
            "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/other/official-artwork/{id}.png"
        );
        let resp = self.client.get(url).send().await.map_err(err)?;
        let resp = resp.error_for_status().map_err(err)?;
        let bytes = resp.bytes().await.map_err(err)?;
        Ok(bytes.to_vec())
    }
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
