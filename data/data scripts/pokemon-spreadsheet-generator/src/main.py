import pandas as pd
import os

def main():
    folder_path = r'd:\slinDex\data'
    pokemon = pd.read_csv(os.path.join(folder_path, 'pokemon.csv'))

    # Carregar arquivos
    stats = pd.read_csv(os.path.join(folder_path, 'pokemon_stats.csv'))
    abilities = pd.read_csv(os.path.join(folder_path, 'pokemon_abilities.csv'))
    species = pd.read_csv(os.path.join(folder_path, 'pokemon_species.csv'))
    colors = pd.read_csv(os.path.join(folder_path, 'pokemon_colors.csv'))
    abilities_names = pd.read_csv(os.path.join(folder_path, 'abilities.csv'))
    species_names_en = pd.read_csv(os.path.join(folder_path, 'pokemon_species_names.csv'))
    ability_names_en = pd.read_csv(os.path.join(folder_path, 'ability_names.csv'))

    # Status base: pivotar para colunas
    stats_pivot = stats.pivot(index='pokemon_id', columns='stat_id', values='base_stat')
    stats_pivot.columns = ['HP', 'Atk', 'Def', 'SpAtk', 'SpDef', 'Speed']
    stats_pivot = stats_pivot.reset_index()

    # Nomes bonitos dos pokémons (inglês)
    species_id_to_name = species_names_en[species_names_en['local_language_id'] == 9].set_index('pokemon_species_id')['name'].to_dict()
    # Nomes bonitos das habilidades (inglês)
    ability_id_to_name = ability_names_en[ability_names_en['local_language_id'] == 9].set_index('ability_id')['name'].to_dict()

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

    # Montar JSON final
    # Juntar species com pokemon.csv para pegar altura e peso
    df = species.merge(pokemon[['species_id', 'height', 'weight']], left_on='id', right_on='species_id', how='left')
    df = df.merge(stats_pivot, left_on='id', right_on='pokemon_id', how='left')
    df = df.merge(abilities_grouped, left_on='id', right_on='pokemon_id', how='left')
    df['weight_kg'] = df['weight'] / 10
    df['height_m'] = df['height'] / 10


    result = []
    for _, row in df.iterrows():
        species_id = int(row['id'])
        if species_id >= 10000:
            break
        poke_obj = {
            'species_id': species_id,
            'name': species_id_to_name.get(species_id, row['identifier']),
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
            'color': row['color_name'] if not pd.isnull(row['color_name']) else ''
        }
        result.append(poke_obj)

    # Salvar arquivo JSON
    import json
    with open('pokemon_planilha.json', 'w', encoding='utf-8') as f:
        json.dump(result, f, ensure_ascii=False, indent=2)
    print('JSON gerado: pokemon_planilha.json')


if __name__ == '__main__':
    main()