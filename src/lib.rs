use helpers::*;
use lru::LruCache;
use slint::{Brush, Color, ModelRc, SharedString, VecModel};
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

mod helpers;
mod service;
slint::include_modules!(); // App, PokemonRow, PokemonDetail, TypeTag, StatBar...

include!(concat!(env!("OUT_DIR"), "/pokemon_list.rs")); // add lista constante com todos os pokémons

type StateHandle = Arc<Mutex<State>>;

/// Estado compartilhado da aplicação
struct State {
    view: Vec<Pokemon>,
    details: LruCache<u32, service::Detail>, // cache detalhes
    sprites: LruCache<u32, Vec<u8>>,         // cache de bytes da sprite
    selected: i32,                           // índice selecionado
}

// =================== UI Utils ===================
fn set_rows_from_pokemon(app: &App, pokemons: &[Pokemon]) {
    let rows: Vec<PokemonRow> = pokemons
        .iter()
        .map(|pokemon| PokemonRow {
            name: format!("{} - {}", pokemon.id, pokemon.name).into(),
        })
        .collect();
    app.set_rows(ModelRc::new(VecModel::from(rows)));
}

fn apply_filter(app: &App, state: &StateHandle, filter: &str) {
    let filter_lower = filter.to_lowercase();
    {
        let mut state = state.lock().unwrap();
        state.selected = -1;
        state.view = POKEMON_LIST
            .iter()
            .copied()
            .filter(|item| {
                item.id.to_string().contains(&filter_lower)
                    || item.name.to_lowercase().contains(&filter_lower)
            })
            .collect();
    }
    let filtered_list: Vec<Pokemon> = POKEMON_LIST
        .iter()
        .copied()
        .filter(|item| {
            item.id.to_string().contains(&filter_lower)
                || item.name.to_lowercase().contains(&filter_lower)
        })
        .collect();
    app.set_selected_index(-1);
    set_rows_from_pokemon(app, &filtered_list);
}

fn make_detail_for_ui(detail: &service::Detail, artwork_bytes: Option<&[u8]>) -> PokemonDetail {
    // Monta chips de tipo
    let types_vec: Vec<TypeTag> = detail
        .types
        .iter()
        .map(|t| TypeTag {
            label: type_label_pt(t).into(),
            bg: type_color(t),
            icon: type_icon(t),
        })
        .collect();
    let types_model = ModelRc::new(VecModel::from(types_vec));

    // Monta stats
    let mut total: i32 = 0;
    let mut stats_vec: Vec<StatBar> = Vec::with_capacity(detail.stats.len());
    for (k, v) in &detail.stats {
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
        stats_vec.push(StatBar {
            name: label.into(),
            value: *v as i32,
            bg: stat_color(k),
        });
    }
    let stats_model = ModelRc::new(VecModel::from(stats_vec));

    // Artwork
    let artwork_img = artwork_bytes
        .and_then(|b| png_to_image(b).ok())
        .unwrap_or_default();

    PokemonDetail {
        name: POKEMON_LIST
            .iter()
            .find(|p| p.id == detail.id)
            .map(|p| p.name)
            .unwrap_or_default()
            .into(),
        id: detail.id as i32,
        height: detail.height as i32,
        weight: detail.weight as i32,
        types: types_model,
        stats: stats_model,
        artwork: artwork_img,
        total,
        ability1: cap_words_and_spaces(&detail.ability1).into(),
        ability2: cap_words_and_spaces(&detail.ability2).into(),
        hiddenAbility: cap_words_and_spaces(&detail.hidden_ability).into(),
        error: "".into(),
        color: pokemon_color(
            POKEMON_LIST
                .iter()
                .find(|p| p.id == detail.id)
                .map(|p| p.color)
                .unwrap_or("11"),
        ), // default
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
        color: Brush::from(Color::from_argb_encoded(0x00000000)),
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
        color: Brush::from(Color::from_argb_encoded(0x00000000)),
    });
}

// =================== Estado base ===================
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

// =================== Desktop ===================
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

    // Splash
    let app_w = app.as_weak();
    let app_w_2 = app_w.clone();
    slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
        if let Some(s) = app_w_2.upgrade() {
            s.set_splash(false);
        }
    });

    // Carrega lista
    let state_list = state.clone();
    app.on_request_load(move || {
        let app_w = app_w.clone();
        let state_list = state_list.clone();
        slint::invoke_from_event_loop(move || {
            if let Some(app) = app_w.upgrade() {
                let mut state = state_list.lock().unwrap();
                state.selected = -1;
                app.set_selected_index(-1);
                set_rows_from_pokemon(&app, &POKEMON_LIST);
            }
        })
        .ok();
    });

    // Seleção
    let state_sel = state.clone();
    let app_w = app.as_weak();
    app.on_select(move |idx| {
        if idx < 0 {
            return;
        }
        state_sel.lock().unwrap().selected = idx;
        if let Some(app) = app_w.upgrade() {
            app.set_carregando(true);
            set_detail_empty(&app);
            app.set_visualiza_pokemon(true);
            app.set_selected_index(idx);
        }
        let id_pokemon = {
            let state = state_sel.lock().unwrap();
            match state.view.get(idx as usize) {
                Some(&pokemon) => pokemon.id,
                None => return,
            }
        };
        if let Some(app) = app_w.upgrade() {
            let (maybe_detail, maybe_bytes) = {
                let mut state = state_sel.lock().unwrap();
                (
                    state.details.get(&id_pokemon).cloned(),
                    state.sprites.get(&id_pokemon).cloned(),
                )
            };
            if let Some(detail) = maybe_detail {
                let ui_detail = make_detail_for_ui(&detail, maybe_bytes.as_deref());
                app.set_detail(ui_detail);
                app.set_carregando(false);
                return;
            }
        }
        let app_w2 = app_w.clone();
        let state_sel2 = state_sel.clone();
        let poke_service = poke_service.clone();
        let handle = handle.clone();
        handle.spawn(async move {
            let detail_result = poke_service.fetch_pokemon_detail(id_pokemon).await;
            let (detail, sprite_bytes): (Option<service::Detail>, Option<Vec<u8>>) =
                match detail_result {
                    Ok(detail) => {
                        let bytes = match detail.artwork_url.as_deref() {
                            Some(url) => poke_service.fetch_image(url).await.ok(),
                            None => None,
                        };
                        (Some(detail), bytes)
                    }
                    Err(_) => (None, None),
                };
            slint::invoke_from_event_loop(move || {
                if let Some(app) = app_w2.upgrade() {
                    match detail {
                        Some(detail) => {
                            let mut state = state_sel2.lock().unwrap();
                            state.details.put(id_pokemon, detail.clone());
                            if let Some(b) = &sprite_bytes {
                                state.sprites.put(id_pokemon, b.clone());
                            }
                            let ui_detail = make_detail_for_ui(&detail, sprite_bytes.as_deref());
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

// =================== WebAssembly ===================
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start_wasm() {
    console_error_panic_hook::set_once();
    let app = App::new().expect("create app");
    let state = wire_app_common(&app);
    let poke_service = service::PokemonService::new();

    // Splash
    let app_w = app.as_weak();
    let app_w_2 = app_w.clone();
    slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
        if let Some(s) = app_w_2.upgrade() {
            s.set_splash(false);
        }
    });

    // Carrega lista
    let state_list = state.clone();
    app.on_request_load(move || {
        let app_w = app_w.clone();
        let state_list = state_list.clone();
        slint::invoke_from_event_loop(move || {
            if let Some(app) = app_w.upgrade() {
                let mut state = state_list.lock().unwrap();
                state.selected = -1;
                app.set_selected_index(-1);
                set_rows_from_pokemon(&app, &POKEMON_LIST);
            }
        })
        .ok();
    });

    // Seleção
    let state_sel = state.clone();
    let app_w = app.as_weak();
    let poke_service = poke_service.clone();
    app.on_select(move |idx| {
        if idx < 0 {
            return;
        }
        state_sel.lock().unwrap().selected = idx;
        if let Some(app) = app_w.upgrade() {
            app.set_carregando(true);
            set_detail_empty(&app);
            app.set_visualiza_pokemon(true);
            app.set_selected_index(idx);
        }
        let id_pokemon = {
            let state = state_sel.lock().unwrap();
            match state.view.get(idx as usize) {
                Some(&pokemon) => pokemon.id,
                None => return,
            }
        };
        if let Some(app) = app_w.upgrade() {
            let (maybe_detail, maybe_bytes) = {
                let mut state = state_sel.lock().unwrap();
                (
                    state.details.get(&id_pokemon).cloned(),
                    state.sprites.get(&id_pokemon).cloned(),
                )
            };
            if let Some(detail) = maybe_detail {
                let ui_detail = make_detail_for_ui(&detail, maybe_bytes.as_deref());
                app.set_detail(ui_detail);
                app.set_carregando(false);
                return;
            }
        }
        let app_w2 = app_w.clone();
        let state_sel2 = state_sel.clone();
        let poke_service = poke_service.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let detail_result = poke_service.fetch_pokemon_detail(id_pokemon).await;
            let (detail, sprite_bytes): (Option<service::Detail>, Option<Vec<u8>>) =
                match detail_result {
                    Ok(detail) => {
                        let bytes = match detail.artwork_url.as_deref() {
                            Some(url) => poke_service.fetch_image(url).await.ok(),
                            None => None,
                        };
                        (Some(detail), bytes)
                    }
                    Err(_) => (None, None),
                };
            slint::invoke_from_event_loop(move || {
                if let Some(app) = app_w2.upgrade() {
                    match detail {
                        Some(detail) => {
                            let mut state = state_sel2.lock().unwrap();
                            state.details.put(id_pokemon, detail.clone());
                            if let Some(b) = &sprite_bytes {
                                state.sprites.put(id_pokemon, b.clone());
                            }
                            let ui_detail = make_detail_for_ui(&detail, sprite_bytes.as_deref());
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

// =================== Android ===================
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).expect("falha ao inicializar Slint no Android");
    if let Err(e) = crate::start_desktop() {
        eprintln!("erro ao iniciar app: {e}");
    }
}
