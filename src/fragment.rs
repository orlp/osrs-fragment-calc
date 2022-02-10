use anyhow::{Context, Ok, Result};
use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

const NUM_FRAG_ID: usize = 53;
const NUM_SET_EFFECT_ID: usize = 15;
const MANDATORY_THRESHOLD: i64 = 1000;
const FORBIDDEN_THRESHOLD: i64 = -1000000;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Fragment {
    image: String,
    name: String,
    category: String,
    set_effect_1: String,
    set_effect_2: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SetEffect {
    name: String,
    minimum: u8,
    maximum: u8,
}

fn get_score(
    combination: &[u8],
    has_sets: &[(u8, u8)],
    frag_weights: &[i64],
    set_weights: &[Vec<(u8, i64)>],
) -> (i64, [u8; NUM_SET_EFFECT_ID]) {
    let mut set_counter = [0u8; NUM_SET_EFFECT_ID];

    let mut score = 0;
    for frag in combination {
        score += frag_weights[*frag as usize];
        let (set1, set2) = has_sets[*frag as usize];
        set_counter[set1 as usize] += 1;
        set_counter[set2 as usize] += 1;
    }

    for (ctr, weights) in set_counter.into_iter().zip(set_weights) {
        if ctr >= 2 {
            score += weights
                .iter()
                .filter(|(threshold, _w)| ctr >= *threshold)
                .max_by_key(|(threshold, _w)| threshold)
                .map(|(_threshold, w)| *w)
                .unwrap_or(0);
        }
    }

    (score, set_counter)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Combination {
    score: i64,
    used_fragments: Vec<String>,
    activated_set_effects: Vec<(String, u8)>,
    alternatives: Option<(String, Vec<String>)>,
}

pub fn best_combinations(
    max_fragments: u8,
    frag_weights: HashMap<String, i64>,
    set_effect_weights: Vec<(String, u8, i64)>,
    max_results: usize,
) -> Result<Vec<Combination>> {
    // Load fragment and set data.
    let fragment_data = include_str!("fragments.json");
    let fragments: Vec<Fragment> = serde_json::from_str(fragment_data)?;
    let fragment_id: HashMap<String, usize> = fragments
        .iter()
        .enumerate()
        .map(|(id, frag)| (frag.name.clone(), id))
        .collect();
    let set_effect_data = include_str!("set_effects.json");
    let set_effects: Vec<SetEffect> = serde_json::from_str(set_effect_data)?;
    let set_effect_id: HashMap<String, usize> = set_effects
        .iter()
        .enumerate()
        .map(|(id, set_effect)| (set_effect.name.clone(), id))
        .collect();

    let get_fragment_id = |f| {
        fragment_id
            .get(f)
            .copied()
            .with_context(|| format!("unknown fragment {}", f))
    };
    let get_set_effect_id = |s| {
        set_effect_id
            .get(s)
            .copied()
            .with_context(|| format!("unknown set effect {}", s))
    };

    assert!(fragments.len() == NUM_FRAG_ID);
    assert!(set_effects.len() == NUM_SET_EFFECT_ID);

    // Compute compact data structures used in combination scoring.
    let mut has_sets = vec![(0u8, 0u8); NUM_FRAG_ID];
    for frag in &fragments {
        let set1 = get_set_effect_id(&frag.set_effect_1)?;
        let set2 = get_set_effect_id(&frag.set_effect_2)?;
        has_sets[get_fragment_id(&frag.name)?] = (set1 as u8, set2 as u8);
    }

    let mut compact_frag_weights = vec![0; NUM_FRAG_ID];
    for (frag, weight) in &frag_weights {
        compact_frag_weights[get_fragment_id(frag)?] = *weight;
    }

    let mut compact_set_effect_weights = vec![vec![]; NUM_SET_EFFECT_ID];
    for (set, threshold, weight) in &set_effect_weights {
        compact_set_effect_weights[get_set_effect_id(set)?].push((*threshold, *weight));
    }

    // Compute which fragments must always be included.
    let mandatory_fragments: Vec<u8> = frag_weights
        .iter()
        .filter(|(_frag, w)| **w >= MANDATORY_THRESHOLD)
        .map(|(frag, _w)| Ok(get_fragment_id(frag)? as u8))
        .try_collect()?;

    // Ignore any irrelevant fragments that can't contribute anything positive or that are forbidden.
    let filtered_frag_ids: Vec<u8> = fragments
        .iter()
        .filter(|frag| {
            let is_already_mandatory = frag_weights.get(&frag.name).map(|w| *w >= MANDATORY_THRESHOLD).unwrap_or(false);
            let can_individually_contribute = frag_weights.get(&frag.name).map(|w| *w > 0).unwrap_or(false);
            let can_contribute_to_set = set_effect_weights
                .iter()
                .any(|(set, _, w)| *w > 0 && (&frag.set_effect_1 == set || &frag.set_effect_2 == set));
            let can_contribute = can_individually_contribute || can_contribute_to_set;
            let forbidden = frag_weights.get(&frag.name).map(|w| *w <= FORBIDDEN_THRESHOLD).unwrap_or(false);
            !is_already_mandatory && !forbidden && can_contribute
        })
        .map(|frag| Ok(get_fragment_id(&frag.name)? as u8))
        .try_collect()?;
    
    let num_optional_frags = max_fragments as i64 - mandatory_fragments.len() as i64;
    let mut best_combinations: Vec<Combination> = (0..=num_optional_frags)
        .flat_map(|k| filtered_frag_ids.iter().copied().combinations(k as usize))
        .map(|mut comb| {
            comb.extend(&mandatory_fragments);
            let (score, set_ctr) = get_score(
                &comb,
                &has_sets,
                &compact_frag_weights,
                &compact_set_effect_weights,
            );
            (-score, comb.len(), comb, set_ctr)
        })
        .k_smallest(max_results)
        .into_iter()
        .sorted()
        .map(|(negscore, _len, combo, set_ctr)| {
            let used_fragments = combo
                .into_iter()
                .map(|frag_idx| fragments[frag_idx as usize].name.clone())
                .sorted()
                .collect();

            let activated_set_effects = set_ctr
                .into_iter()
                .enumerate()
                .filter(|(set_idx, ctr)| *ctr >= set_effects[*set_idx].minimum)
                .map(|(set_idx, ctr)| {
                    (
                        set_effects[set_idx].name.clone(),
                        ctr.min(set_effects[set_idx].maximum),
                    )
                })
                .sorted()
                .collect();

            Combination {
                score: -negscore,
                used_fragments,
                activated_set_effects,
                alternatives: None
            }
        })
        .collect();

    // Try to merge results.
    let mut i = 0;
    while i < best_combinations.len() {
        let mut merged = false;
        let (before, after) = best_combinations.split_at_mut(i);
        let bci = &after[0];
        let bci_frags: HashSet<String> = HashSet::from_iter(bci.used_fragments.iter().cloned());

        for absorber in before.iter_mut() {
            let same_score = bci.score == absorber.score;
            let same_num_frags = bci.used_fragments.len() == absorber.used_fragments.len();
            if !(same_score && same_num_frags) {
                continue;
            }

            let same_set_effects = bci.activated_set_effects == absorber.activated_set_effects;
            if !same_set_effects {
                continue;
            }

            let absorber_frags: HashSet<String> = HashSet::from_iter(absorber.used_fragments.iter().cloned());
            let common = &bci_frags & &absorber_frags; 
            if common.len() == bci.used_fragments.len() - 1 {
                let cur = (&absorber_frags - &common).into_iter().next().unwrap();
                let alt = (&bci_frags - &common).into_iter().next().unwrap();
                if frag_weights.contains_key(&cur) || frag_weights.contains_key(&alt) {
                    continue;
                }
                match &mut absorber.alternatives {
                    out @ None => {
                        *out = Some((cur, vec![alt]));
                        merged = true;
                        break;
                    },
                    Some((c, v)) if c == &cur => {
                        v.push(alt);
                        v.sort();
                        merged = true;
                        break;
                    },

                    Some(_) => {}
                }
            }
        }

        if merged {
            best_combinations.remove(i);
        } else {
            i += 1;
        }
    }

    Ok(best_combinations)
}
