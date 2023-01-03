#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod game;
use game::{Action, Game};
use std::{fs::File, ops::RangeInclusive};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_new_game,
            get_possible_actions,
            get_call_amount,
            get_raise_or_bet_range,
            act
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_new_game() -> Result<Game, String> {
    // Load in game config
    match File::open("../poker.conf.json") {
        Err(e) => Err(format!("Can't read poker.conf.json: {}", e)),
        // Parse the config
        Ok(file) => match serde_json::from_reader(file) {
            Err(e) => Err(format!("Can't parse poker.conf.json: {}", e)),
            // Returns new game
            Ok(game_cfg) => Game::new(game_cfg),
        },
    }
}

#[tauri::command]
fn get_possible_actions(game: Game) -> Vec<Action> {
    game.get_possible_actions()
}

#[tauri::command]
fn get_call_amount(game: Game) -> Option<usize> {
    game.get_call_amount()
}

#[tauri::command]
fn get_raise_or_bet_range(game: Game) -> Option<RangeInclusive<usize>> {
    game.get_raise_or_bet_range()
}

#[tauri::command]
fn act(game: Game, action: Action) -> Result<Game, String> {
    game.act(action)
}
