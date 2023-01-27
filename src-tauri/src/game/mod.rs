mod card;

use std::{cmp::min, ops::RangeInclusive};

use self::card::{Card, Deck, Hand};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameCfg {
    player_name: [String; 2],
    starting_chip: [usize; 2],
    small_blind_amount: usize,
    big_blind_amount: usize,
    first_dealer_index: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Player {
    name: String,
    hole: [Card; 2],
    stack: usize,
    bet_size: usize,
    folded: bool,
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    Call,
    Bet(usize),
    Raise(usize),
    Check,
    Fold,
}

#[derive(Serialize, Deserialize, Clone)]
enum BettingRound {
    PreFlop,
    Flop,
    Turn,
    River,
}
impl BettingRound {
    /// set betting round to next
    /// loops back to pre-flop if it's the river
    fn next(&mut self) {
        *self = match *self {
            BettingRound::PreFlop => BettingRound::Flop,
            BettingRound::Flop => BettingRound::Turn,
            BettingRound::Turn => BettingRound::River,
            BettingRound::River => BettingRound::PreFlop,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    cfg: GameCfg,
    deck: Deck,
    players: Vec<Player>,
    community: Vec<Card>,
    dealer_index: usize,
    small_blind_index: usize, // != dealer_index + 1 in heads up poker
    betting_round: BettingRound,
    pot_size: usize,
    min_raise: usize,
    current_player_index: usize,
    previous_active_index: Option<usize>, // last person to bet/raise, None when game starts
}
impl Game {
    const NUM_PLAYER: usize = 2;
    fn get_small_blind_index(dealer_index: usize) -> usize {
        if Game::NUM_PLAYER != 2 {
            (dealer_index + 1) % Game::NUM_PLAYER // left of dealer
        } else {
            dealer_index // the blinds swap positions in heads up poker
        }
    }
    fn get_first_player_index(pre_flop: bool, dealer_index: usize) -> usize {
        if pre_flop {
            //  big blind has to act last on pre-flop
            if Game::NUM_PLAYER != 2 {
                // left of BB act first
                return (dealer_index + 3) % Game::NUM_PLAYER;
            }
            // dealer who is small blind act first
            return dealer_index;
        }
        // dealer acts last on post-flop
        (dealer_index + 1) % Game::NUM_PLAYER // BB if heads up else SB
    }
    /// assign the blind amounts to the players in the blind index
    /// returns error if player's stack isn't enough
    fn assign_blinds(
        players: &mut Vec<Player>,
        small_blind_index: usize,
        small_blind_amount: usize,
        big_blind_index: usize,
        big_blind_amount: usize,
    ) -> Result<(), String> {
        for (index, player) in players.iter_mut().enumerate() {
            // set bet size to blinds if necessary
            let bet_size = if index == small_blind_index {
                small_blind_amount
            } else if index == big_blind_index {
                big_blind_amount
            } else {
                0
            };
            // check if player has enough
            if bet_size > player.stack {
                return Err(format!(
                    "{} don't have enough stack for blind amount",
                    player.name
                ));
            }
            player.bet_size = bet_size;
            player.stack -= bet_size;
        }
        Ok(())
    }

    /// make new game based on cfg
    /// returns error when cfg's don't have enough for blinds
    pub fn new(cfg: GameCfg) -> Result<Game, String> {
        // find the play order
        let dealer_index = cfg.first_dealer_index;
        let small_blind_index = Game::get_small_blind_index(dealer_index);
        let current_player_index = Game::get_first_player_index(true, dealer_index);

        // create decks and players
        let mut deck = Deck::new();
        let mut players = Vec::with_capacity(Game::NUM_PLAYER);
        for i in 0..Game::NUM_PLAYER {
            players.push(Player {
                name: cfg.player_name[i].clone(),
                hole: [deck.random_card(), deck.random_card()],
                stack: cfg.starting_chip[i],
                bet_size: 0,
                folded: false,
            });
        }
        // assign blinds
        Game::assign_blinds(
            &mut players,
            small_blind_index,
            cfg.small_blind_amount,
            (small_blind_index + 1) % Game::NUM_PLAYER,
            cfg.big_blind_amount,
        )?;
        // minimum raise at start of game is big blind
        let min_raise = cfg.big_blind_amount;
        // return the new game
        Ok(Game {
            cfg,
            deck,
            players,
            community: Vec::new(),
            dealer_index,
            small_blind_index,
            betting_round: BettingRound::PreFlop,
            pot_size: 0,
            min_raise,
            current_player_index,
            previous_active_index: None, // no raise at start of game, BB doesn't count
        })
    }

    fn get_previous_player_index(&self) -> usize {
        if self.current_player_index != 0 {
            self.current_player_index - 1
        } else {
            Game::NUM_PLAYER - 1
        }
    }

    fn get_previous_bet(&self) -> usize {
        self.players[self.get_previous_player_index()].bet_size
    }

    /// increment current_player_index
    /// skipping folded players
    fn next_player(&mut self) {
        loop {
            self.current_player_index = (self.current_player_index + 1) % Game::NUM_PLAYER;

            if !self.players[self.current_player_index].folded {
                break;
            }
        }
    }

    /// set self to a next game
    /// assumes bet_size is already reset
    fn go_to_next_game(&mut self, winners_indices: Vec<usize>) {
        // split the pot evenly between the winners
        for index in winners_indices.iter() {
            self.players[*index].stack += self.pot_size / winners_indices.len();
        }
        // TODO: decides who gets the leftover chip
        // rotates button
        let dealer_index = (self.dealer_index + 1) % Game::NUM_PLAYER;
        let small_blind_index = Game::get_small_blind_index(dealer_index);
        let mut players = self.players.clone();
        // assign blinds
        Game::assign_blinds(
            &mut players,
            small_blind_index,
            self.cfg.small_blind_amount,
            (small_blind_index + 1) % Game::NUM_PLAYER,
            self.cfg.big_blind_amount,
        )
        .unwrap(); // panics if players don't have enough stack

        // reset folded
        for mut player in players.iter_mut() {
            player.folded = false;
        }
        // assign self to new game
        *self = Game {
            cfg: self.cfg.clone(),
            deck: Deck::new(),
            players,
            community: Vec::new(),
            dealer_index,
            small_blind_index,
            betting_round: BettingRound::PreFlop,
            pot_size: 0,
            min_raise: self.cfg.big_blind_amount,
            current_player_index: Game::get_first_player_index(true, dealer_index),
            previous_active_index: None,
        };
    }

    pub fn next_betting_round(&mut self) {
        let mut deal_cards_to_community = |num_card: usize| {
            for _ in 0..num_card {
                self.community.push(self.deck.random_card());
            }
        };

        /// return the winning player indices after comparing hands
        fn showdown(game: &Game) -> Vec<usize> {
            // convert community to array
            // also double check to see if community is full
            let community_array: [Card; 5] = game
                .community
                .clone()
                .try_into()
                .unwrap_or_else(|v: Vec<Card>| panic!("Community not full ({}/5)", v.len()));
            // find best hand of all players
            let best_hands: Vec<Hand> = game
                .players
                .iter()
                .map(|player| {
                    Hand::get_all_hands(player.hole, community_array)
                        .into_iter()
                        .max()
                        .unwrap()
                })
                .collect();
            // get all the winners
            let first_hand = best_hands[0];
            let (winning_hand, winning_players_indices) = best_hands.into_iter().enumerate().fold(
                (first_hand, Vec::new()),
                |(max_hand, mut indices), (index, hand)| {
                    if hand > max_hand {
                        return (hand, vec![index]);
                    }
                    if hand == max_hand {
                        indices.push(index);
                    }
                    return (max_hand, indices);
                },
            );
            println!(
                "Winner: {}",
                winning_players_indices
                    .iter()
                    .map(|i| game.players[*i].name.clone())
                    .join(", ")
            );
            println!("Hand type: {}", winning_hand.get_hand_type());
            winning_players_indices
        }

        // reset min_raise
        self.min_raise = self.cfg.big_blind_amount;
        // add up and reset bets
        for player in self.players.iter_mut() {
            self.pot_size += player.bet_size;
            player.bet_size = 0;
        }
        // starting player
        self.current_player_index = Game::get_first_player_index(false, self.dealer_index);
        self.previous_active_index = None;
        // deal community cards and set to next betting round
        match self.betting_round {
            BettingRound::PreFlop => deal_cards_to_community(3),
            BettingRound::Flop => deal_cards_to_community(1),
            BettingRound::Turn => deal_cards_to_community(1),
            BettingRound::River => {
                let winners_indices = showdown(self);
                // go to next game
                self.go_to_next_game(winners_indices);
                return; // skip the self.betting_round.next()
            }
        }
        self.betting_round.next();
    }

    /// return possible actions for current player
    pub fn get_possible_actions(&self) -> Vec<Action> {
        if let Some(previous_active_index) = self.previous_active_index {
            if previous_active_index == self.current_player_index {
                // went back to original raiser, round should end, no possible actions
                return Vec::new();
            }
        }

        let previous_bet = self.get_previous_bet();
        let current_player = &self.players[self.current_player_index];
        let mut possible_actions: Vec<Action> = Vec::new();

        if previous_bet > current_player.bet_size {
            // can call, raise, or fold
            // calling (can call all-in or matching the bet)
            possible_actions.push(Action::Call);

            // raising
            if previous_bet - current_player.bet_size < current_player.stack {
                // if sufficient stack
                // can raise from min_raise to current_player.stack or raise all-in
                // 0 as default value
                possible_actions.push(Action::Raise(0));
            }

            // folding
            possible_actions.push(Action::Fold);
        } else if previous_bet == current_player.bet_size {
            // can bet/raise or check
            if current_player.bet_size == 0 {
                // betting
                // happens when betting round start,
                // can bet from min_raise to current_player.stack
                // can't bet with empty stack
                if current_player.stack != 0 {
                    // 0 as default value
                    possible_actions.push(Action::Bet(0));
                }
            } else {
                // happens when everyone (except BB) called in pre-flop
                // (all players' bet_size is BB)
                possible_actions.push(Action::Raise(0));
            }
            // checking
            possible_actions.push(Action::Check);
        }

        possible_actions
    }

    /// get amount of chips that we call
    /// returns None if we can't call
    pub fn get_call_amount(&self) -> Option<usize> {
        // check if can't call
        if self
            .get_possible_actions()
            .iter()
            .all(|action| !matches!(action, Action::Call))
        {
            return None;
        }
        let current_player = &self.players[self.current_player_index];
        let previous_bet = self.get_previous_bet();
        let amount = min(current_player.stack, previous_bet - current_player.bet_size);
        Some(amount)
    }

    /// get amount of chips range that we raise by or bet
    /// returns None if we can't raise or bet
    pub fn get_raise_or_bet_range(&self) -> Option<RangeInclusive<usize>> {
        // check if can't raise or bet
        if !self
            .get_possible_actions()
            .iter()
            .any(|action| matches!(action, Action::Raise(_) | Action::Bet(_)))
        {
            return None;
        }
        let current_player = &self.players[self.current_player_index];
        let previous_bet = self.get_previous_bet();
        let max_amount = current_player.bet_size + current_player.stack - previous_bet;
        let range = min(max_amount, self.min_raise)..=max_amount;
        Some(range)
    }

    pub fn act(&self, action: Action) -> Result<Game, String> {
        fn call(new_game: &mut Game) -> Result<(), String> {
            match new_game.get_call_amount() {
                // update stack and bet size if we can call
                Some(amount) => {
                    let current_player = &mut new_game.players[new_game.current_player_index];
                    current_player.stack -= amount;
                    current_player.bet_size += amount;
                }
                // if calling is not legal
                None => return Err("Player can't call at this point".into()),
            }

            // set to next player
            new_game.next_player();
            if let Some(previous_active_index) = new_game.previous_active_index {
                if previous_active_index == new_game.current_player_index {
                    // if the next index is the last to bet/raise, end round
                    new_game.next_betting_round();
                }
            } else if matches!(new_game.betting_round, BettingRound::PreFlop) {
                // if first to call on pre-flop, set previous_active_index
                new_game.previous_active_index = Some(new_game.get_previous_player_index());
            }
            Ok(())
        }

        fn bet_or_raise(new_game: &mut Game, bet: bool, amount: usize) -> Result<(), String> {
            if let Some(raise_range) = new_game.get_raise_or_bet_range() {
                // checks if amount ir legal
                if !raise_range.contains(&amount) {
                    // return illegal amount error
                    return if bet {
                        Err(format!("{} is an illegal bet amount", amount))
                    } else {
                        Err(format!("{} is an illegal raise amount", amount))
                    };
                }
            } else {
                // return illegal action error
                return if bet {
                    Err("Player can't bet at this point".into())
                } else {
                    Err("Player can't raise at this point".into())
                };
            }
            let previous_bet = new_game.get_previous_bet();
            let current_player = &mut new_game.players[new_game.current_player_index];

            // update stack, bet size, and min_raise
            current_player.stack -= previous_bet + amount - current_player.bet_size;
            current_player.bet_size = previous_bet + amount;
            new_game.min_raise = amount;

            // set to next player
            new_game.previous_active_index = Some(new_game.current_player_index);
            new_game.next_player();

            Ok(())
        }

        fn check(new_game: &mut Game) -> Result<(), String> {
            new_game.next_player();

            // check if the next player is the last one who bet
            if let Some(previous_active_index) = new_game.previous_active_index {
                if previous_active_index == new_game.current_player_index {
                    // if the next index is the last to bet/raise, end round
                    new_game.next_betting_round();
                }
            } else {
                // if previous_active_index is None
                // set most recent active player index to last player
                new_game.previous_active_index = Some(new_game.get_previous_player_index());
            }
            Ok(())
        }
        fn fold(new_game: &mut Game) -> Result<(), String> {
            let current_player = &mut new_game.players[new_game.current_player_index];
            current_player.folded = true;

            // check if only one person remaining
            let remaining_players_indices: Vec<usize> = new_game
                .players
                .iter()
                .enumerate()
                .filter(|(_index, player)| !player.folded) // if player not folded
                .map(|(index, _player)| index) // add their index to vec
                .collect();
            if remaining_players_indices.len() == 1 {
                for player in new_game.players.iter_mut() {
                    new_game.pot_size += player.bet_size;
                    player.bet_size = 0;
                }
                // go to next game
                new_game.go_to_next_game(remaining_players_indices);
                return Ok(());
            }
            check(new_game)
        }
        // create new game
        let mut new_game = self.clone();
        // execute depends on action
        let result = match action {
            Action::Call => call(&mut new_game),
            Action::Bet(amount) => bet_or_raise(&mut new_game, true, amount),
            Action::Raise(amount) => bet_or_raise(&mut new_game, false, amount),
            Action::Check => check(&mut new_game),
            Action::Fold => fold(&mut new_game),
        };
        match result {
            Ok(_) => Ok(new_game),
            Err(e) => Err(e),
        }
    }
}
