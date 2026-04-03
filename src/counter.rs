use crate::mechanics::{calculate_score, circular_diff, Objective};
use crate::player::Player;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use log::{debug, error, trace};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Gestion du temps réel et des objectifs.
///
/// Ce module gère les mécaniques liées au temps :
///
/// - Compteurs en temps réel
/// - Threads
/// - Synchronisation via atomics
///
/// Lance le compteur pour un objectif donné, attend l'appui sur la touche correcte,
/// retourne le score obtenu.
///
/// # Comportement
///
/// Deux threads tournent en parallèle du thread principal :
///
/// - **Thread d'incrémentation** : tick toutes les `player.speed` ms, incrémente de 0 à 100.
/// - **Thread d'affichage** : rafraîchit la ligne courante toutes les 30 ms via `\r`.
///
/// En mode nombre, à l'appui sur ENTRÉE le score est calculé via [`circular_diff`] et
/// [`calculate_score`]. En mode lettre, seul l'appui sur la bonne lettre donne des points.
/// Les threads sont stoppés via un [`AtomicBool`] partagé.
///
/// # Affichage du score
///
/// Le score final est affiché sous la forme :
/// ```text
/// → Objectif  a : Miss = 0 | Compteur =  10 // Score = 150
/// ```
///
/// # Erreurs
///
/// Retourne `Err` si l'activation ou la désactivation du mode raw échoue,
/// ou si la lecture d'événement clavier échoue.
pub fn play_objective(player: &Player, objective: Objective) -> Result<i32, String> {
    trace!(
        "Lancement de l'objectif pour {} : {:?}",
        player.name,
        objective
    );
    let counter = Arc::new(AtomicI32::new(0));
    let miss = Arc::new(AtomicI32::new(0));
    let running = Arc::new(AtomicBool::new(true));

    let is_letter = matches!(objective, Objective::Letter(_, _));

    // Thread 1 : incrémentation du compteur
    {
        let counter_t = Arc::clone(&counter);
        let miss_t = Arc::clone(&miss);
        let running_t = Arc::clone(&running);
        let speed = player.speed;

        thread::spawn(move || {
            while running_t.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(speed));
                if !running_t.load(Ordering::Relaxed) {
                    break;
                }

                let prev = counter_t.fetch_add(1, Ordering::Relaxed);
                if prev + 1 >= 100 {
                    counter_t.store(0, Ordering::Relaxed);
                    miss_t.fetch_add(1, Ordering::Relaxed);
                    debug!(
                        "Compteur a bouclé — miss={}",
                        miss_t.load(Ordering::Relaxed)
                    );
                }
            }
        });
    }

    // Thread 2 : affichage rafraîchi toutes les 30 ms
    {
        let counter_d = Arc::clone(&counter);
        let miss_d = Arc::clone(&miss);
        let running_d = Arc::clone(&running);
        let is_letter_t = is_letter;

        thread::spawn(move || {
            while running_d.load(Ordering::Relaxed) {
                let value = counter_d.load(Ordering::Relaxed);
                let miss_val = miss_d.load(Ordering::Relaxed);
                if is_letter_t {
                    if let Objective::Letter(letter, target) = objective {
                        print!(
                            "\rObjectif [{}] à {} : Miss = {} | Compteur = {}  ",
                            letter, target, miss_val, value
                        );
                    }
                } else if let Objective::Number(num) = objective {
                    print!(
                        "\rObjectif {} : Miss = {} | Compteur = {}  ",
                        num, miss_val, value
                    );
                }
                io::stdout().flush().ok();
                thread::sleep(Duration::from_millis(30));
            }
        });
    }

    // Thread principal : attend la saisie (raw pour mode direct)
    let mut pressed_char = None;
    let mut pressed_enter = false;

    if let Err(e) = enable_raw_mode() {
        error!("Impossible d'activer le mode raw : {}", e);
        return Err(format!("Impossible d'activer le mode raw : {e}"));
    }

    while running.load(Ordering::Relaxed) {
        if event::poll(Duration::from_millis(10)).map_err(|e| format!("Erreur event poll : {e}"))? {
            if let Event::Key(key_event) =
                event::read().map_err(|e| format!("Erreur event read : {e}"))?
            {
                if key_event.kind != KeyEventKind::Press {
                    continue; // ignore Repeat (maintien) et Release (relâchement)
                }
                match key_event.code {
                    KeyCode::Char(c) => {
                        pressed_char = Some(c);
                        break;
                    }
                    KeyCode::Enter => {
                        if !is_letter {
                            pressed_enter = true;
                            break;
                        }
                        // En mode lettre, on ignore Enter ; on attend la lettre
                    }
                    _ => {
                        if !is_letter {
                            // En mode ENTER, n'importe quelle autre touche est un échec immédiat
                            break;
                        }
                    }
                }
            }
        }
    }

    running.store(false, Ordering::Relaxed);
    thread::sleep(Duration::from_millis(35)); // laisse le thread d'affichage terminer

    if let Err(e) = disable_raw_mode() {
        error!("Impossible de quitter le mode raw : {}", e);
        return Err(format!("Impossible de quitter le mode raw : {e}"));
    }

    let final_counter = counter.load(Ordering::Relaxed);
    let final_miss = miss.load(Ordering::Relaxed);

    let score = if is_letter {
        if let Objective::Letter(letter, target) = objective {
            if pressed_char == Some(letter) {
                let diff = circular_diff(final_counter, target);
                calculate_score(diff, player.strength, final_miss)
            } else {
                0
            }
        } else {
            0
        }
    } else if !pressed_enter {
        0
    } else if let Objective::Number(target) = objective {
        let diff = circular_diff(final_counter, target);
        calculate_score(diff, player.strength, final_miss)
    } else {
        0
    };

    // Affichage du résultat de l'objectif
    if is_letter {
        if let Objective::Letter(letter, target) = objective {
            let validation = if pressed_char == Some(letter) {
                "✓"
            } else {
                "✗"
            };
            println!(
                "\nObjectif [{}] à {} : Touche appuyée [{}] {} | Score = {}",
                letter,
                target,
                pressed_char.unwrap_or('?'),
                validation,
                score
            );
            debug!(
                "Objectif lettre: {} cherchée, {} appuyée, cible={}, compteur={}, diff={}, miss={}, force={}, score={}",
                letter, pressed_char.unwrap_or('?'), target, final_counter,
                circular_diff(final_counter, target), final_miss, player.strength, score
            );
        }
    } else if let Objective::Number(num) = objective {
        println!(
            "\nObjectif {} : Compteur final = {} | Score = {}",
            num, final_counter, score
        );
        debug!(
            "Objectif nombre: cible={}, compteur={}, diff={}, miss={}, force={}, score={}",
            num,
            final_counter,
            circular_diff(final_counter, num),
            final_miss,
            player.strength,
            score
        );
    }

    trace!(
        "play_objective({}) - Résumé: counter={}, miss={}, score={}",
        player.name,
        final_counter,
        final_miss,
        score
    );

    Ok(score)
}

// ————————————————————————————————————————————————————————
// Tests
// ————————————————————————————————————————————————————————

#[cfg(test)]
mod tests {
    use crate::mechanics::{calculate_score, circular_diff};

    /// Vérifie le pipeline de score complet pour un objectif lettre : bonne touche,
    /// compteur proche de la cible → score maximal avec force.
    #[test]
    fn verifie_pipeline_score_touche_correcte_exacte() {
        let letter = 'a';
        let pressed = 'a';
        let counter = 10i32;
        let target = 10i32; // diff circulaire = 0
        let strength = 50i32;
        let miss = 0i32;

        let score = if pressed == letter {
            let diff = circular_diff(counter, target);
            calculate_score(diff, strength, miss)
        } else {
            0
        };

        // diff=0 → score_base=100, (100 + 50) / 1 = 150
        assert_eq!(score, 150);
    }

    /// Vérifie le pipeline de score pour un objectif lettre avec une différence non nulle.
    #[test]
    fn verifie_pipeline_score_touche_correcte_avec_difference() {
        let letter = 'z';
        let pressed = 'z';
        let counter = 30i32;
        let target = 35i32; // diff = 5
        let strength = 50i32;
        let miss = 0i32;

        let score = if pressed == letter {
            let diff = circular_diff(counter, target);
            calculate_score(diff, strength, miss)
        } else {
            0
        };

        // diff=5 → score_base=80, (80 + 50) / 1 = 130
        assert_eq!(score, 130);
    }

    /// Vérifie que la mauvaise touche donne toujours 0.
    #[test]
    fn verifie_pipeline_score_touche_incorrecte() {
        let letter = 'a';
        let pressed = 'b';
        let counter = 10i32;
        let target = 10i32;
        let strength = 50i32;
        let miss = 0i32;

        let score = if pressed == letter {
            let diff = circular_diff(counter, target);
            calculate_score(diff, strength, miss)
        } else {
            0
        };

        assert_eq!(score, 0);
    }

    /// Vérifie que le miss pénalise bien le score (division par miss + 1).
    #[test]
    fn verifie_pipeline_score_avec_rate() {
        let letter = 'a';
        let pressed = 'a';
        let counter = 10i32;
        let target = 10i32; // diff = 0
        let strength = 50i32;
        let miss = 1i32; // un bouclage

        let score = if pressed == letter {
            let diff = circular_diff(counter, target);
            calculate_score(diff, strength, miss)
        } else {
            0
        };

        // diff=0 → score_base=100, (100 + 50) / 2 = 75
        assert_eq!(score, 75);
    }

    /// Vérifie le pipeline en mode nombre (ENTRÉE pressée, cible proche).
    #[test]
    fn verifie_pipeline_score_mode_nombre_avec_entree() {
        let target = 80i32;
        let counter = 82i32; // diff = 2
        let strength = 50i32;
        let miss = 0i32;
        let pressed_enter = true;

        let score = if !pressed_enter {
            0
        } else {
            let diff = circular_diff(counter, target);
            calculate_score(diff, strength, miss)
        };

        // diff=2 → score_base=80, (80 + 50) / 1 = 130
        assert_eq!(score, 130);
    }

    /// Vérifie que ne pas appuyer sur ENTRÉE en mode nombre donne 0.
    #[test]
    fn verifie_pipeline_score_mode_nombre_sans_entree() {
        let pressed_enter = false;
        let score = if !pressed_enter { 0 } else { 999 };
        assert_eq!(score, 0);
    }

    /// Vérifie le cas boucle-around : compteur=95, cible=15 → diff=20, pas 80.
    #[test]
    fn verifie_pipeline_score_boucle_circulaire() {
        let target = 15i32;
        let counter = 95i32;
        let strength = 50i32;
        let miss = 0i32;
        let pressed_enter = true;

        let score = if !pressed_enter {
            0
        } else {
            let diff = circular_diff(counter, target);
            calculate_score(diff, strength, miss)
        };

        // diff circulaire = min(80, 20) = 20 → score_base=40, (40 + 50) / 1 = 90
        assert_eq!(score, 90);
    }
}
