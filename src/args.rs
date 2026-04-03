use clap::Parser;

/// Gestion des arguments en ligne de commande.
///
/// Ce module utilise la crate [`clap`] pour parser les paramètres
/// fournis au lancement du programme.
///
/// ## Paramètres supportés
///
/// - Nom des joueurs
/// - Vitalité initiale
/// - Nombre d'objectifs par manche
/// - Statistiques (vitesse, force)
///
/// Arguments de ligne de commande pour configurer la partie.
///
/// Tous les paramètres ont des valeurs par défaut et peuvent être omis.
///
/// # Exemple de lancement
///
/// ```text
/// cargo run -- --name1 Amaury --name2 Julien --name3 Player3 --name4 Player4 --vitality 50 --objectifs 5
/// ```
///
/// # Valeurs par défaut
///
/// | Paramètre   | Défaut  | Description                                    |
/// |-------------|---------|------------------------------------------------|
/// | `name1`     | Amaury  | Nom du joueur 1                                |
/// | `name2`     | Julien  | Nom du joueur 2                                |
/// | `name3`     | Player3 | Nom du joueur 3                                |
/// | `name4`     | Player4 | Nom du joueur 4                                |
/// | `vitality`  | 50      | Points de vie initiaux                         |
/// | `objectifs` | 5       | Nombre d'objectifs par tour                    |
/// | `speed`     | 50      | ms entre chaque tick du compteur (min : 5)     |
/// | `strength`  | 50      | Bonus de force ajouté au score brut            |
#[derive(Parser, Debug)]
#[command(
    name = "duel_game",
    about = "Mini-jeu de duel au tour par tour en Rust",
    version
)]
pub struct Args {
    /// Nom du joueur 1
    #[arg(long, default_value = "Amaury")]
    pub name1: String,

    /// Nom du joueur 2
    #[arg(long, default_value = "Julien")]
    pub name2: String,

    /// Nom du joueur 3
    #[arg(long)]
    pub name3: Option<String>,

    /// Nom du joueur 4
    #[arg(long)]
    pub name4: Option<String>,

    /// Vitalité initiale commune aux deux joueurs
    #[arg(long, default_value_t = 50)]
    pub vitality: i32,

    /// Nombre d'objectifs par tour de jeu
    #[arg(long, default_value_t = 5)]
    pub objectifs: usize,

    /// Vitesse initiale en ms par incrément du compteur (plus bas = plus rapide).
    ///
    /// Valeur minimale : 5 ms (cohérent avec le plancher appliqué par les poisons).
    /// Passer une valeur inférieure à 5 est rejeté au démarrage.
    #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u64).range(5..))]
    pub speed: u64,

    /// Force initiale des joueurs (bonus ajouté au score brut de chaque objectif)
    #[arg(long, default_value_t = 50)]
    pub strength: i32,
}

// ————————————————————————————————————————————————————————
// Tests
// ————————————————————————————————————————————————————————

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Vérifie que toutes les valeurs par défaut correspondent au cahier des charges.
    #[test]
    fn verifie_arguments_par_defaut() {
        let args = Args::try_parse_from(["duel_game"]).unwrap();
        assert_eq!(args.name1, "Amaury");
        assert_eq!(args.name2, "Julien");
        assert!(args.name3.is_none());
        assert!(args.name4.is_none());
        assert_eq!(args.vitality, 50);
        assert_eq!(args.objectifs, 5);
        assert_eq!(args.speed, 50);
        assert_eq!(args.strength, 50);
    }

    /// Vérifie que des paramètres custom remplacent bien les valeurs par défaut.
    #[test]
    fn verifie_arguments_noms_et_vitalite_personnalises() {
        let args = Args::try_parse_from([
            "duel_game",
            "--name1",
            "Amaury",
            "--name2",
            "Julien",
            "--name3",
            "Player3",
            "--name4",
            "Player4",
            "--vitality",
            "100",
        ])
        .unwrap();
        assert_eq!(args.name1, "Amaury");
        assert_eq!(args.name2, "Julien");
        assert_eq!(args.name3, Some("Player3".to_string()));
        assert_eq!(args.name4, Some("Player4".to_string()));
        assert_eq!(args.vitality, 100);
    }

    /// Vérifie que --speed avec une valeur valide (≥ 5) est accepté.
    #[test]
    fn verifie_arguments_vitesse_valide() {
        let args = Args::try_parse_from(["duel_game", "--speed", "5"]).unwrap();
        assert_eq!(args.speed, 5);

        let args2 = Args::try_parse_from(["duel_game", "--speed", "200"]).unwrap();
        assert_eq!(args2.speed, 200);
    }

    /// Vérifie que --speed < 5 est rejeté par le validateur clap (cohérence avec le plancher des poisons).
    #[test]
    fn verifie_arguments_vitesse_trop_basse_rejetee() {
        let result = Args::try_parse_from(["duel_game", "--speed", "4"]);
        assert!(
            result.is_err(),
            "speed=4 devrait être refusé (plancher = 5)"
        );

        let result2 = Args::try_parse_from(["duel_game", "--speed", "0"]);
        assert!(result2.is_err(), "speed=0 devrait être refusé");
    }

    /// Vérifie le nombre d'objectifs personnalisé.
    #[test]
    fn verifie_arguments_objectifs_personnalises() {
        let args = Args::try_parse_from(["duel_game", "--objectifs", "3"]).unwrap();
        assert_eq!(args.objectifs, 3);
    }
}
