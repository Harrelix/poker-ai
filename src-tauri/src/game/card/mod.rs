use std::{array::IntoIter, cmp::Ordering, collections::LinkedList, ops::Index};

mod handtype;

use concat_arrays::concat_arrays;
use itertools::Itertools;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use self::handtype::HandType;

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
    data: [Card; 5],

    // hand_type is initially None.
    // is Some when `get_hand_type` is first called
    // acts as a cache so we don't have to check every hand type
    // when we want to get hand type
    hand_type: Option<HandType>,
}
impl Hand {
    fn new(cards: [Card; 5]) -> Hand {
        let mut sorted = cards.clone();
        // sort low to high
        sorted.sort();
        Hand {
            data: sorted,
            hand_type: None,
        }
    }
    fn iter(&self) -> impl Iterator<Item = &Card> {
        self.data.iter()
    }
    fn into_iter(self) -> IntoIter<Card, 5> {
        self.data.into_iter()
    }
    fn get_ranks_array(self) -> [u8; 5] {
        self.data
            .iter()
            .map(|card| card.rank)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap()
    }
    fn get_hand_type(self) -> HandType {
        // check if already calculated
        if let Some(hand_type) = self.hand_type {
            return hand_type;
        }
        // else loop through all hand types in order
        // to find the highest type this hand is
        for hand_type in HandType::ALL_HAND_TYPES {
            if hand_type.check_hand(self) {
                return hand_type;
            }
        }
        panic!("Hand {:?} type is unknown", self.data)
    }
    fn is_hand_type(self, hand_type: HandType) -> bool {
        hand_type.check_hand(self)
    }
    /// returns all possible hand that can be made using current hole and community
    pub fn get_all_hands(hole: [Card; 2], community: [Card; 5]) -> Vec<Hand> {
        // concatenate into a array with length 7
        let cards: [Card; 7] = concat_arrays!(hole, community);

        cards
            .into_iter()
            .combinations(5) // get all possible combination length 5
            .map(|possible_hand| Hand::new(possible_hand.try_into().unwrap()))
            .collect()
    }
}
impl Index<usize> for Hand {
    type Output = Card;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl PartialEq for Hand {
    /// for consistency with Ord, two hands are equal when their ranking is equal
    fn eq(&self, other: &Self) -> bool {
        self.get_hand_type() == other.get_hand_type()
    }
}
impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_hand_type().cmp(&other.get_hand_type())
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
