use helpers::*;
use lru::LruCache;
use slint::{Brush, Color, ModelRc, SharedString, VecModel};
use std::{
    collections::{HashMap, HashSet, VecDeque}, num::NonZeroUsize, sync::{Arc, Mutex}
};

mod helpers;
mod service;
slint::include_modules!(); // App, PokemonRow, PokemonDetail, TypeTag, StatBar...

include!(concat!(env!("OUT_DIR"), "/pokemon_list.rs")); // add lista constante com todos os pokémons

type StateHandle = Arc<Mutex<State>>;

/// Estado compartilhado da aplicação
struct State {
    view: Vec<Pokemon>,
    sprites: LruCache<u32, Vec<u8>>, // cache de bytes da sprite
    selected: i32,                   // índice selecionado
}

#[derive(Debug, Copy, Clone)]
pub struct EvolutionEdge {
    pub from: u32,
    pub to: u32,
    pub method: &'static str,
}

#[derive(Debug)]
pub struct EvolutionNode {
    pub id: u32,
    pub name: &'static str,
    pub stage: u32,
    pub parents: Vec<u32>,
    pub children: Vec<u32>,
}

pub struct EvolutionGraph {
    pub nodes: Vec<EvolutionNode>,
    pub edges: Vec<EvolutionEdge>,
    // opcional: agrupado por estágio pra UI rápida
    pub stages: Vec<Vec<u32>>, // ids por estágio
}


fn build_graph(selected_id: u32) -> EvolutionGraph {
    let base = find_base_id(selected_id);
    let mut queue = VecDeque::new();
    queue.push_back((base, 0u32));

    let mut stage_map: HashMap<u32, u32> = HashMap::new();
    let mut nodes_map: HashMap<u32, EvolutionNode> = HashMap::new();
    let mut edges: Vec<EvolutionEdge> = Vec::new();
    let mut seen_edge = HashSet::new();

    while let Some((current, stage)) = queue.pop_front() {
        // menor estágio vence (evita loops)
        if stage_map.get(&current).map(|&s| stage < s).unwrap_or(true) {
            stage_map.insert(current, stage);
        }
        let p = match POKEMON_LIST.iter().find(|pp| pp.species_id == current) {
            Some(x) => x,
            None => continue,
        };
        nodes_map.entry(current).or_insert_with(|| EvolutionNode {
            id: p.species_id,
            name: p.name,
            stage,
            parents: Vec::new(),
            children: Vec::new(),
        });

        for ev in p.evolutions {
            let edge_key = (p.species_id, ev.to);
            if seen_edge.insert(edge_key) {
                edges.push(EvolutionEdge {
                    from: p.species_id,
                    to: ev.to,
                    method: ev.method,
                });
            }
            // Atualiza children / parents (cria placeholder se necessário)
            nodes_map
                .entry(p.species_id)
                .and_modify(|n| {
                    if !n.children.contains(&ev.to) { n.children.push(ev.to); }
                });

            nodes_map
                .entry(ev.to)
                .or_insert(EvolutionNode {
                    id: ev.to,
                    name: POKEMON_LIST
                        .iter()
                        .find(|pp| pp.species_id == ev.to)
                        .map(|pp| pp.name)
                        .unwrap_or("?"),
                    stage: stage + 1, // inicial
                    parents: vec![p.species_id],
                    children: Vec::new(),
                })
                .parents
                .push(p.species_id);

            // enfileira próximo nível
            queue.push_back((ev.to, stage + 1));
        }
    }

    // Ajusta stage final em nós a partir de stage_map
    for (id, st) in &stage_map {
        if let Some(n) = nodes_map.get_mut(id) {
            n.stage = *st;
        }
    }

    // Agrupa por estágio
    let max_stage = stage_map.values().copied().max().unwrap_or(0);
    let mut stages: Vec<Vec<u32>> = vec![Vec::new(); (max_stage + 1) as usize];
    for n in nodes_map.values() {
        stages[n.stage as usize].push(n.id);
    }
    for v in &mut stages { v.sort(); }

    EvolutionGraph {
        nodes: nodes_map.into_values().collect(),
        edges,
        stages,
    }
}

fn to_slint(g: &EvolutionGraph) -> (Vec<EvolutionNodeSlint>, Vec<EvolutionEdgeSlint>, i32) {
    // Agrupa ids por stage para calcular row
    let mut stage_lists = g.stages.clone(); // Vec<Vec<u32>>
    for list in &mut stage_lists {
        list.sort();
    }
    let mut nodes_out = Vec::new();
    for list in &stage_lists {
        for (row, id) in list.iter().enumerate() {
            let node = g.nodes.iter().find(|n| n.id == *id).unwrap();
            nodes_out.push(EvolutionNodeSlint {
                id: node.id as i32,
                name: node.name.into(),
                stage: node.stage as i32,
                row: row as i32,
                artwork: artwork_img(node.id),
                method: if node.stage == 0 {
                    "".into()
                } else {
                    // pega método a partir de alguma edge que chega aqui
                    g.edges
                        .iter()
                        .find(|e| e.to == node.id)
                        .map(|e| e.method.into())
                        .unwrap_or_else(|| "".into())
                },
            });
        }
    }
    let edges_out = g
        .edges
        .iter()
        .map(|e| EvolutionEdgeSlint {
            from: e.from as i32,
            to: e.to as i32,
            method: e.method.into(),
        })
        .collect();
    let max_stage = g.stages.len() as i32 - 1;
    (nodes_out, edges_out, max_stage)
}

// Dado um species_id, encontra o base_id (primeira forma da cadeia)
pub fn find_base_id(mut id: u32) -> u32 {
    loop {
        // procura quem tem "id" na lista de evoluções
        if let Some(prev) = POKEMON_LIST.iter().find(|p| p.evolutions.iter().any(|e| e.to == id)) {
            id = prev.species_id;
        } else {
            return id; // ninguém evolui para ele → é base
        }
    }
}

// =================== UI Utils ===================
fn set_rows_from_pokemon(app: &App, pokemons: &[Pokemon]) {
    let rows: Vec<PokemonRow> = pokemons
        .iter()
        .map(|pokemon| PokemonRow {
            name: format!("{} - {}", pokemon.species_id, pokemon.name).into(),
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
                item.species_id.to_string().contains(&filter_lower)
                    || item.name.to_lowercase().contains(&filter_lower)
            })
            .collect();
    }
    let filtered_list: Vec<Pokemon> = POKEMON_LIST
        .iter()
        .copied()
        .filter(|item| {
            item.species_id.to_string().contains(&filter_lower)
                || item.name.to_lowercase().contains(&filter_lower)
        })
        .collect();
    app.set_selected_index(-1);
    set_rows_from_pokemon(app, &filtered_list);
}

fn make_detail_for_ui(detail: &Pokemon, artwork_bytes: Option<&[u8]>) -> PokemonDetail {
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
    let total: i32 = detail.hp as i32
        + detail.atk as i32
        + detail.def as i32
        + detail.sp_atk as i32
        + detail.sp_def as i32
        + detail.speed as i32;

    // Artwork
    let artwork_img = artwork_bytes
        .and_then(|b| png_to_image(b).ok())
        .unwrap_or_default();
    // Evolução
    let graph = build_graph(detail.species_id);
    let (nodes_model, edges_model, max_stage) = to_slint(&graph);
    PokemonDetail {
        name: detail.name.into(),
        id: detail.species_id as i32,
        height: detail.height_m as f32,
        weight: detail.weight_kg as f32,
        types: types_model,
        artwork: artwork_img,
        hp: detail.hp as i32,
        specialAttack: detail.sp_atk as i32,
        specialDefense: detail.sp_def as i32,
        attack: detail.atk as i32,
        defense: detail.def as i32,
        speed: detail.speed as i32,
        total,
        ability1: detail.ability1.into(),
        ability2: detail.ability2.into(),
        hiddenAbility: detail.hidden.into(),
        error: "".into(),
        color: pokemon_color(detail.color),
        nodes: ModelRc::new(VecModel::from(nodes_model)),
        edges: ModelRc::new(VecModel::from(edges_model)),
        max_stage,
    }
}

fn set_detail_error(app: &App, msg: &str) {
    app.set_detail(PokemonDetail {
        name: "".into(),
        id: 0,
        height: 0.0,
        weight: 0.0,
        types: ModelRc::new(VecModel::from(Vec::<TypeTag>::new())),
        hp: 0,
        specialAttack: 0,
        specialDefense: 0,
        attack: 0,
        defense: 0,
        speed: 0,
        artwork: slint::Image::default(),
        total: 0,
        ability1: "".into(),
        ability2: "".into(),
        hiddenAbility: "".into(),
        error: msg.into(),
        color: Brush::from(Color::from_argb_encoded(0x00000000)),
        nodes: ModelRc::new(VecModel::from(Vec::<EvolutionNodeSlint>::new())),
        edges: ModelRc::new(VecModel::from(Vec::<EvolutionEdgeSlint>::new())),
        max_stage: 0,
    });
}

fn set_detail_empty(app: &App) {
    app.set_detail(PokemonDetail {
        name: "Carregando...".into(),
        id: 0,
        height: 0.0,
        weight: 0.0,
        types: ModelRc::new(VecModel::from(Vec::<TypeTag>::new())),
        hp: 0,
        specialAttack: 0,
        specialDefense: 0,
        attack: 0,
        defense: 0,
        speed: 0,
        artwork: slint::Image::default(),
        total: 0,
        ability1: "".into(),
        ability2: "".into(),
        hiddenAbility: "".into(),
        error: "".into(),
        color: Brush::from(Color::from_argb_encoded(0x00000000)),
        nodes: ModelRc::new(VecModel::from(Vec::<EvolutionNodeSlint>::new())),
        edges: ModelRc::new(VecModel::from(Vec::<EvolutionEdgeSlint>::new())),
        max_stage: 0,
    });
}

// =================== Estado base ===================
fn wire_app_common(app: &App) -> StateHandle {
    let cap = NonZeroUsize::new(50).unwrap();
    let state = Arc::new(Mutex::new(State {
        view: POKEMON_LIST.iter().copied().collect(),
        sprites: LruCache::new(cap),
        selected: -1,
    }));
    app.set_filter(SharedString::from(""));
    app.set_selected_index(-1);
    set_detail_error(app, "");
    state
}

// =================== Async Fetch (compartilhado) ===================
async fn fetch_and_update(
    id_pokemon: u32,
    app_w2: slint::Weak<App>,
    state_sel2: StateHandle,
    poke_service: service::PokemonService,
) {
    let detail_result = POKEMON_LIST.iter().find(|p| p.species_id == id_pokemon);
    let (detail, sprite_bytes): (Option<Pokemon>, Option<Vec<u8>>) = match detail_result {
        Some(detail) => {
            let bytes = poke_service.fetch_image(detail.species_id).await.ok();
            (Some(*detail), bytes)
        }
        None => (None, None),
    };
    slint::invoke_from_event_loop(move || {
        if let Some(app) = app_w2.upgrade() {
            match detail {
                Some(detail) => {
                    let mut state = state_sel2.lock().unwrap();
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
}

// =================== Lógica comum de handlers ===================
fn setup_common_handlers<SpawnFn>(
    app: &App,
    state: &StateHandle,
    poke_service: service::PokemonService,
    spawn_fetch: SpawnFn,
) where
    SpawnFn: Fn(u32, slint::Weak<App>, StateHandle, service::PokemonService) + 'static + Clone,
{
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
        if let Some(app) = app_w.upgrade() {
            let mut state = state_list.lock().unwrap();
            state.selected = -1;
            app.set_selected_index(-1);
            set_rows_from_pokemon(&app, &POKEMON_LIST);
        }
    });

    // Seleção
    let state_sel = state.clone();
    let app_w = app.as_weak();
    let spawn_fetch_closure = spawn_fetch.clone();
    let poke_service_sel = poke_service.clone();
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
                Some(&pokemon) => pokemon.species_id,
                None => return,
            }
        };
        if let Some(app) = app_w.upgrade() {
            let maybe_bytes = {
                let mut state = state_sel.lock().unwrap();
                state.sprites.get(&id_pokemon).cloned()
            };
            if maybe_bytes.is_some() {
                let detail = POKEMON_LIST
                    .iter()
                    .find(|p| p.species_id == id_pokemon)
                    .expect("pokemon not found");
                let ui_detail = make_detail_for_ui(detail, maybe_bytes.as_deref());
                app.set_detail(ui_detail);
                app.set_carregando(false);
                return;
            }
        }
        // dispara fetch assíncrono
        spawn_fetch_closure(
            id_pokemon,
            app_w.clone(),
            state_sel.clone(),
            poke_service_sel.clone(),
        );
    });

    // Filtro
    let state_filter = state.clone();
    let app_c = app.as_weak();
    app.on_apply_filter(move |f: SharedString| {
        if let Some(app) = app_c.upgrade() {
            apply_filter(&app, &state_filter, f.as_str());
        }
    });
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

    setup_common_handlers(
        &app,
        &state,
        poke_service.clone(),
        move |id, app_w2, state_sel2, poke_service| {
            let handle = handle.clone();
            handle.spawn(fetch_and_update(id, app_w2, state_sel2, poke_service));
        },
    );

    // Inicial
    if let Some(app0) = app.as_weak().upgrade() {
        app0.invoke_request_load();
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

    setup_common_handlers(
        &app,
        &state,
        poke_service.clone(),
        move |id, app_w2, state_sel2, poke_service| {
            wasm_bindgen_futures::spawn_local(fetch_and_update(
                id,
                app_w2,
                state_sel2,
                poke_service,
            ));
        },
    );

    if let Some(app0) = app.as_weak().upgrade() {
        app0.invoke_request_load();
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
