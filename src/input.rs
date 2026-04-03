use log::warn;
use std::io::{self, Write};

/// Gestion des entrées utilisateur.
///
/// Ce module s'occupe de la lecture des entrées clavier,
/// notamment les choix effectués par les joueurs.
///
/// Bloque jusqu'à ce que l'utilisateur appuie sur ENTRÉE.
///
/// # Erreurs
///
/// Retourne `Err` si la lecture de stdin échoue.
pub fn wait_for_enter() -> Result<(), String> {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("Erreur lecture stdin : {e}"))?;
    Ok(())
}

/// Analyse une chaîne saisie et retourne `1` ou `2`.
///
/// Retourne `1` par défaut si la saisie n'est ni `"1"` ni `"2"`.
/// Cette fonction pure est séparée de [`read_choice_1_or_2`] pour être testable
/// sans dépendance à stdin.
pub fn parse_choice_1_or_2(input: &str) -> u8 {
    match input.trim() {
        "1" => 1,
        "2" => 2,
        other => {
            warn!("Choix invalide '{}', 1 appliqué par défaut.", other);
            1
        }
    }
}

/// Lit un choix 1 ou 2 sur stdin. Retourne 1 par défaut si la saisie est invalide.
///
/// Affiche un prompt `> ` avant la lecture. Délègue l'analyse à [`parse_choice_1_or_2`].
///
/// # Erreurs
///
/// Retourne `Err` si le flush ou la lecture de stdin échoue.
pub fn read_choice_1_or_2() -> Result<u8, String> {
    print!("> ");
    io::stdout()
        .flush()
        .map_err(|e| format!("Erreur flush stdout : {e}"))?;

    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("Erreur lecture stdin : {e}"))?;

    let choice = parse_choice_1_or_2(&line);
    if choice == 1 && line.trim() != "1" {
        println!(" ⚠ Choix invalide, option 1 appliquée par défaut.");
    }
    Ok(choice)
}

// ————————————————————————————————————————————————————————
// Tests
// ————————————————————————————————————————————————————————

#[cfg(test)]
mod tests {
    use super::*;

    /// Choix valides reconnus correctement.
    #[test]
    fn verifie_analyse_choix_valide() {
        assert_eq!(parse_choice_1_or_2("1"), 1);
        assert_eq!(parse_choice_1_or_2("2"), 2);
    }

    /// Les espaces/sauts de ligne autour de la saisie sont ignorés (trim).
    #[test]
    fn verifie_analyse_choix_avec_espaces() {
        assert_eq!(parse_choice_1_or_2("1\n"), 1);
        assert_eq!(parse_choice_1_or_2("  2  "), 2);
    }

    /// Toute saisie autre que "1" ou "2" retourne 1 par défaut.
    #[test]
    fn verifie_analyse_choix_invalide_defaut_a_1() {
        assert_eq!(parse_choice_1_or_2("3"), 1);
        assert_eq!(parse_choice_1_or_2(""), 1);
        assert_eq!(parse_choice_1_or_2("abc"), 1);
        assert_eq!(parse_choice_1_or_2("0"), 1);
    }
}
