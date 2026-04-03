use crate::counter::play_objective;
use crate::input::{read_choice_1_or_2, wait_for_enter};
use crate::mechanics::{generate_objectives, Objective};
use crate::player::Player;
use log::{debug, info, trace, warn};
use std::time::{SystemTime, UNIX_EPOCH};

/// Gestion des manches et du déroulement du jeu.
///
/// Ce module orchestre :
///
/// - Le déroulement d'un tour
/// - La succession des manches
/// - Les effets (comme le poison)
///
/// Joue tous les objectifs d'un joueur et retourne son score moyen (ceiling).
///
/// Affiche les stats du joueur, la liste des objectifs, puis lance [`play_objective`]
/// pour chaque objectif. Le score final est la moyenne arrondie à l'entier supérieur.
///
/// # Erreurs
///
/// Propage les erreurs stdin de [`play_objective`] et [`wait_for_enter`].
pub fn play_turn(player: &Player, objectives: &[Objective]) -> Result<i32, String> {
    trace!("Début du tour pour joueur: {}", player.name);
    player.display_stats();

    let formatted_objectives = objectives
        .iter()
        .map(|obj| match obj {
            Objective::Number(n) => format!("{}", n),
            Objective::Letter(c, t) => format!("({}:{})", c, t),
        })
        .collect::<Vec<_>>()
        .join(", ");

    debug!(
        "Objectifs générés pour {} : [{}]",
        player.name, formatted_objectives
    );
    println!("→ Objectifs : [{}]", formatted_objectives);
    println!("→ Appuyer sur la touche demandée (ENTER pour mode numérique, lettre directe pour mode lettre) pour démarrer le tour..");
    wait_for_enter()?;

    let mut total_score = 0i32;
    for &obj in objectives {
        total_score += play_objective(player, obj)?;
    }

    let avg = compute_turn_average(total_score, objectives.len());

    println!("# Fin du tour #");
    println!("→ Score moyen : {}", avg);
    debug!(
        "Détail du calcul - Total: {}, Count: {}, Moyenne: {}",
        total_score,
        objectives.len(),
        avg
    );
    info!(
        "{} termine son tour avec un score moyen de {} (total={}, count={})",
        player.name,
        avg,
        total_score,
        objectives.len()
    );

    Ok(avg)
}

/// Calcule la moyenne d'un tour (ceiling), extraite pour être testable indépendamment.
///
/// Retourne `0` si `count` vaut 0 afin d'éviter une division par zéro.
pub fn compute_turn_average(total: i32, count: usize) -> i32 {
    if count == 0 {
        return 0;
    }
    f64::ceil(total as f64 / count as f64) as i32
}

/// Applique un poison au `loser` selon le `choice` déjà lu (1 ou 2).
///
/// Séparée de [`apply_poison`] pour être testable sans dépendance à stdin.
///
/// | Choice | Effet                                        | Plancher |
/// |--------|----------------------------------------------|----------|
/// | 1      | −5 Speed  (compteur plus rapide → difficile) | 5 ms     |
/// | 2      | −5 Strength (moins de points par objectif)   | 0        |
pub fn apply_poison_choice(loser: &mut Player, choice: u8) {
    match choice {
        1 => {
            let old_speed = loser.speed;
            loser.speed = loser.speed.saturating_sub(5).max(5);
            println!(
                "☠ {} perd 5 de Speed → Speed = {} ms/incrément",
                loser.name, loser.speed
            );
            warn!(
                "Poison Speed appliqué à {} : {} → {}",
                loser.name, old_speed, loser.speed
            );
        }
        2 => {
            let old_strength = loser.strength;
            loser.strength = (loser.strength - 5).max(0);
            println!(
                "☠ {} perd 5 de Strength → Strength = {}",
                loser.name, loser.strength
            );
            warn!(
                "Poison Strength appliqué à {} : {} → {}",
                loser.name, old_strength, loser.strength
            );
        }
        _ => {
            // Ne devrait pas arriver — read_choice_1_or_2 garantit 1 ou 2.
            println!("⚠ Choix inconnu, aucun poison appliqué.");
        }
    }
}

/// Le gagnant d'une manche choisit un malus à infliger au perdant.
///
/// Affiche les options, lit le choix sur stdin, puis délègue à [`apply_poison_choice`].
///
/// # Erreurs
///
/// Retourne `Err` si la lecture stdin échoue.
pub fn apply_poison(winner: &Player, loser: &mut Player) -> Result<(), String> {
    println!(
        "\n{} doit choisir le poison à appliquer à {} :",
        winner.name, loser.name
    );
    println!("  → 1: -5 Speed  (compteur plus rapide)");
    println!("  → 2: -5 Strength (moins de points par objectif)");

    let choice = read_choice_1_or_2()?;
    apply_poison_choice(loser, choice);
    Ok(())
}

/// Joue une manche complète (tour de chaque joueur, résolution, poison).
///
/// Retourne `Ok((true, vec![]))` si la partie continue,
/// `Ok((false, newly_eliminated))` si la partie se termine (avec les indices des joueurs nouvellement éliminés).
///
/// # Erreurs
///
/// Propage les erreurs stdin des tours et du choix de poison.
pub fn play_round(
    round: usize,
    players: &mut [Player],
    nb_objectives: usize,
    is_number_mode: bool,
) -> Result<(bool, Vec<usize>), String> {
    info!(
        "=== Manche {} === Mode: {}",
        round,
        if is_number_mode { "ENTER" } else { "LETTRE" }
    );
    println!("\n╔══════════════════════════╗");
    println!("║       ## Manche {} ##      ║", round);
    println!("╚══════════════════════════╝\n");

    let mut scores = Vec::new();
    let mut active_indices = Vec::new();
    for (i, player) in players.iter_mut().enumerate() {
        if player.vitality > 0 {
            let score = play_turn(player, &generate_objectives(nb_objectives, !is_number_mode))?;
            scores.push(score);
            active_indices.push(i);
            println!();
        }
    }

    // Résolution : le joueur avec le score le plus bas perd, le meilleur score choisit le poison
    let mut newly_eliminated = Vec::new();

    let mut score_pairs: Vec<(usize, i32)> = active_indices
        .iter()
        .enumerate()
        .map(|(position, &idx)| (idx, scores[position]))
        .collect();

    debug!(
        "Scores bruts pour la manche {} : {:?}",
        round,
        score_pairs
            .iter()
            .map(|(idx, score)| (players[*idx].name.clone(), score))
            .collect::<Vec<_>>()
    );

    // Tri décroissant pour accéder facilement au top/bottom
    score_pairs.sort_by_key(|&(_, score)| -score);

    let top_score = score_pairs[0].1;
    let bottom_score = score_pairs.last().unwrap().1;
    let all_equal = top_score == bottom_score;

    let top_tie_count = score_pairs.iter().filter(|&&(_, score)| score == top_score).count();
    let bottom_tie_count = score_pairs
        .iter()
        .rev()
        .filter(|&&(_, score)| score == bottom_score)
        .count();

    if all_equal {
        warn!("Égalité parfaite tous joueurs confondus - aucun malus/bonus");
        println!("⚖  Égalité parfaite entre tous les joueurs ! Aucun bonus/malus appliqué.");
    } else if top_tie_count > 1 {
        // Deux (ou plus) joueurs à égalité avec le meilleur score
        let winner_indices: Vec<usize> = score_pairs
            .iter()
            .filter(|&&(_, score)| score == top_score)
            .map(|&(idx, _)| idx)
            .collect();
        let loser_idx = score_pairs.last().unwrap().0;

        let diff = top_score - bottom_score;
        players[loser_idx].vitality -= diff;
        println!(
            "🏆 Égalité en tête — {} perd {} points de vitalité.",
            players[loser_idx].name,
            diff
        );
        if players[loser_idx].vitality <= 0 {
            newly_eliminated.push(loser_idx);
            warn!(
                "{} est éliminé par l'égalité en tête - vitalité finale = {}",
                players[loser_idx].name, players[loser_idx].vitality
            );
        }

        println!(
            "⚖  Égalité sur le meilleur score entre {} et {}. Discussion du malus...",
            winner_indices
                .iter()
                .map(|&i| players[i].name.clone())
                .collect::<Vec<_>>()
                .join(" et "),
            players[loser_idx].name
        );

        let mut choices = Vec::new();
        for &idx in &winner_indices {
            println!("{} : choisissez le malus pour {} (1=Speed, 2=Strength)", players[idx].name, players[loser_idx].name);
            let choice = read_choice_1_or_2()?;
            choices.push(choice);
        }

        let poison_choice = if choices.iter().all(|&x| x == choices[0]) {
            choices[0]
        } else {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos();
            let random_choice = if nanos % 2 == 0 { 1 } else { 2 };
            println!("⚖  Divergence d'avis détectée, malus aléatoire appliqué : {}", random_choice);
            random_choice
        };

        apply_poison_choice(&mut players[loser_idx], poison_choice);
        if players[loser_idx].vitality <= 0 {
            newly_eliminated.push(loser_idx);
            warn!("{} est éliminé - vitalité finale = {}", players[loser_idx].name, players[loser_idx].vitality);
        }
    } else if bottom_tie_count > 1 {
        // Deux (ou plus) joueurs à égalité avec le plus mauvais score
        let winner_idx = score_pairs[0].0;
        let loser_indices: Vec<usize> = score_pairs
            .iter()
            .rev()
            .take(bottom_tie_count)
            .map(|&(idx, _)| idx)
            .collect();

        println!(
            "⚖  Égalité sur le pire score entre {}. {} décide du malus pour les perdants.",
            loser_indices
                .iter()
                .map(|&i| players[i].name.clone())
                .collect::<Vec<_>>()
                .join(" et "),
            players[winner_idx].name
        );

        println!("{} : choisissez le malus (1=Speed, 2=Strength)", players[winner_idx].name);
        let poison_choice = read_choice_1_or_2()?;

        let diff = top_score - bottom_score;
        for &loser_idx in &loser_indices {
            players[loser_idx].vitality -= diff;
            println!("💔 {} perd {} points de vitalité.", players[loser_idx].name, diff);
            if players[loser_idx].vitality <= 0 {
                newly_eliminated.push(loser_idx);
                warn!(
                    "{} est éliminé par l'égalité du bas - vitalité finale = {}",
                    players[loser_idx].name,
                    players[loser_idx].vitality
                );
            }
            apply_poison_choice(&mut players[loser_idx], poison_choice);
            if players[loser_idx].vitality <= 0 {
                // already ajouté si éliminé ci-dessus, mais on re-vérifie pour information
                if !newly_eliminated.contains(&loser_idx) {
                    newly_eliminated.push(loser_idx);
                }
            }
        }
    } else {
        let winner_idx = score_pairs[0].0;
        let loser_idx = score_pairs.last().unwrap().0;
        let diff = score_pairs[0].1 - score_pairs.last().unwrap().1;

        println!(
            "🏆 {} gagne la manche avec le meilleur score ! {} perd {} points de vitalité.",
            players[winner_idx].name, players[loser_idx].name, diff
        );
        info!(
            "Manche {} - Gagnant: {} ({}), Perdant: {} ({}) - Diff vitality: {}",
            round,
            players[winner_idx].name,
            score_pairs[0].1,
            players[loser_idx].name,
            score_pairs.last().unwrap().1,
            diff
        );
        players[loser_idx].vitality -= diff;
        if players[loser_idx].vitality <= 0 {
            newly_eliminated.push(loser_idx);
            warn!(
                "{} est éliminé - vitalité finale = {}",
                players[loser_idx].name, players[loser_idx].vitality
            );
        }
        let winner = players[winner_idx].clone();
        apply_poison(&winner, &mut players[loser_idx])?;
    }

    println!("\n## FIN Manche {} ##", round);
    print!("Vitalités — ");
    for (i, player) in players.iter().enumerate() {
        if i > 0 {
            print!(" | ");
        }
        print!("{} : {} pv", player.name, player.vitality);
    }
    println!();

    // Vérification fin de partie : seulement quand il reste 1 joueur ou moins
    let alive_count = players.iter().filter(|p| p.vitality > 0).count();
    if alive_count <= 1 {
        if let Some(winner) = players.iter().find(|p| p.vitality > 0) {
            println!("\n🎉 {} gagne la partie !", winner.name);
        } else {
            println!("\n🤝 Égalité parfaite ! Tous les joueurs sont éliminés.");
        }

        return Ok((false, newly_eliminated));
    }

    Ok((true, newly_eliminated))
}

// ————————————————————————————————————————————————————————
// Tests
// ————————————————————————————————————————————————————————

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;

    fn make_player(name: &str) -> Player {
        Player::new(name.to_string(), 50, 50, 50)
    }

    // --- compute_turn_average ---

    /// Vérifie compute_turn_average pour une moyenne exacte.
    #[test]
    fn verifie_calculer_moyenne_tour_exact() {
        assert_eq!(compute_turn_average(100, 2), 50);
    }

    /// Vérifie compute_turn_average avec ceiling.
    #[test]
    fn verifie_calculer_moyenne_tour_ceiling() {
        // 85 / 3 = 28.33… → 29 après ceiling
        assert_eq!(compute_turn_average(85, 3), 29);
    }

    /// Vérifie compute_turn_average avec l'exemple du cahier des charges.
    #[test]
    fn verifie_calculer_moyenne_tour_spec_example() {
        // Exemple du cahier des charges : [45, 130, 130, 55, 65] → total=425, avg=85
        assert_eq!(compute_turn_average(425, 5), 85);
    }

    /// Vérifie compute_turn_average avec un nombre d'objectifs nul.
    #[test]
    fn verifie_calculer_moyenne_tour_zero_objectif() {
        // Division par zéro évitée
        assert_eq!(compute_turn_average(100, 0), 0);
    }

    // --- apply_poison_choice ---

    /// Vérifie apply_poison_choice pour la vitesse.
    #[test]
    fn verifie_appliquer_poison_speed() {
        let mut loser = make_player("Loser");
        loser.speed = 50;
        apply_poison_choice(&mut loser, 1);
        assert_eq!(loser.speed, 45);
    }

    /// Vérifie apply_poison_choice pour la vitesse au plancher.
    #[test]
    fn verifie_appliquer_poison_speed_floor() {
        let mut loser = make_player("Loser");
        loser.speed = 7; // 7 - 5 = 2, mais plancher = 5
        apply_poison_choice(&mut loser, 1);
        assert_eq!(loser.speed, 5);
    }

    /// Vérifie apply_poison_choice quand la vitesse est déjà au plancher.
    #[test]
    fn verifie_appliquer_poison_speed_already_at_floor() {
        let mut loser = make_player("Loser");
        loser.speed = 5; // déjà au plancher
        apply_poison_choice(&mut loser, 1);
        assert_eq!(loser.speed, 5);
    }

    /// Vérifie apply_poison_choice pour la force.
    #[test]
    fn verifie_appliquer_poison_strength() {
        let mut loser = make_player("Loser");
        loser.strength = 50;
        apply_poison_choice(&mut loser, 2);
        assert_eq!(loser.strength, 45);
    }

    /// Vérifie apply_poison_choice pour la force au plancher.
    #[test]
    fn verifie_appliquer_poison_strength_floor() {
        let mut loser = make_player("Loser");
        loser.strength = 3; // 3 - 5 < 0 → plancher = 0
        apply_poison_choice(&mut loser, 2);
        assert_eq!(loser.strength, 0);
    }

    /// Vérifie apply_poison_choice quand la force est déjà à zéro.
    #[test]
    fn verifie_appliquer_poison_strength_already_zero() {
        let mut loser = make_player("Loser");
        loser.strength = 0;
        apply_poison_choice(&mut loser, 2);
        assert_eq!(loser.strength, 0);
    }

    /// Un poison invalide ne doit pas modifier le joueur.
    #[test]
    fn verifie_appliquer_poison_invalid_choice_no_change() {
        let mut loser = make_player("Loser");
        let speed_before = loser.speed;
        let strength_before = loser.strength;
        apply_poison_choice(&mut loser, 9); // choix invalide
        assert_eq!(loser.speed, speed_before);
        assert_eq!(loser.strength, strength_before);
    }
}
