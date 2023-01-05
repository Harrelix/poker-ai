use std::{cmp::Ordering, collections::LinkedList, ops::Index};

use itertools::Itertools;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

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
    rank: u8, // J, Q, K, A are 11, 12, 13, 1 respectively
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
    fn get_ranks_array(self) -> [u8; 5] {
        self._data
            .iter()
            .map(|card| card.rank)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap()
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
        todo!()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Deck {
    cards: LinkedList<u8>, // u8 represent card's index in a sorted deck
}
impl Deck {
    pub fn new() -> Deck {
        // create a new shuffled deck
        let mut cards: Vec<u8> = (0..52).collect();
        cards.shuffle(&mut rand::thread_rng());
        Deck {
            cards: cards.into_iter().collect(),
        }
    }
    pub fn random_card(&mut self) -> Card {
        // get the top card of the shuffled deck
        let index = self.cards.pop_front().expect("Deck is empty!");

        // calculate rank and suit based on index
        Card {
            rank: index / 4 + 1,
            suit: Suit::ALL_SUITS[index as usize % 4],
        }
    }
}
