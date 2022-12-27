#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{cmp::min, fs::File};

#[derive(Serialize, Deserialize, Clone, Copy)]
enum Suit {
    Spade,
    Club,
    Diamond,
    Heart,
}
#[derive(Serialize, Deserialize, Clone)]
struct Card {
    value: u8,
    suit: Suit,
}

#[derive(Serialize, Deserialize, Clone)]
struct Deck {
    cards: Vec<u8>,
}
impl Deck {
    const SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Diamond, Suit::Heart];
    fn new() -> Deck {
        let mut cards: Vec<u8> = (0..52).collect();
        cards.shuffle(&mut rand::thread_rng());
        Deck { cards }
    }
    fn random_card(&mut self) -> Card {
        let ind = self.cards.pop().expect("Deck is empty!");

        Card {
            value: ind / 4,
            suit: Self::SUITS[ind as usize % 4],
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum BettingRound {
    PreFlop,
    Flop,
    Turn,
    River,
}
impl BettingRound {
    fn next(&mut self) {
        *self = match *self {
            BettingRound::PreFlop => BettingRound::Flop,
            BettingRound::Flop => BettingRound::Turn,
            BettingRound::Turn => BettingRound::River,
            BettingRound::River => panic!("River is the final round"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct GameCfg {
    player_name: [String; 2],
    starting_chip: [u64; 2],
    small_blind_amount: u64,
    big_blind_amount: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct Player {
    name: String,
    hole: [Card; 2],
    stack: u64,
    bet_size: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct Game {
    deck: Deck,
    players: Vec<Player>,
    community: Vec<Card>,
    small_blind_index: u8,
    betting_round: BettingRound,
    pot_size: u64,
    min_raise: u64,
    previous_player_index: u8,
    previous_active_index: Option<u8>, // last person to bet/raise, None when the game starts
}
impl Game {
    fn new(cfg: GameCfg, small_blind_index: u8) -> Game {
        // create decks and players
        // there're two players
        let mut deck = Deck::new();
        let mut players = Vec::new();
        for i in 0..2 {
            players.push(Player {
                name: cfg.player_name[i].clone(),
                hole: [deck.random_card(), deck.random_card()],
                stack: cfg.starting_chip[i],
                bet_size: if i == small_blind_index as usize {
                    cfg.small_blind_amount
                } else {
                    cfg.big_blind_amount
                },
            });
        }
        // calculate pot size
        let pot_size = players.iter().map(|p| p.bet_size).sum();
        // return the new game
        Game {
            deck,
            players,
            community: Vec::new(),
            small_blind_index,
            betting_round: BettingRound::PreFlop,
            pot_size,
            min_raise: cfg.big_blind_amount,
            previous_player_index: (small_blind_index + 1) % 2, // big blind's position
            previous_active_index: None, // no raise at start of game, big blind doesn't count
        }
    }
    fn next_betting_round(&mut self) {
        match self.betting_round {
            BettingRound::PreFlop => {
                for player in self.players.iter_mut() {
                    player.bet_size = 0;
                }
                self.previous_player_index = (self.small_blind_index + 1) % 2;
                self.previous_active_index = None;
                for _ in 0..3 {
                    self.community.push(self.deck.random_card());
                }
                self.betting_round.next();
            }
            BettingRound::Flop => todo!(),
            BettingRound::Turn => todo!(),
            BettingRound::River => todo!(),
        }
    }
}
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_new_game,
            get_possible_actions,
            get_raise_range,
            act
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_new_game(game_number: u16) -> tauri::Result<Game> {
    // Load in game config
    let file = File::open("../poker.conf.json").expect("Can't read poker.conf.json");

    let game_cfg: GameCfg =
        serde_json::from_reader(file).expect("Can't read values from poker.conf.json");
    Ok(Game::new(game_cfg, (game_number % 2) as u8))
}

#[derive(Serialize, Deserialize)]
enum Action {
    Call(u64),
    Bet(u64),
    Raise(u64),
    Check,
    Fold,
}
#[tauri::command]
fn get_possible_actions(game: Game) -> Vec<Action> {
    let current_index = (game.previous_player_index + 1) % 2;
    if let Some(previous_active_index) = game.previous_active_index {
        if previous_active_index == current_index {
            // Went back to the original raiser, the round should end, no possible actions
            return Vec::new();
        }
    }

    let previous_bet = game.players[game.previous_player_index as usize].bet_size;
    let current_player = &game.players[current_index as usize];
    let mut possible_actions: Vec<Action> = Vec::new();

    if previous_bet > current_player.bet_size {
        // calling
        // can call all-in or matching the bet
        possible_actions.push(Action::Call(min(
            current_player.stack,
            previous_bet - current_player.bet_size,
        )));

        // raising
        if previous_bet - current_player.bet_size < current_player.stack {
            // if sufficient stack
            // possible to raise from min_raise to current_player.stack, or raise all-in
            // 0 as default value
            possible_actions.push(Action::Raise(0));
        }

        // folding
        possible_actions.push(Action::Fold);
    } else if previous_bet == current_player.bet_size {
        if current_player.bet_size == 0 {
            // betting
            // happens when betting round start,
            // possible to bet from min_raise to current_player.stack
            // 0 as default value
            possible_actions.push(Action::Bet(0));
        } else {
            // happens when everyone (except BB) called in pre-flop (all players' bet_size is BB)
            possible_actions.push(Action::Raise(0))
        }
        // checking
        possible_actions.push(Action::Check);
    }

    possible_actions
}

#[tauri::command]
fn get_raise_range(game: Game) -> std::ops::Range<u64> {
    let current_player = &game.players[(game.previous_player_index as usize + 1) % 2];
    // can raise from min_raise or go all-in
    min(current_player.stack, game.min_raise)..(current_player.stack + 1)
}

#[tauri::command]
fn act(game: Game, action: Action) -> Game {
    let mut new_game = game.clone();
    let current_player_index = (game.previous_player_index + 1) % 2;
    let current_player = &mut new_game.players[current_player_index as usize];
    match action {
        Action::Call(amount) => {
            current_player.bet_size += amount;
            new_game.pot_size += amount;
            new_game.previous_player_index = current_player_index;
            if matches!(new_game.betting_round, BettingRound::PreFlop) {
                new_game.previous_active_index = Some(current_player_index);
            }
            if let Some(previous_active_index) = new_game.previous_active_index {
                if previous_active_index == (current_player_index + 1) % 2 {
                    // if the next index is the previous_active_index, end the round
                    new_game.next_betting_round();
                }
            }
        }
        Action::Bet(_) => todo!(),
        Action::Raise(_) => todo!(),
        Action::Check => {
            new_game.previous_player_index = current_player_index;
            if let Some(previous_active_index) = new_game.previous_active_index {
                if previous_active_index == (current_player_index + 1) % 2 {
                    // if the next index is the previous_active_index, end the round
                    new_game.next_betting_round();
                }
            }
        }
        Action::Fold => todo!(),
    }
    new_game
}
