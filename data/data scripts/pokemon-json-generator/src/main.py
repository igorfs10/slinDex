import pandas as pd
import os
import json


def main():    
    print('Iniciando script Pokémon')
    folder_path = r'../../../../data'
    # Carregar arquivos principais
    pokemon = pd.read_csv(os.path.join(folder_path, 'pokemon.csv'))
    stats = pd.read_csv(os.path.join(folder_path, 'pokemon_stats.csv'))
    abilities = pd.read_csv(os.path.join(folder_path, 'pokemon_abilities.csv'))
    species = pd.read_csv(os.path.join(folder_path, 'pokemon_species.csv'))
    colors = pd.read_csv(os.path.join(folder_path, 'pokemon_colors.csv'))
    abilities_names = pd.read_csv(os.path.join(folder_path, 'abilities.csv'))
    species_names_en = pd.read_csv(os.path.join(folder_path, 'pokemon_species_names.csv'))
    ability_names_en = pd.read_csv(os.path.join(folder_path, 'ability_names.csv'))
    evo = pd.read_csv(os.path.join(folder_path, 'pokemon_evolution.csv'))
    evo_triggers = pd.read_csv(os.path.join(folder_path, 'evolution_triggers.csv'))
    item_names = pd.read_csv(os.path.join(folder_path, 'item_names.csv'))
    move_names = pd.read_csv(os.path.join(folder_path, 'move_names.csv'))
    type_names = pd.read_csv(os.path.join(folder_path, 'type_names.csv'))
    location_names = pd.read_csv(os.path.join(folder_path, 'location_names.csv'))
    pokemon_types = pd.read_csv(os.path.join(folder_path, 'pokemon_types.csv'))
    # Pega apenas nomes em ingleês (local_language_id == 9)
    location_id_to_name = location_names[location_names['local_language_id'] == 9].set_index('location_id')['name'].to_dict()
    type_names_pt = type_names[type_names['local_language_id'] == 9]
    item_id_to_name = item_names[item_names['local_language_id'] == 9].set_index('item_id')['name'].to_dict()
    move_id_to_name = move_names[move_names['local_language_id'] == 9].set_index('move_id')['name'].to_dict()
    trigger_id_to_name = evo_triggers.set_index('id')['identifier'].to_dict()
    species_id_to_name = species_names_en[species_names_en['local_language_id'] == 9].set_index('pokemon_species_id')['name'].to_dict()
    ability_id_to_name = ability_names_en[ability_names_en['local_language_id'] == 9].set_index('ability_id')['name'].to_dict()
    type_id_to_name = dict(zip(type_names_pt['type_id'], type_names_pt['name']))
    types_grouped = pokemon_types.groupby('pokemon_id')['type_id'].apply(list).to_dict()
    # Dicionário de para: inglês -> português
    type_en_to_pt = {
        'normal': 'normal',
        'fighting': 'lutador',
        'flying': 'voador',
        'poison': 'venenoso',
        'ground': 'terrestre',
        'rock': 'pedra',
        'bug': 'inseto',
        'ghost': 'fantasma',
        'steel': 'aço',
        'fire': 'fogo',
        'water': 'água',
        'grass': 'grama',
        'electric': 'elétrico',
        'psychic': 'psíquico',
        'ice': 'gelo',
        'dragon': 'dragão',
        'dark': 'noturno',
        'fairy': 'fada',
    }

    # Grafo de evolução: para cada cadeia, lista de species_id
    chain_to_species = species.groupby('evolution_chain_id')['id'].apply(list).to_dict()
    # Mapeia species_id -> filhos (evoluções)
    species_evolves_to = {}
    for _, row in species.iterrows():
        sid = int(row['id'])
        prev = row['evolves_from_species_id']
        if pd.notnull(prev):
            prev = int(prev)
            species_evolves_to.setdefault(prev, []).append(sid)

    # Status base: pivotar para colunas
    stats_pivot = stats.pivot(index='pokemon_id', columns='stat_id', values='base_stat')
    stats_pivot.columns = ['HP', 'Atk', 'Def', 'SpAtk', 'SpDef', 'Speed']
    stats_pivot = stats_pivot.reset_index()

    # Habilidades: juntar por id e agrupar
    abilities = abilities.merge(abilities_names, left_on='ability_id', right_on='id')
    abilities_sorted = abilities.sort_values(['pokemon_id', 'slot'])
    abilities_grouped = abilities_sorted.groupby('pokemon_id').apply(
        lambda x: pd.Series({
            'ability1': ability_id_to_name.get(x[x['slot'] == 1]['ability_id'].iloc[0], '') if (x['slot'] == 1).any() else '',
            'ability2': ability_id_to_name.get(x[x['slot'] == 2]['ability_id'].iloc[0], '') if (x['slot'] == 2).any() else '',
            'hidden': ability_id_to_name.get(x[x['is_hidden'] == 1]['ability_id'].iloc[0], '') if (x['is_hidden'] == 1).any() else ''
        })
    , include_groups=False).reset_index()

    # Cor: juntar species com colors
    species = species.merge(colors, left_on='color_id', right_on='id', suffixes=('', '_color'))
    species = species.rename(columns={'identifier_color': 'color_name'})

    # Montar DataFrame final
    df = species.merge(pokemon[['species_id', 'height', 'weight']], left_on='id', right_on='species_id', how='left')
    df = df.merge(stats_pivot, left_on='id', right_on='pokemon_id', how='left')
    df = df.merge(abilities_grouped, left_on='id', right_on='pokemon_id', how='left')
    df['weight_kg'] = df['weight'] / 10
    df['height_m'] = df['height'] / 10
    # Remover duplicações: manter apenas a primeira linha de cada species_id
    df = df.drop_duplicates(subset=['id'])

    result = []
    for _, row in df.iterrows():
        species_id = int(row['id'])
        poke_types_ids = types_grouped.get(species_id, [])
        poke_types_names = [type_id_to_name.get(tid, str(tid)) for tid in poke_types_ids]
        # Filtra apenas formas padrão (ignora formas alternativas, mega, gigantamax, etc)
        if species_id >= 10000:
            continue
        # Só processa se species_id for igual ao species_id base (ignora formas alternativas)
        if 'form' in row and pd.notnull(row['form']) and row['form'] != '':
            continue
        chain_id = row['evolution_chain_id'] if 'evolution_chain_id' in row else None
        evolutions = []
        evol_ids = set()
        for _, evo_species in species[species['evolves_from_species_id'] == species_id].iterrows():
            evo_sid = int(evo_species['id'])
            if evo_sid in evol_ids:
                continue
            evol_ids.add(evo_sid)
            evo_row = evo[evo['evolved_species_id'] == evo_sid]
            method = None
            if not evo_row.empty:
                evo_row = evo_row.iloc[0]
                trigger = trigger_id_to_name.get(evo_row['evolution_trigger_id'], '')
                turn_upside_down = False
                if 'turn_upside_down' in evo_row and pd.notnull(evo_row['turn_upside_down']):
                    turn_upside_down = int(evo_row['turn_upside_down']) == 1
                if turn_upside_down:
                    lvl_txt = ''
                    if 'minimum_level' in evo_row and pd.notnull(evo_row['minimum_level']):
                        lvl_txt = f' (lvl {int(evo_row["minimum_level"])}+)'
                    method = f'virar de ponta cabeça{lvl_txt}'
                elif trigger == 'level-up':
                    parts = []
                    if 'minimum_level' in evo_row and pd.notnull(evo_row['minimum_level']):
                        parts.append(f'lvl {int(evo_row["minimum_level"])}')
                    if 'minimum_happiness' in evo_row and pd.notnull(evo_row['minimum_happiness']) and int(evo_row['minimum_happiness']) > 0:
                        parts.append(f'amizade {int(evo_row["minimum_happiness"])}')
                    if 'minimum_affection' in evo_row and pd.notnull(evo_row['minimum_affection']) and int(evo_row['minimum_affection']) > 0:
                        parts.append(f'afeição {int(evo_row["minimum_affection"])}')
                    if 'minimum_beauty' in evo_row and pd.notnull(evo_row['minimum_beauty']) and int(evo_row['minimum_beauty']) > 0:
                        parts.append(f'beleza {int(evo_row["minimum_beauty"])}')
                    if 'location_id' in evo_row and pd.notnull(evo_row['location_id']):
                        location_id = int(evo_row['location_id'])
                        location_name = location_id_to_name.get(location_id, f'id {location_id}')
                        parts.append(f'local especial: {location_name}')
                    if 'time_of_day' in evo_row and pd.notnull(evo_row['time_of_day']) and evo_row['time_of_day']:
                        tod = evo_row['time_of_day']
                        if tod == 'day':
                            tod_pt = 'dia'
                        elif tod == 'night':
                            tod_pt = 'noite'
                        else:
                            tod_pt = tod
                        parts.append(f'de {tod_pt}')
                    if 'known_move_id' in evo_row and pd.notnull(evo_row['known_move_id']):
                        move_name = move_id_to_name.get(int(evo_row['known_move_id']), '')
                        parts.append(f'sabendo golpe {move_name}')
                    if 'known_move_type_id' in evo_row and pd.notnull(evo_row['known_move_type_id']):
                        type_id = int(evo_row['known_move_type_id'])
                        type_name = type_id_to_name.get(type_id, str(type_id))
                        type_name_pt = type_en_to_pt.get(type_name.lower(), type_name)
                        parts.append(f'sabendo golpe tipo {type_name_pt}')
                    if 'gender_id' in evo_row and pd.notnull(evo_row['gender_id']):
                        gender_id = int(evo_row['gender_id'])
                        if gender_id == 1:
                            parts.append('fêmea')
                        elif gender_id == 2:
                            parts.append('macho')
                    if parts:
                        method = 'lvl up: ' + ', '.join(parts)
                    else:
                        method = 'lvl up'
                elif trigger == 'trade':
                    if 'held_item_id' in evo_row and pd.notnull(evo_row['held_item_id']):
                        item_name = item_id_to_name.get(int(evo_row['held_item_id']), '')
                        method = f'troca segurando {item_name}'
                    else:
                        method = 'troca'
                elif trigger == 'use-item' and 'trigger_item_id' in evo_row and pd.notnull(evo_row['trigger_item_id']):
                    item_name = item_id_to_name.get(int(evo_row['trigger_item_id']), '')
                    gender_txt = ''
                    if 'gender_id' in evo_row and pd.notnull(evo_row['gender_id']):
                        gender_id = int(evo_row['gender_id'])
                        if gender_id == 1:
                            gender_txt = ', fêmea'
                        elif gender_id == 2:
                            gender_txt = ', macho'
                    method = f'item: {item_name}{gender_txt}'
                else:
                    method = trigger if trigger else 'desconhecido'
            evolutions.append({
                'to': evo_sid,
                'method': method
            })
        poke_obj = {
            'species_id': species_id,
            'name': species_id_to_name.get(species_id, row['identifier']),
            'types': poke_types_names,
            'HP': int(row['HP']) if not pd.isnull(row['HP']) else None,
            'Atk': int(row['Atk']) if not pd.isnull(row['Atk']) else None,
            'Def': int(row['Def']) if not pd.isnull(row['Def']) else None,
            'SpAtk': int(row['SpAtk']) if not pd.isnull(row['SpAtk']) else None,
            'SpDef': int(row['SpDef']) if not pd.isnull(row['SpDef']) else None,
            'Speed': int(row['Speed']) if not pd.isnull(row['Speed']) else None,
            'ability1': row['ability1'] if not pd.isnull(row['ability1']) else '',
            'ability2': row['ability2'] if not pd.isnull(row['ability2']) else '',
            'hidden': row['hidden'] if not pd.isnull(row['hidden']) else '',
            'weight_kg': float(row['weight_kg']) if not pd.isnull(row['weight_kg']) else None,
            'height_m': float(row['height_m']) if not pd.isnull(row['height_m']) else None,
            'color': row['color_name'] if not pd.isnull(row['color_name']) else '',
            'evolutions': evolutions
        }
        result.append(poke_obj)

    print('Salvando arquivo JSON...')
    with open('../../../../data_json/pokemons.json', 'w', encoding='utf-8') as f:
        json.dump(result, f, ensure_ascii=False, indent=2)
    print('JSON gerado: pokemons.json')

if __name__ == "__main__":
    main()