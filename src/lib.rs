use slint::{Brush, Color, ModelRc, SharedString, VecModel};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod service;
slint::include_modules!(); // App, PokemonRow, PokemonDetail, TypeTag, StatBar...

type StateHandle = Arc<Mutex<State>>;

#[derive(Default)]
struct State {
    full: Vec<String>,                         // todos os nomes
    view: Vec<String>,                         // nomes filtrados (lista)
    details: HashMap<String, service::Detail>, // cache de detalhes
    sprites: HashMap<String, Vec<u8>>,         // cache de bytes da sprite
    selected: i32,                             // índice selecionado
}

/* ---------- helpers ---------- */

fn cap_first(s: &str) -> String {
    let mut it = s.chars();
    match it.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + it.as_str(),
    }
}

// cores por tipo (chips)
fn type_color(t: &str) -> Brush {
    let c = match t {
        "normal" => Color::from_rgb_u8(168, 167, 122),
        "fire" => Color::from_rgb_u8(238, 129, 48),
        "water" => Color::from_rgb_u8(99, 144, 240),
        "electric" => Color::from_rgb_u8(247, 208, 44),
        "grass" => Color::from_rgb_u8(122, 199, 76),
        "ice" => Color::from_rgb_u8(150, 217, 214),
        "fighting" => Color::from_rgb_u8(194, 46, 40),
        "poison" => Color::from_rgb_u8(163, 62, 161),
        "ground" => Color::from_rgb_u8(226, 191, 101),
        "flying" => Color::from_rgb_u8(169, 143, 243),
        "psychic" => Color::from_rgb_u8(249, 85, 135),
        "bug" => Color::from_rgb_u8(166, 185, 26),
        "rock" => Color::from_rgb_u8(182, 161, 54),
        "ghost" => Color::from_rgb_u8(115, 87, 151),
        "dragon" => Color::from_rgb_u8(111, 53, 252),
        "dark" => Color::from_rgb_u8(112, 87, 70),
        "steel" => Color::from_rgb_u8(183, 183, 206),
        "fairy" => Color::from_rgb_u8(214, 133, 173),
        _ => Color::from_rgb_u8(156, 163, 175),
    };
    Brush::from(c)
}

// texto branco para fundos escuros específicos
fn type_fg_color(t: &str) -> Brush {
    match t {
        "ghost" | "dark" | "poison" => Brush::from(Color::from_rgb_u8(255, 255, 255)), // branco
        _ => Brush::from(Color::from_rgb_u8(15, 23, 42)), // azul-bem-escuro (como antes)
    }
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

// cores por stat (barras)
fn stat_color(k: &str) -> Brush {
    let c = match k {
        "hp" => Color::from_rgb_u8(239, 68, 68),              // vermelho
        "attack" => Color::from_rgb_u8(245, 158, 11),         // âmbar
        "defense" => Color::from_rgb_u8(59, 130, 246),        // azul
        "special-attack" => Color::from_rgb_u8(16, 185, 129), // verde
        "special-defense" => Color::from_rgb_u8(139, 92, 246),// roxo
        "speed" => Color::from_rgb_u8(234, 179, 8),           // amarelo
        _ => Color::from_rgb_u8(107, 114, 128),               // cinza
    };
    Brush::from(c)
}

fn set_rows_from_names(app: &App, names: &[String]) {
    let rows: Vec<PokemonRow> = names
        .iter()
        .map(|n| PokemonRow { name: cap_first(n).into() })
        .collect();
    app.set_rows(ModelRc::new(VecModel::from(rows)));
}

fn apply_filter(app: &App, state: &StateHandle, filter: &str) {
    let f = filter.to_lowercase();
    {
        let mut st = state.lock().unwrap();
        st.view = st
            .full
            .iter()
            .filter(|n| n.to_lowercase().contains(&f))
            .cloned()
            .collect();
        st.selected = -1; // limpamos seleção ao filtrar
    }
    app.set_selected_index(-1);
    let snapshot = { state.lock().unwrap().view.clone() };
    set_rows_from_names(app, &snapshot);
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
fn make_detail_for_ui(
    d: &service::Detail,
    artwork_bytes: Option<&[u8]>,
) -> PokemonDetail {
    // chips de tipo (tradução + cor)
    let types_vec: Vec<TypeTag> = d
        .types
        .iter()
        .map(|t| TypeTag {
            label: type_label_pt(t).into(),
            bg: type_color(t),
            fg: type_fg_color(t),
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
        name: cap_first(&d.name).into(),
        id: d.id as i32,
        height: d.height as i32,
        weight: d.weight as i32,
        types: types_model,
        stats: stats_model,
        artwork: artwork_img,
        total,
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
        error: msg.into(),
    });
}

/* ---------- estado base ---------- */

fn wire_app_common(app: &App) -> StateHandle {
    let state = Arc::new(Mutex::new(State { selected: -1, ..Default::default() }));
    app.set_loading(false);
    app.set_filter(SharedString::from(""));
    app.set_selected_index(-1);
    set_detail_error(app, "");
    state
}

/* =================== Desktop =================== */

#[cfg(not(target_arch = "wasm32"))]
pub fn start_desktop() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    let state = wire_app_common(&app);

    // Carrega apenas a lista de nomes
    let app_w = app.as_weak();
    let state_list = state.clone();
    app.on_request_load(move || {
        if let Some(app) = app_w.upgrade() { app.set_loading(true); }
        let app_w = app_w.clone();
        let state_list = state_list.clone();
        std::thread::spawn(move || {
            let res = service::fetch_pokemon_list_blocking();
            slint::invoke_from_event_loop(move || {
                if let Some(app) = app_w.upgrade() {
                    match res {
                        Ok(full) => {
                            { let mut st = state_list.lock().unwrap(); st.full = full.clone(); st.view = full.clone(); st.selected = -1; }
                            app.set_selected_index(-1);
                            let snapshot = { state_list.lock().unwrap().view.clone() };
                            set_rows_from_names(&app, &snapshot);
                        }
                        Err(e) => set_detail_error(&app, &e),
                    }
                    app.set_loading(false);
                }
            }).ok();
        });
    });

    // Clique: baixa detalhes e sprite, mostra no painel (com cache) e mantém seleção
    let app_w = app.as_weak();
    let state_sel = state.clone();
    app.on_select(move |idx| {
        if idx < 0 { return; }
        { state_sel.lock().unwrap().selected = idx; }
        if let Some(app) = app_w.upgrade() { app.set_selected_index(idx); }

        let name = { let st = state_sel.lock().unwrap(); st.view.get(idx as usize).cloned() };
        let Some(name) = name else { return };

        if let Some(app) = app_w.upgrade() {
            let (maybe_d, maybe_bytes) = {
                let st = state_sel.lock().unwrap();
                (st.details.get(&name).cloned(), st.sprites.get(&name).cloned())
            };
            if let Some(d) = maybe_d {
                let ui_detail = make_detail_for_ui(&d, maybe_bytes.as_deref());
                app.set_detail(ui_detail);
                return;
            }
            app.set_loading(true);
        }

        let app_w2 = app_w.clone();
        let state_sel2 = state_sel.clone();
        std::thread::spawn(move || {
            let dres = service::fetch_pokemon_detail_blocking(&name);
            let (detail, sprite_bytes): (Option<service::Detail>, Option<Vec<u8>>) = match dres {
                Ok(d) => {
                    let bytes = match d.artwork_url.as_deref() {
                        Some(url) => service::fetch_image_blocking(url).ok(),
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
                                st.details.insert(name.clone(), d.clone());
                                if let Some(b) = &sprite_bytes {
                                    st.sprites.insert(name.clone(), b.clone());
                                }
                            }
                            let ui_detail = make_detail_for_ui(&d, sprite_bytes.as_deref());
                            app.set_detail(ui_detail);
                        }
                        None => set_detail_error(&app, "Falha ao carregar detalhes"),
                    }
                    app.set_loading(false);
                }
            }).ok();
        });
    });

    // Filtro
    let state_filter = state.clone();
    let app_c = app.as_weak();
    app.on_apply_filter(move |f: SharedString| {
        if let Some(app) = app_c.upgrade() {
            apply_filter(&app, &state_filter, &f.as_str().to_string());
        }
    });

    // Inicial
    let app_c = app.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(50), move || {
        if let Some(app) = app_c.upgrade() { app.invoke_request_load(); }
    });

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

    // Carrega só a lista de nomes
    let app_w = app.as_weak();
    let state_list = state.clone();
    app.on_request_load(move || {
        if let Some(app) = app_w.upgrade() { app.set_loading(true); }
        let app_w = app_w.clone();
        let state_list = state_list.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let res = service::fetch_pokemon_list().await;
            slint::invoke_from_event_loop(move || {
                if let Some(app) = app_w.upgrade() {
                    match res {
                        Ok(full) => {
                            { let mut st = state_list.lock().unwrap(); st.full = full.clone(); st.view = full.clone(); st.selected = -1; }
                            app.set_selected_index(-1);
                            let snapshot = { state_list.lock().unwrap().view.clone() };
                            set_rows_from_names(&app, &snapshot);
                        }
                        Err(e) => set_detail_error(&app, &e),
                    }
                    app.set_loading(false);
                }
            }).ok();
        });
    });

    // Clique: baixa detalhes + sprite (async) e mantém seleção
    let app_w = app.as_weak();
    let state_sel = state.clone();
    app.on_select(move |idx| {
        if idx < 0 { return; }
        { state_sel.lock().unwrap().selected = idx; }
        if let Some(app) = app_w.upgrade() { app.set_selected_index(idx); }

        let name = { let st = state_sel.lock().unwrap(); st.view.get(idx as usize).cloned() };
        let Some(name) = name else { return };

        if let Some(app) = app_w.upgrade() {
            let (maybe_d, maybe_bytes) = {
                let st = state_sel.lock().unwrap();
                (st.details.get(&name).cloned(), st.sprites.get(&name).cloned())
            };
            if let Some(d) = maybe_d {
                let ui_detail = make_detail_for_ui(&d, maybe_bytes.as_deref());
                app.set_detail(ui_detail);
                return;
            }
            app.set_loading(true);
        }

        let app_w2 = app_w.clone();
        let state_sel2 = state_sel.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let dres = service::fetch_pokemon_detail(&name).await;
            let (detail, sprite_bytes): (Option<service::Detail>, Option<Vec<u8>>) = match dres {
                Ok(d) => {
                    let bytes = match d.artwork_url.as_deref() {
                        Some(url) => service::fetch_image(url).await.ok(),
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
                                st.details.insert(name.clone(), d.clone());
                                if let Some(b) = &sprite_bytes {
                                    st.sprites.insert(name.clone(), b.clone());
                                }
                            }
                            let ui_detail = make_detail_for_ui(&d, sprite_bytes.as_deref());
                            app.set_detail(ui_detail);
                        }
                        None => set_detail_error(&app, "Falha ao carregar detalhes"),
                    }
                    app.set_loading(false);
                }
            }).ok();
        });
    });

    // Filtro
    let state_filter = state.clone();
    let app_c = app.as_weak();
    app.on_apply_filter(move |f: SharedString| {
        if let Some(app) = app_c.upgrade() {
            apply_filter(&app, &state_filter, &f.as_str().to_string());
        }
    });

    // Inicial
    let app_c = app.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(50), move || {
        if let Some(app) = app_c.upgrade() { app.invoke_request_load(); }
    });

    app.run().expect("run app");
}
