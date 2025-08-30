use lru::LruCache;
use slint::{Brush, Color, Image, ModelRc, SharedString, VecModel};
use std::{num::NonZeroUsize, path::Path, sync::{Arc, Mutex}};

mod service;
slint::include_modules!(); // App, PokemonRow, PokemonDetail, TypeTag, StatBar...

type StateHandle = Arc<Mutex<State>>;

struct State {
    full: Vec<String>,                              // todos os nomes
    view: Vec<String>,                              // nomes filtrados (lista)
    details: LruCache<String, service::Detail>,    // cache detalhes
    sprites: LruCache<String, Vec<u8>>,             // cache de bytes da sprite
    selected: i32,                                  // índice selecionado
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

// ícones dos tipos
fn type_icon(t: &str) -> Image {
    Image::load_from_path(Path::new(&format!("ui/imagens/tipos/{t}.png"))).unwrap()
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

fn set_rows_from_names(app: &App, names: &[String]) {
    let rows: Vec<PokemonRow> = names
        .iter()
        .map(|n| PokemonRow { name: cap_words_and_spaces(n).into() })
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
            icon: type_icon(t)
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
        full: Vec::new(),
        view: Vec::new(),
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
    let pkclone = poke_service.clone();
    let h = handle.clone();
    let app_w_2 = app_w.clone();
    slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
        if let Some(s) = app_w_2.upgrade() {
            s.set_splash(false);
        }
    });
    app.on_request_load(move || {
        let app_w = app_w.clone();
        let state_list = state_list.clone();
        let pkclone = pkclone.clone();
        h.spawn(async move {
            let res = pkclone.fetch_pokemon_list().await;
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
        if let Some(app) = app_w.upgrade() {
            app.set_carregando(true);
            set_detail_empty(&app);
            app.set_visualiza_pokemon(true);
            app.set_selected_index(idx);
        }

        let name = { let st = state_sel.lock().unwrap(); st.view.get(idx as usize).cloned() };
        let Some(name) = name else { return };

        if let Some(app) = app_w.upgrade() {
            let (maybe_d, maybe_bytes) = {
                let mut st = state_sel.lock().unwrap();
                (st.details.get(&name).cloned(), st.sprites.get(&name).cloned())
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
            let dres = pkclone.fetch_pokemon_detail(&name).await;
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
                                st.details.put(name.clone(), d.clone());
                                if let Some(b) = &sprite_bytes {
                                    st.sprites.put(name.clone(), b.clone());
                                }
                            }
                            let ui_detail = make_detail_for_ui(&d, sprite_bytes.as_deref());
                            app.set_detail(ui_detail);
                            app.set_carregando(false);
                        }
                        None => {
                            set_detail_error(&app, "Falha ao carregar detalhes");
                            app.set_carregando(false);
                        },
                    }
                }
            }).ok();
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
    if let Some(app) = app_c.upgrade() { app.invoke_request_load(); }

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
    let pkclone = poke_service.clone();
    let app_w_2 = app_w.clone();
    slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
        if let Some(s) = app_w_2.upgrade() {
            s.set_splash(false);
        }
    });
    app.on_request_load(move || {
        let app_w = app_w.clone();
        let state_list = state_list.clone();
        let pkclone = pkclone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let res = pkclone.fetch_pokemon_list().await;
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
                }
            }).ok();
        });
    });

    // Clique: baixa detalhes + sprite (async) e mantém seleção
    let app_w = app.as_weak();
    let state_sel = state.clone();
    let pkclone = poke_service.clone();
    app.on_select(move |idx| {
        if idx < 0 { return; }
        { state_sel.lock().unwrap().selected = idx; }
        if let Some(app) = app_w.upgrade() {
            app.set_carregando(true);
            set_detail_empty(&app);
            app.set_visualiza_pokemon(true);
            app.set_selected_index(idx);
        }

        let name = { let st = state_sel.lock().unwrap(); st.view.get(idx as usize).cloned() };
        let Some(name) = name else { return };

        if let Some(app) = app_w.upgrade() {
            let (maybe_d, maybe_bytes) = {
                let mut st = state_sel.lock().unwrap();
                (st.details.get(&name).cloned(), st.sprites.get(&name).cloned())
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
            let dres = pkclone.fetch_pokemon_detail(&name).await;
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
                                st.details.put(name.clone(), d.clone());
                                if let Some(b) = &sprite_bytes {
                                    st.sprites.put(name.clone(), b.clone());
                                }
                            }
                            let ui_detail = make_detail_for_ui(&d, sprite_bytes.as_deref());
                            app.set_detail(ui_detail);
                            app.set_carregando(false);
                        }
                        None =>{
                            set_detail_error(&app, "Falha ao carregar detalhes");
                            app.set_carregando(false);
                        },
                    }
                }
            }).ok();
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
    if let Some(app) = app_c.upgrade() { app.invoke_request_load(); }

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