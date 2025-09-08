use helpers::*;
use slint::{Brush, Color, ModelRc, SharedString, VecModel};
use std::{
    collections::{HashMap, HashSet, VecDeque}, sync::{Arc, Mutex}
};

mod helpers;
slint::include_modules!(); // App, PokemonRow, PokemonDetail, TypeTag, StatBar...

include!(concat!(env!("OUT_DIR"), "/pokemon_list.rs")); // add lista constante com todos os pokémons

type StateHandle = Arc<Mutex<State>>;

/// Estado compartilhado da aplicação
struct State {
    view: Vec<Pokemon>,
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

fn to_slint(g: &EvolutionGraph) -> (Vec<EvolutionNodeSlint>, Vec<EvolutionEdgeSlint>, Vec<EvolutionLineSlint>, i32, i32) {
    // Agrupa ids por stage para calcular row
    let mut stage_lists = g.stages.clone(); // Vec<Vec<u32>>
    for list in &mut stage_lists {
        list.sort();
    }
    // Detecta cadeias lineares (pai único -> filho único) e aplica deslocamento apenas ao nó filho
    use std::collections::HashMap as _HashMap;
    let mut parent_count: _HashMap<u32, u32> = _HashMap::new();
    let mut child_count: _HashMap<u32, u32> = _HashMap::new();
    for e in &g.edges { *parent_count.entry(e.to).or_insert(0) += 1; *child_count.entry(e.from).or_insert(0) += 1; }
    const LINEAR_REDUCE: f32 = 40.0;
    let mut node_dx: _HashMap<u32, f32> = _HashMap::new();
    for e in &g.edges {
        if child_count.get(&e.from).copied().unwrap_or(0) == 1 && parent_count.get(&e.to).copied().unwrap_or(0) == 1 {
            node_dx.entry(e.to).or_insert(-LINEAR_REDUCE);
        }
    }

    let mut nodes_out = Vec::new();
    for list in &stage_lists {
        for (row, id) in list.iter().enumerate() {
            let node = g.nodes.iter().find(|n| n.id == *id).unwrap();
            // tiny icons list
            let types_for_node = POKEMON_LIST.iter()
                .find(|p| p.species_id == node.id)
                .map(|p| {
                    let v: Vec<TypeTag> = p.types.iter().map(|t| TypeTag { label: type_label_pt(t).into(), bg: type_color(t), icon: type_icon(t) }).collect();
                    ModelRc::new(VecModel::from(v))
                })
                .unwrap_or_else(|| ModelRc::new(VecModel::from(Vec::<TypeTag>::new())));
            let dx = node_dx.get(&node.id).copied().unwrap_or(0.0);
            nodes_out.push(EvolutionNodeSlint {
                id: node.id as i32,
                name: node.name.into(),
                stage: node.stage as i32,
                row: row as i32,
                artwork: artwork_img(node.id),
                method: if node.stage == 0 { "".into() } else { g.edges.iter().find(|e| e.to == node.id).map(|e| e.method.into()).unwrap_or_else(|| "".into()) },
                types: types_for_node,
                dx: dx as f32,
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
    // Linhas (coordenadas simples em grid)
    const COL_W: f32 = 140.0; // largura de coluna
    const ROW_H: f32 = 220.0; // node-h (200) + 20 spacing (maior para texto longo)
    const NODE_W: f32 = 96.0; // largura sprite
    const NODE_H: f32 = 200.0; // altura aumentada para evitar sobreposição de texto
    let mut idx_map: std::collections::HashMap<(i32, i32), (f32, f32)> = std::collections::HashMap::new();
    // offset horizontal para alinhar com content_layout (primeira coluna começa em 0 dentro do layout)
    // offset para alinhar com centralização do layout (será calculado na UI, então aqui mantemos 0)
    let x_offset: f32 = 0.0;
    for n in &nodes_out {
        // centro calculado com deslocamento dx
        let cx = n.stage as f32 * COL_W + n.dx + NODE_W / 2.0 + x_offset;
        let cy = n.row as f32 * ROW_H + NODE_H / 2.0;
        idx_map.insert((n.id, n.stage), (cx, cy));
    }
    let mut lines_out: Vec<EvolutionLineSlint> = Vec::new();
    use std::collections::HashMap;
    let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();
    for e in &g.edges { children_map.entry(e.from).or_default().push(e.to); }

    for (parent, kids) in children_map.into_iter() {
        // coordenadas do pai
        let from_stage = g.nodes.iter().find(|n| n.id == parent).map(|n| n.stage as i32).unwrap_or(0);
    let (fx_c, fy) = match idx_map.get(&(parent as i32, from_stage)) { Some(c) => *c, None => continue };
    let fx_right = fx_c + NODE_W / 2.0; // centro + metade = borda direita
    const BRANCH_GAP: f32 = 7.0; // leve ajuste com redução
    const H_SEG: f32 = 9.0; // horizontais um pouco menores
    // Ajuste adicional para linhas horizontais diretas: usar segmento curto centralizado
    const DIRECT_SHORT: f32 = 18.0; // comprimento total desejado para linhas diretas

        if kids.len() == 1 {
            let child = kids[0];
            let to_stage = g.nodes.iter().find(|n| n.id == child).map(|n| n.stage as i32).unwrap_or(from_stage + 1);
            if let Some(&(tx, ty)) = idx_map.get(&(child as i32, to_stage)) {
                let child_left = tx - NODE_W / 2.0;
                // Se mesma linha (y próximo), linha direta horizontal com gap
                if (fy - ty).abs() < 4.0 {
                    // linha direta encurtada: desenha apenas perto do pai e perto do filho? Para simplicidade, um único segmento curto a partir da borda do pai
                    let desired = DIRECT_SHORT.min(child_left - fx_right - BRANCH_GAP);
                    let x2 = fx_right + desired.max(4.0); // garante mínimo
                    lines_out.push(EvolutionLineSlint { x1: fx_right, y1: fy, x2, y2: fy, method: "".into() });
                } else {
                    // L-shaped: horizontal até trunk, vertical, horizontal até filho
                    let trunk_start_x = fx_right;
                    // offset menor para reduzir "saída" horizontal antes da curva
                    let trunk_x = trunk_start_x + H_SEG; // usa comprimento padrão
                    // horizontal saída
                    lines_out.push(EvolutionLineSlint { x1: trunk_start_x, y1: fy, x2: trunk_x, y2: fy, method: "".into() });
                    // vertical
                    lines_out.push(EvolutionLineSlint { x1: trunk_x, y1: fy.min(ty), x2: trunk_x, y2: fy.max(ty), method: "".into() });
                    // horizontal entrada (gap)
                    let mut branch_end = child_left;
                    if branch_end - trunk_x > BRANCH_GAP + H_SEG { branch_end = trunk_x + H_SEG; }
                    else if branch_end - trunk_x > BRANCH_GAP { branch_end -= BRANCH_GAP; }
                    lines_out.push(EvolutionLineSlint { x1: trunk_x, y1: ty, x2: branch_end, y2: ty, method: "".into() });
                }
            }
            continue;
        }

        // múltiplos filhos -> tronco (horizontal + vertical) + ramificações horizontais
        let mut child_coords: Vec<(f32,f32)> = Vec::new();
        for kid in &kids {
            let to_stage = g.nodes.iter().find(|n| n.id == *kid).map(|n| n.stage as i32).unwrap_or(from_stage + 1);
            if let Some(&(tx, ty)) = idx_map.get(&(*kid as i32, to_stage)) { child_coords.push((tx, ty)); }
        }
        if child_coords.is_empty() { continue; }
        child_coords.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());
        let min_y = child_coords.first().unwrap().1;
        let max_y = child_coords.last().unwrap().1;

    let trunk_start_x = fx_right;
        let trunk_x = trunk_start_x + H_SEG; // uniformiza segmento esquerdo

        // tronco horizontal
        lines_out.push(EvolutionLineSlint { x1: trunk_start_x, y1: fy, x2: trunk_x, y2: fy, method: "".into() });
        // tronco vertical
        if (max_y - min_y).abs() > 4.0 {
            lines_out.push(EvolutionLineSlint { x1: trunk_x, y1: min_y, x2: trunk_x, y2: max_y, method: "".into() });
        }
        // ramos horizontais
        for (tx, ty) in child_coords {
            let mut branch_end = tx - NODE_W / 2.0; // entrada esquerda do filho
            let max_end = trunk_x + H_SEG; // limita horizontal direita
            if branch_end - trunk_x > BRANCH_GAP + H_SEG { branch_end = max_end; }
            else if branch_end - trunk_x > BRANCH_GAP { branch_end -= BRANCH_GAP; }
            lines_out.push(EvolutionLineSlint { x1: trunk_x, y1: ty, x2: branch_end, y2: ty, method: "".into() });
        }
    }
    let max_stage = g.stages.len() as i32 - 1;
    let max_rows = stage_lists.iter().map(|l| l.len()).max().unwrap_or(0) as i32;
    (nodes_out, edges_out, lines_out, max_stage, max_rows)
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

fn make_detail_for_ui(detail: &Pokemon) -> PokemonDetail {
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
    let artwork_img = if detail.species_id > 0{
        artwork_img(detail.species_id)
    }else{
        slint::Image::default()
    };
    // Evolução
    let graph = build_graph(detail.species_id);
    let (nodes_model, edges_model, lines_model, max_stage, max_rows) = to_slint(&graph);
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
        lines: ModelRc::new(VecModel::from(lines_model)),
    max_stage,
    max_rows,
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
        lines: ModelRc::new(VecModel::from(Vec::<EvolutionLineSlint>::new())),
        max_stage: 0,
    max_rows: 0,
    });
}

// =================== Estado base ===================
fn wire_app_common(app: &App) -> StateHandle {
    let state = Arc::new(Mutex::new(State {
        view: POKEMON_LIST.iter().copied().collect(),
        selected: -1,
    }));
    app.set_filter(SharedString::from(""));
    app.set_selected_index(-1);
    set_detail_error(app, "");
    state
}

// =================== Lógica comum de handlers ===================
fn setup_common_handlers(app: &App, state: &StateHandle) {
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
    app.on_select(move |idx| {
        if idx < 0 {
            return;
        }
        state_sel.lock().unwrap().selected = idx;
        if let Some(app) = app_w.upgrade() {
            app.set_carregando(true); // mostra loading rápido
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
            if let Some(detail) = POKEMON_LIST.iter().find(|p| p.species_id == id_pokemon) {
                let ui_detail = make_detail_for_ui(detail);
                app.set_detail(ui_detail);
            } else {
                set_detail_error(&app, "Pokémon não encontrado");
            }
            app.set_carregando(false);
        }
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
    let app = App::new()?;
    let state = wire_app_common(&app);
    setup_common_handlers(&app, &state);
    if let Some(app0) = app.as_weak().upgrade() { app0.invoke_request_load(); }
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
    setup_common_handlers(&app, &state);
    if let Some(app0) = app.as_weak().upgrade() { app0.invoke_request_load(); }
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
