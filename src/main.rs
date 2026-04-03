//! # Duel Game
//!
//! Mini-jeu de duel au tour par tour en Rust.
//!
//! ## Gameplay
//!
//! - Chaque joueur agit à tour de rôle
//! - Des objectifs sont générés à chaque manche
//! - Les scores influencent les dégâts
//! - Le jeu continue jusqu'à épuisement de la vitalité
//!
//! ## Lancement
//!
//! ```text
//! cargo run -- --name1 Amaury --name2 Julien --vitality 50 --objectifs 5
//! ```
//!
//! ## Architecture
//!
//! | Module          | Rôle                                                     |
//! |-----------------|----------------------------------------------------------|
//! | [`args`]        | Parsing des arguments CLI via `clap`                     |
//! | [`player`]      | Struct [`player::Player`] et ses caractéristiques        |
//! | [`mechanics`]   | Logique pure : diff circulaire, score, génération objets |
//! | [`counter`]     | Compteur temps-réel (threads + atomics)                  |
//! | [`input`]       | Lecture stdin (ENTRÉE, choix 1/2)                        |
//! | [`round`]       | Orchestration d'un tour, d'une manche, du poison         |
//!
//! ## Documentation
//!
//! Génère ET ouvre directement dans le navigateur :
//! ```text
//! cargo doc --open
//! ```

pub mod args;
pub mod counter;
pub mod input;
pub mod mechanics;
pub mod player;
pub mod round;

use args::Args;
use clap::Parser;
use log::{debug, error, info, warn};
use player::Player;
use round::play_round;
use std::io::{self, Write};

fn main() -> Result<(), String> {
    env_logger::init();
    info!("Démarrage du jeu de duel");

    let args = Args::parse();
    info!(
        "Paramètres : name1={} name2={} vitality={} objectifs={} speed={} strength={}",
        args.name1, args.name2, args.vitality, args.objectifs, args.speed, args.strength
    );

    println!("Choisissez le mode de jeu :");
    println!("1. Mode ENTER (objectifs numériques)");
    println!("2. Mode touches (objectifs lettres)");
    let mode_choice = input::read_choice_1_or_2()?;
    let is_number_mode = mode_choice == 1;

    let mut names = vec![args.name1, args.name2];
    if let Some(ref n) = args.name3 {
        names.push(n.clone());
    }
    if let Some(ref n) = args.name4 {
        names.push(n.clone());
    }
    if names.len() < 2 || names.len() > 4 {
        let msg = "Le nombre de joueurs doit être entre 2 et 4.".to_string();
        error!("{}", msg);
        return Err(msg);
    }

    // Boucle permettant de rejouer sans relancer le binaire
    loop {
        println!("\n\n##### Démarrage de la partie #####");
        let player_names = names
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" vs ");
        println!("Joueurs : {}", player_names);
        println!(
            "Vitalité initiale : {} | Objectifs par tour : {}",
            args.vitality, args.objectifs
        );
        println!("──────────────────────────────────\n");
        info!(
            "Nouvelle partie lancée : {} joueurs, mode={}",
            names.len(),
            if is_number_mode { "ENTER" } else { "LETTRE" }
        );
        debug!("Joueurs : {}", player_names);

        // Réinitialisation complète des joueurs à chaque partie
        let mut players = names
            .iter()
            .map(|name| Player::new(name.clone(), args.vitality, args.speed, args.strength))
            .collect::<Vec<_>>();
        let mut eliminated_order = Vec::new();

        let mut round = 1;
        loop {
            let (cont, new_elim) = play_round(round, &mut players, args.objectifs, is_number_mode)
                .map_err(|e| {
                    error!("Erreur fatale à la manche {} : {}", round, e);
                    e
                })?;

            eliminated_order.extend(new_elim.clone());
            debug!(
                "Fin manche {}: joueurs éliminés cette manche = {:?}",
                round,
                new_elim
                    .iter()
                    .map(|&idx| players[idx].name.clone())
                    .collect::<Vec<_>>()
            );

            if !cont {
                // Afficher le podium par ordre d'élimination
                if let Some(winner_idx) = players.iter().position(|p| p.vitality > 0) {
                    println!("\n🏆 Classement final :");
                    println!("1. {} (Gagnant)", players[winner_idx].name);
                    info!("Fin de partie - Gagnant: {}", players[winner_idx].name);
                    // Les éliminés dans l'ordre inverse (dernier éliminé en 2ème place)
                    for (pos, &elim_idx) in eliminated_order.iter().rev().enumerate() {
                        println!("{}. {}", pos + 2, players[elim_idx].name);
                        debug!("Classement place {}: {}", pos + 2, players[elim_idx].name);
                    }
                } else {
                    warn!("Fin de partie : tous les joueurs sont éliminés (égalité complète)");
                }
                break;
            }
            round += 1;
        }

        println!("\n##### Partie terminée #####");
        println!("Relancer une partie ? [Y/N]");
        print!("> ");
        io::stdout()
            .flush()
            .map_err(|e| format!("Erreur flush : {e}"))?;

        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .map_err(|e| format!("Erreur lecture : {e}"))?;

        if answer.trim().to_lowercase() != "y" {
            println!("À bientôt !");
            info!("Fin du programme.");
            break;
        }
    }

    Ok(())
}
