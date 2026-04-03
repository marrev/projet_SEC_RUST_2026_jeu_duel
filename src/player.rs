use log::info;

/// Gestion des joueurs : structure et statistiques.
///
/// Ce module définit la structure [`Player`] et les méthodes associées,
/// notamment la création et la gestion de l'état d'un joueur.
///
/// Un joueur avec ses caractéristiques, susceptibles d'évoluer sous l'effet des poisons.
///
/// # Cycle de vie
///
/// - Créé en début de partie via [`Player::new`] avec les paramètres CLI.
/// - Ses stats (`speed`, `strength`) sont réduites par les poisons entre chaque manche.
/// - La partie s'arrête dès que `vitality ≤ 0`.
///
/// # Exemple
///
/// ```text
/// Au tour de Amaury (Vitality=50, Speed=50, Strength=50)
/// ```
#[derive(Debug, Clone)]
pub struct Player {
    /// Nom affiché dans les messages de jeu.
    pub name: String,
    /// Points de vie — la partie s'arrête quand `vitality <= 0`.
    pub vitality: i32,
    /// Délai en ms entre chaque tick du compteur. Plus bas = compteur plus rapide = plus difficile.
    /// Plancher appliqué à 5 ms par les poisons et à l'entrée CLI.
    pub speed: u64,
    /// Bonus de force ajouté au score brut à chaque objectif. Plancher à 0.
    pub strength: i32,
}

impl Player {
    /// Crée un nouveau joueur avec les caractéristiques données.
    ///
    /// # Arguments
    ///
    /// * `name`     – Nom du joueur (affiché en jeu)
    /// * `vitality` – Points de vie initiaux
    /// * `speed`    – Délai en ms entre chaque tick du compteur (≥ 5 recommandé)
    /// * `strength` – Bonus de force initial (≥ 0 recommandé)
    pub fn new(name: String, vitality: i32, speed: u64, strength: i32) -> Self {
        Player {
            name,
            vitality,
            speed,
            strength,
        }
    }

    /// Affiche les caractéristiques actuelles du joueur dans le terminal et les logue.
    ///
    /// Appelée au début de chaque tour pour informer les joueurs de l'état courant.
    ///
    /// # Exemple de sortie
    ///
    /// ```text
    /// Au tour de Amaury (Vitality=50, Speed=50, Strength=50)
    /// ```
    pub fn display_stats(&self) {
        println!(
            "Au tour de {} (Vitality={}, Speed={}, Strength={})",
            self.name, self.vitality, self.speed, self.strength
        );
        info!(
            "Stats de {} : vitality={} speed={} strength={}",
            self.name, self.vitality, self.speed, self.strength
        );
    }
}

// ————————————————————————————————————————————————————————
// Tests
// ————————————————————————————————————————————————————————

#[cfg(test)]
mod tests {
    use super::*;

    /// Vérifie que tous les champs sont bien initialisés par [`Player::new`].
    #[test]
    fn verifie_joueur_new_fields() {
        let p = Player::new("Alice".to_string(), 100, 40, 60);
        assert_eq!(p.name, "Alice");
        assert_eq!(p.vitality, 100);
        assert_eq!(p.speed, 40);
        assert_eq!(p.strength, 60);
    }

    /// Clone : les deux instances doivent être indépendantes.
    #[test]
    fn verifie_joueur_clone_independence() {
        let p1 = Player::new("Bob".to_string(), 50, 50, 50);
        let mut p2 = p1.clone();
        p2.vitality -= 10;
        assert_eq!(p1.vitality, 50);
        assert_eq!(p2.vitality, 40);
    }

    /// Vérifie que la vitalité peut descendre en négatif (fin de partie détectée par play_round).
    #[test]
    fn verifie_joueur_vitality_can_go_negative() {
        let mut p = Player::new("Test".to_string(), 5, 50, 50);
        p.vitality -= 10;
        assert_eq!(p.vitality, -5);
    }

    /// Vérifie que le champ name est bien un String owned (pas une référence).
    #[test]
    fn verifie_joueur_name_owned() {
        let name = String::from("Charlie");
        let p = Player::new(name.clone(), 50, 50, 50);
        assert_eq!(p.name, name);
    }

    /// Vérifie que display_stats ne panique pas, quelles que soient les valeurs du joueur.
    #[test]
    fn verifie_afficher_stats_ne_panique_pas() {
        let p = Player::new("Testeur".to_string(), 50, 50, 50);
        p.display_stats();
    }

    /// Vérifie que display_stats ne panique pas avec des valeurs extrêmes (vitalité négative, force nulle).
    #[test]
    fn verifie_afficher_stats_extreme_values_ne_panique_pas() {
        let p = Player::new("Extrême".to_string(), -999, 5, 0);
        p.display_stats();
    }
}
