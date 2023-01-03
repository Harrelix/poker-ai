mod handtype;

use std::{cmp::Ordering, ops::Index};

use handtype::HandType;
use itertools::Itertools;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use self::handtype::get_all_hand_types;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum Suit {
    Spade,
    Club,
    Diamond,
    Heart,
}
impl Suit {
    const ALL_SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Diamond, Suit::Heart];
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq)]
pub struct Card {
    rank: u8,
    suit: Suit,
}
impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank
    }
}
impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Card {
    /// Compare cards
    /// The suit does not matter, eg. two of spade == two of clubs
    /// Assumes ace are higher than two
    fn cmp(&self, other: &Self) -> Ordering {
        if self.rank == 1 || self.rank > other.rank {
            return Ordering::Greater;
        }
        if self.rank < other.rank {
            return Ordering::Less;
        }
        return Ordering::Equal;
    }
}

/// Wrapper for type [Card; 5], makes sure that hand is always sorted
#[derive(Clone, Copy, Eq)]
pub struct Hand {
    _data: [Card; 5],
}
impl Hand {
    fn new(cards: [Card; 5]) -> Hand {
        let mut sorted = cards.clone();
        // sort low to high
        sorted.sort();
        Hand { _data: sorted }
    }
    fn iter(&self) -> impl Iterator<Item = &Card> {
        self._data.iter()
    }
    fn check(self, hand_type: impl HandType) -> bool {
        hand_type.check(self)
    }
    fn get_ranks_array(self) -> [u8; 5] {
        self._data
            .iter()
            .map(|card| card.rank)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap()
    }
    fn get_hand_type(self) -> &impl HandType {
        // for hand_type in get_all_hand_types() {
        //     if hand_type.check(self) {
        //         return *hand_type;
        //     }
        // }
        // HandType::HighCard
        todo!()
    }
    pub fn get_all_hands(cards: [Card; 7]) -> Vec<Hand> {
        cards
            .into_iter()
            .combinations(5)
            .map(|possible_hand| Hand::new(possible_hand.try_into().unwrap()))
            .collect()
    }
}
impl Index<usize> for Hand {
    type Output = Card;

    fn index(&self, index: usize) -> &Self::Output {
        &self._data[index]
    }
}
impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self._data == other._data
    }
}
impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        // compare ranking for now
        // the one with lower ranking is better
        // TODO: tie-breaks
        self.get_hand_type()
            .get_ranking()
            .cmp(&other.get_hand_type().get_ranking())
            .reverse()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Deck {
    cards: Vec<u8>,
}
impl Deck {
    pub fn new() -> Deck {
        // create a new shuffled deck
        let mut cards: Vec<u8> = (0..52).collect();
        cards.shuffle(&mut rand::thread_rng());
        Deck { cards }
    }
    pub fn random_card(&mut self) -> Card {
        // get the top card of the shuffled deck
        let ind = self.cards.pop().expect("Deck is empty!");

        Card {
            rank: ind / 4 + 1,
            suit: Suit::ALL_SUITS[ind as usize % 4],
        }
    }
}
