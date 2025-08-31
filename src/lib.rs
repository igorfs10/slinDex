use lru::LruCache;
use slint::{Brush, Color, ModelRc, SharedString, VecModel};
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

mod service;
slint::include_modules!(); // App, PokemonRow, PokemonDetail, TypeTag, StatBar...

include!(concat!(env!("OUT_DIR"), "/pokemon_list.rs")); // add lista constante com todos os pokémons

type StateHandle = Arc<Mutex<State>>;

struct State {
    view: Vec<(u32, &'static str)>,
    details: LruCache<u32, service::Detail>, // cache detalhes
    sprites: LruCache<u32, Vec<u8>>,         // cache de bytes da sprite
    selected: i32,                           // índice selecionado
}

/* ---------- helpers ---------- */
fn cap_words_and_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut cap_next = true; // capitalizar o próximo caractere não-separador

    for ch in s.chars() {
        if ch == '-' {
            out.push(' ');
            cap_next = true;
        } else if ch.is_whitespace() {
            out.push(ch);
            cap_next = true;
        } else if cap_next {
            out.extend(ch.to_uppercase()); // Unicode-safe (pode render 1+ chars)
            cap_next = false;
        } else {
            out.push(ch);
        }
    }

    out
}

// cores por tipo (chips)
fn type_color(t: &str) -> Brush {
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

// rótulo PT-BR dos tipos (com acento)
fn type_label_pt(t: &str) -> &'static str {
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

fn load_icon(bytes: &'static [u8]) -> slint::Image {
    // usa crate `image` para decodificar PNG dos bytes embutidos
    let img = image::load_from_memory(bytes).unwrap();
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
    buf.make_mut_bytes().copy_from_slice(rgba.as_raw());
    slint::Image::from_rgba8(buf)
}

// ícones dos tipos
fn type_icon(t: &str) -> slint::Image {
    // Carrega bytes embutidos em compile-time e cria a imagem; sem panic se faltar.
    // (Se faltar o arquivo, o compilador já erra na macro `include_bytes!`)
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

// cores por stat (barras)
fn stat_color(k: &str) -> Brush {
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

fn set_rows_from_pokemon(app: &App, pokemons: &[(u32, &str)]) {
    let rows: Vec<PokemonRow> = pokemons
        .iter()
        .map(|n| {
            let id = n.0;
            let nome = cap_words_and_spaces(n.1);
            PokemonRow {
                name: format!("{id} - {nome}").into(),
            }
        })
        .collect();
    app.set_rows(ModelRc::new(VecModel::from(rows)));
}

fn apply_filter(app: &App, state: &StateHandle, filter: &str) {
    let f = filter.to_lowercase();
    {
        let mut st = state.lock().unwrap();
        st.selected = -1; // limpamos seleção ao filtrar
        st.view = POKEMON_LIST
            .iter()
            .copied()
            .filter(|item| item.0.to_string().contains(&f) || item.1.to_lowercase().contains(&f))
            .collect();
    }
    let lista: Vec<(u32, &'static str)> = POKEMON_LIST
        .iter()
        .copied()
        .filter(|item| item.0.to_string().contains(&f) || item.1.to_lowercase().contains(&f))
        .collect();
    app.set_selected_index(-1);
    set_rows_from_pokemon(app, &lista);
}

// Converte bytes PNG -> Image (feito na thread da UI)
fn png_to_image(bytes: &[u8]) -> Result<slint::Image, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
    buf.make_mut_bytes().copy_from_slice(rgba.as_raw());
    Ok(slint::Image::from_rgba8(buf))
}

// Monta o `PokemonDetail` para a UI (tipos traduzidos, barras com cor, sprite e total)
fn make_detail_for_ui(d: &service::Detail, artwork_bytes: Option<&[u8]>) -> PokemonDetail {
    // chips de tipo (tradução + cor)
    let types_vec: Vec<TypeTag> = d
        .types
        .iter()
        .map(|t| TypeTag {
            label: type_label_pt(t).into(),
            bg: type_color(t),
            icon: type_icon(t),
        })
        .collect();
    let types_model = ModelRc::new(VecModel::from(types_vec));

    // stats com rótulos completos (PT-BR) + total
    let mut total: i32 = 0;
    let mut stats_v: Vec<StatBar> = Vec::with_capacity(d.stats.len());
    for (k, v) in &d.stats {
        total += *v as i32;
        let label = match k.as_str() {
            "hp" => "Pontos de Vida",
            "attack" => "Ataque",
            "defense" => "Defesa",
            "special-attack" => "Ataque Especial",
            "special-defense" => "Defesa Especial",
            "speed" => "Velocidade",
            _ => k.as_str(),
        };
        stats_v.push(StatBar {
            name: label.into(),
            value: *v as i32,
            bg: stat_color(k),
        });
    }
    let stats_model = ModelRc::new(VecModel::from(stats_v));

    // artwork
    let artwork_img = artwork_bytes
        .and_then(|b| png_to_image(b).ok())
        .unwrap_or_default();

    PokemonDetail {
        name: cap_words_and_spaces(&d.name).into(),
        id: d.id as i32,
        height: d.height as i32,
        weight: d.weight as i32,
        types: types_model,
        stats: stats_model,
        artwork: artwork_img,
        total,
        ability1: cap_words_and_spaces(&d.ability1).into(),
        ability2: cap_words_and_spaces(&d.ability2).into(),
        hiddenAbility: cap_words_and_spaces(&d.hidden_ability).into(),
        error: "".into(),
    }
}

fn set_detail_error(app: &App, msg: &str) {
    app.set_detail(PokemonDetail {
        name: "".into(),
        id: 0,
        height: 0,
        weight: 0,
        types: ModelRc::new(VecModel::from(Vec::<TypeTag>::new())),
        stats: ModelRc::new(VecModel::from(Vec::<StatBar>::new())),
        artwork: slint::Image::default(),
        total: 0,
        ability1: "".into(),
        ability2: "".into(),
        hiddenAbility: "".into(),
        error: msg.into(),
    });
}

fn set_detail_empty(app: &App) {
    app.set_detail(PokemonDetail {
        name: "Carregando...".into(),
        id: 0,
        height: 0,
        weight: 0,
        types: ModelRc::new(VecModel::from(Vec::<TypeTag>::new())),
        stats: ModelRc::new(VecModel::from(Vec::<StatBar>::new())),
        artwork: slint::Image::default(),
        total: 0,
        ability1: "".into(),
        ability2: "".into(),
        hiddenAbility: "".into(),
        error: "".into(),
    });
}

/* ---------- estado base ---------- */

fn wire_app_common(app: &App) -> StateHandle {
    let cap = NonZeroUsize::new(50).unwrap();
    let state = Arc::new(Mutex::new(State {
        view: POKEMON_LIST.iter().copied().collect(),
        details: LruCache::new(cap),
        sprites: LruCache::new(cap),
        selected: -1,
    }));
    app.set_filter(SharedString::from(""));
    app.set_selected_index(-1);
    set_detail_error(app, "");
    state
}

/* =================== Desktop =================== */
#[cfg(not(target_arch = "wasm32"))]
pub fn start_desktop() -> Result<(), slint::PlatformError> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let handle = rt.handle().clone();

    let poke_service = service::PokemonService::new();

    let app = App::new()?;
    let state = wire_app_common(&app);

    // Carrega apenas a lista de nomes
    let app_w = app.as_weak();
    let state_list = state.clone();
    let app_w_2 = app_w.clone();
    slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
        if let Some(s) = app_w_2.upgrade() {
            s.set_splash(false);
        }
    });
    app.on_request_load(move || {
        let app_w = app_w.clone();
        let state_list = state_list.clone();
        slint::invoke_from_event_loop(move || {
            if let Some(app) = app_w.upgrade() {
                let mut st = state_list.lock().unwrap();
                st.selected = -1;
                app.set_selected_index(-1);
                set_rows_from_pokemon(&app, &POKEMON_LIST);
            }
        })
        .ok();
    });

    // Clique: baixa detalhes e sprite, mostra no painel (com cache) e mantém seleção
    let app_w = app.as_weak();
    let state_sel = state.clone();
    app.on_select(move |idx| {
        if idx < 0 {
            return;
        }
        {
            state_sel.lock().unwrap().selected = idx;
        }
        if let Some(app) = app_w.upgrade() {
            app.set_carregando(true);
            set_detail_empty(&app);
            app.set_visualiza_pokemon(true);
            app.set_selected_index(idx);
        }

        let id_pokemon = {
            let st = state_sel.lock().unwrap();
            match st.view.get(idx as usize) {
                Some(&(id, _)) => id, // copia o id enquanto o lock está ativo
                None => return,
            }
        };

        if let Some(app) = app_w.upgrade() {
            let (maybe_d, maybe_bytes) = {
                let mut st = state_sel.lock().unwrap();
                (
                    st.details.get(&id_pokemon).cloned(),
                    st.sprites.get(&id_pokemon).cloned(),
                )
            };
            if let Some(d) = maybe_d {
                let ui_detail = make_detail_for_ui(&d, maybe_bytes.as_deref());
                app.set_detail(ui_detail);
                app.set_carregando(false);
                return;
            }
        }
        let app_w2 = app_w.clone();
        let state_sel2 = state_sel.clone();
        let pkclone = poke_service.clone();
        let h = handle.clone();
        h.spawn(async move {
            let dres = pkclone.fetch_pokemon_detail(id_pokemon).await;
            let (detail, sprite_bytes): (Option<service::Detail>, Option<Vec<u8>>) = match dres {
                Ok(d) => {
                    let bytes = match d.artwork_url.as_deref() {
                        Some(url) => pkclone.fetch_image(url).await.ok(),
                        None => None,
                    };
                    (Some(d), bytes)
                }
                Err(_) => (None, None),
            };

            slint::invoke_from_event_loop(move || {
                if let Some(app) = app_w2.upgrade() {
                    match detail {
                        Some(d) => {
                            {
                                let mut st = state_sel2.lock().unwrap();
                                st.details.put(id_pokemon, d.clone());
                                if let Some(b) = &sprite_bytes {
                                    st.sprites.put(id_pokemon, b.clone());
                                }
                            }
                            let ui_detail = make_detail_for_ui(&d, sprite_bytes.as_deref());
                            app.set_detail(ui_detail);
                            app.set_carregando(false);
                        }
                        None => {
                            set_detail_error(&app, "Falha ao carregar detalhes");
                            app.set_carregando(false);
                        }
                    }
                }
            })
            .ok();
        });
    });

    // Filtro
    let state_filter = state.clone();
    let app_c = app.as_weak();
    app.on_apply_filter(move |f: SharedString| {
        if let Some(app) = app_c.upgrade() {
            apply_filter(&app, &state_filter, f.as_str());
        }
    });

    // Inicial
    let app_c = app.as_weak();
    if let Some(app) = app_c.upgrade() {
        app.invoke_request_load();
    }

    app.run()
}

/* =================== WebAssembly =================== */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start_wasm() {
    console_error_panic_hook::set_once();

    let app = App::new().expect("create app");
    let state = wire_app_common(&app);
    let poke_service = service::PokemonService::new();

    // Carrega só a lista de nomes
    let app_w = app.as_weak();
    let state_list = state.clone();
    let app_w_2 = app_w.clone();
    slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
        if let Some(s) = app_w_2.upgrade() {
            s.set_splash(false);
        }
    });
    app.on_request_load(move || {
        let app_w = app_w.clone();
        let state_list = state_list.clone();
        slint::invoke_from_event_loop(move || {
            if let Some(app) = app_w.upgrade() {
                let mut st = state_list.lock().unwrap();
                st.selected = -1;
                app.set_selected_index(-1);
                set_rows_from_pokemon(&app, &POKEMON_LIST);
            }
        })
        .ok();
    });

    // Clique: baixa detalhes + sprite (async) e mantém seleção
    let app_w = app.as_weak();
    let state_sel = state.clone();
    let pkclone = poke_service.clone();
    app.on_select(move |idx| {
        if idx < 0 {
            return;
        }
        {
            state_sel.lock().unwrap().selected = idx;
        }
        if let Some(app) = app_w.upgrade() {
            app.set_carregando(true);
            set_detail_empty(&app);
            app.set_visualiza_pokemon(true);
            app.set_selected_index(idx);
        }

        let id_pokemon = {
            let st = state_sel.lock().unwrap();
            match st.view.get(idx as usize) {
                Some(&(id, _)) => id, // copia o id enquanto o lock está ativo
                None => return,
            }
        };

        if let Some(app) = app_w.upgrade() {
            let (maybe_d, maybe_bytes) = {
                let mut st = state_sel.lock().unwrap();
                (
                    st.details.get(&id_pokemon).cloned(),
                    st.sprites.get(&id_pokemon).cloned(),
                )
            };
            if let Some(d) = maybe_d {
                let ui_detail = make_detail_for_ui(&d, maybe_bytes.as_deref());
                app.set_detail(ui_detail);
                app.set_carregando(false);
                return;
            }
        }

        let app_w2 = app_w.clone();
        let state_sel2 = state_sel.clone();
        let pkclone = pkclone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let dres = pkclone.fetch_pokemon_detail(id_pokemon).await;
            let (detail, sprite_bytes): (Option<service::Detail>, Option<Vec<u8>>) = match dres {
                Ok(d) => {
                    let bytes = match d.artwork_url.as_deref() {
                        Some(url) => pkclone.fetch_image(url).await.ok(),
                        None => None,
                    };
                    (Some(d), bytes)
                }
                Err(_) => (None, None),
            };

            slint::invoke_from_event_loop(move || {
                if let Some(app) = app_w2.upgrade() {
                    match detail {
                        Some(d) => {
                            {
                                let mut st = state_sel2.lock().unwrap();
                                st.details.put(id_pokemon, d.clone());
                                if let Some(b) = &sprite_bytes {
                                    st.sprites.put(id_pokemon, b.clone());
                                }
                            }
                            let ui_detail = make_detail_for_ui(&d, sprite_bytes.as_deref());
                            app.set_detail(ui_detail);
                            app.set_carregando(false);
                        }
                        None => {
                            set_detail_error(&app, "Falha ao carregar detalhes");
                            app.set_carregando(false);
                        }
                    }
                }
            })
            .ok();
        });
    });

    // Filtro
    let state_filter = state.clone();
    let app_c = app.as_weak();
    app.on_apply_filter(move |f: SharedString| {
        if let Some(app) = app_c.upgrade() {
            apply_filter(&app, &state_filter, f.as_str());
        }
    });

    // Inicial
    let app_c = app.as_weak();
    if let Some(app) = app_c.upgrade() {
        app.invoke_request_load();
    }

    app.run().expect("run app");
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    // inicializa o backend Android do Slint antes de qualquer uso da UI
    slint::android::init(app).expect("falha ao inicializar Slint no Android");

    // reaproveita sua lógica já existente (usa threads, timers etc)
    if let Err(e) = crate::start_desktop() {
        // vai para o logcat
        eprintln!("erro ao iniciar app: {e}");
    }
}
