use rand::RngExt;

/// Logique de jeu pure.
///
/// Ce module contient les fonctions liées aux calculs du jeu :
///
/// - Calcul des scores
/// - Différences circulaires
/// - Génération d'objectifs
///
/// Ces fonctions sont indépendantes des entrées/sorties.
/// Elles sont utilisées à la fois dans la logique de jeu (cf. [`crate::round`]) et dans les tests unitaires.
///
/// Retourne le score de base associé à une différence circulaire.
///
/// Utilisé à la fois dans [`calculate_score`] et dans l'affichage de [`crate::counter::play_objective`],
/// ce qui évite de dupliquer la table de correspondance diff → base.
///
/// # Table de correspondance
///
/// | Diff (absolue) | Score de base |
/// |----------------|---------------|
/// | 0              | 100           |
/// | 1 – 5          | 80            |
/// | 6 – 10         | 60            |
/// | 11 – 20        | 40            |
/// | 21 – 40        | 20            |
/// | > 40           | 0             |
pub fn score_base(diff: i32) -> i32 {
    match diff {
        0 => 100,
        1..=5 => 80,
        6..=10 => 60,
        11..=20 => 40,
        21..=40 => 20,
        _ => 0,
    }
}

/// Différence circulaire entre compteur et objectif (le compteur boucle sur 0–100).
///
/// Prend le minimum entre la distance directe et la distance en passant par 0.
///
/// # Exemple
///
/// ```text
/// counter=95, objective=15 → diff=20 (via 0), pas 80 (direct).
/// ```
pub fn circular_diff(counter: i32, objective: i32) -> i32 {
    let direct = (counter - objective).abs();
    let circular = 100 - direct;
    direct.min(circular)
}

/// Calcule le score pour un objectif en fonction de la précision, de la force et des ratés.
///
/// Formule : `(score_base(diff) + strength) / (miss + 1)`
///
/// Le score est divisé par `miss + 1` pour pénaliser les joueurs qui laissent
/// le compteur boucler plusieurs fois.
///
/// # Arguments
///
/// * `diff`     – Différence circulaire entre compteur et objectif (cf. [`circular_diff`])
/// * `strength` – Bonus de force du joueur (peut être 0 si le poison l'a réduit)
/// * `miss`     – Nombre de fois où le compteur a bouclé sur 0
pub fn calculate_score(diff: i32, strength: i32, miss: i32) -> i32 {
    (score_base(diff) + strength) / (miss + 1)
}

/// Objectif pour un tour : nombre ou lettre.
#[derive(Debug, Clone, Copy)]
pub enum Objective {
    Number(i32),
    Letter(char, i32),
}

/// Génère `count` objectifs selon le mode choisi.
///
/// Si `is_letter_mode`, génère des lettres minuscules aléatoires.
/// Sinon, génère des nombres dans [0, 100].
pub fn generate_objectives(count: usize, is_letter_mode: bool) -> Vec<Objective> {
    let mut rng = rand::rng();
    if is_letter_mode {
        (0..count)
            .map(|_| {
                Objective::Letter(
                    rng.random_range(b'a'..=b'z') as char,
                    rng.random_range(0..=99),
                )
            })
            .collect()
    } else {
        (0..count)
            .map(|_| Objective::Number(rng.random_range(0..=100)))
            .collect()
    }
}

// ————————————————————————————————————————————————————————
// Tests
// ————————————————————————————————————————————————————————

#[cfg(test)]
mod tests {
    use super::*;

    // --- score_base ---

    /// Vérifie que score_base retourne 100 pour une différence de 0.
    #[test]
    fn verifie_base_score_exact() {
        assert_eq!(score_base(0), 100);
    }

    /// Vérifie les limites des paliers de score_base.
    #[test]
    fn verifie_base_score_boundaries() {
        assert_eq!(score_base(1), 80);
        assert_eq!(score_base(5), 80);
        assert_eq!(score_base(6), 60);
        assert_eq!(score_base(10), 60);
        assert_eq!(score_base(11), 40);
        assert_eq!(score_base(20), 40);
        assert_eq!(score_base(21), 20);
        assert_eq!(score_base(40), 20);
        assert_eq!(score_base(41), 0);
    }

    /// Vérifie score_base pour une grande différence.
    #[test]
    fn verifie_base_score_large_diff() {
        assert_eq!(score_base(100), 0);
    }

    // --- circular_diff ---

    /// Vérifie circular_diff pour des différences directes.
    #[test]
    fn verifie_difference_circulaire_direct() {
        assert_eq!(circular_diff(80, 82), 2);
        assert_eq!(circular_diff(30, 36), 6);
    }

    /// Vérifie circular_diff avec boucleping autour de 100.
    #[test]
    fn verifie_difference_circulaire_boucle() {
        assert_eq!(circular_diff(95, 15), 20);
        assert_eq!(circular_diff(5, 98), 7);
    }

    /// Vérifie circular_diff quand les valeurs sont identiques.
    #[test]
    fn verifie_difference_circulaire_zero() {
        assert_eq!(circular_diff(50, 50), 0);
        assert_eq!(circular_diff(0, 0), 0);
        assert_eq!(circular_diff(100, 100), 0);
    }

    /// Vérifie circular_diff au milieu du cercle.
    #[test]
    fn verifie_difference_circulaire_milieu() {
        assert_eq!(circular_diff(0, 50), 50);
    }

    // --- calculate_score ---

    /// Vérifie calculate_score sans miss et différence nulle.
    #[test]
    fn verifie_calculer_score_exact_no_miss() {
        assert_eq!(calculate_score(0, 50, 0), 150);
    }

    /// Vérifie calculate_score avec petite différence et un miss.
    #[test]
    fn verifie_calculer_score_small_diff_with_miss() {
        assert_eq!(calculate_score(3, 50, 1), 65);
    }

    /// Vérifie calculate_score avec différence moyenne.
    #[test]
    fn verifie_calculer_score_medium_diff() {
        assert_eq!(calculate_score(8, 50, 0), 110);
    }

    /// Vérifie calculate_score avec grande différence.
    #[test]
    fn verifie_calculer_score_large_diff() {
        assert_eq!(calculate_score(15, 50, 0), 90);
    }

    /// Vérifie calculate_score avec très grande différence.
    #[test]
    fn verifie_calculer_score_very_large_diff() {
        assert_eq!(calculate_score(30, 50, 0), 70);
    }

    /// Vérifie calculate_score quand l'objectif est complètement raté.
    #[test]
    fn verifie_calculer_score_missed_entirely() {
        assert_eq!(calculate_score(50, 50, 0), 50);
    }

    /// Vérifie calculate_score avec force nulle.
    #[test]
    fn verifie_calculer_score_zero_strength() {
        assert_eq!(calculate_score(0, 0, 0), 100);
        assert_eq!(calculate_score(50, 0, 0), 0);
    }

    /// Vérifie que calculate_score délègue bien à score_base (cohérence DRY).
    #[test]
    fn verifie_calculer_score_uses_score_base() {
        for diff in [0, 3, 8, 15, 30, 50] {
            let expected = (score_base(diff) + 10) / 1;
            assert_eq!(calculate_score(diff, 10, 0), expected);
        }
    }

    // --- generate_objectives ---

    /// Vérifie que generate_objectives retourne le bon nombre d'objectifs.
    #[test]
    fn verifie_generer_objectifs_count() {
        for n in [1, 3, 5, 10] {
            assert_eq!(generate_objectives(n, true).len(), n);
            assert_eq!(generate_objectives(n, false).len(), n);
        }
    }

    /// Vérifie que les objectifs sont des lettres ou nombres selon le mode.
    #[test]
    fn verifie_generer_objectifs_range() {
        let letters = generate_objectives(200, true);
        assert!(letters.iter().all(|&obj| matches!(obj, Objective::Letter(c, n) if c.is_ascii_lowercase() && (0..=99).contains(&n))));

        let numbers = generate_objectives(200, false);
        assert!(numbers
            .iter()
            .all(|&obj| matches!(obj, Objective::Number(n) if (0..=100).contains(&n))));
    }

    /// Score moyen avec ceiling — cas du cahier des charges (Michel manche 1).
    #[test]
    fn verifie_moyenne_ceiling() {
        let scores = vec![45i32, 130, 130, 55, 65];
        let total: i32 = scores.iter().sum();
        let avg = f64::ceil(total as f64 / scores.len() as f64) as i32;
        assert_eq!(avg, 85);
    }

    /// Vérifie le ceiling pour une moyenne non entière.
    #[test]
    fn verifie_moyenne_ceiling_non_integer() {
        let scores = vec![10i32, 10, 11];
        let total: i32 = scores.iter().sum();
        let avg = f64::ceil(total as f64 / scores.len() as f64) as i32;
        assert_eq!(avg, 11);
    }
}
