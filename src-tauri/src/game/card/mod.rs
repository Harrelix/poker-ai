use std::{array::IntoIter, cmp::Ordering, collections::LinkedList, fmt::Display, ops::Index};

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
impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Suit::Spade => write!(f, "Spades"),
            Suit::Club => write!(f, "Clubs"),
            Suit::Diamond => write!(f, "Diamonds"),
            Suit::Heart => write!(f, "Hearts"),
        }
    }
}

type Rank = u8;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq)]
pub struct Card {
    rank: Rank, // J, Q, K, A are 11, 12, 13, 1 respectively
    suit: Suit,
}
impl Card {
    fn new(rank: Rank, suit: Suit) -> Card {
        if rank == 0 || rank > 13 {
            panic!("Rank {} not valid. Should be between 1 and 13", rank);
        }
        Card { rank, suit }
    }
    fn display_rank(rank: Rank) -> String {
        match rank {
            1 => "Ace".into(),
            11 => "Jack".into(),
            12 => "Queen".into(),
            13 => "King".into(),
            _ => rank.to_string(),
        }
    }
}
impl PartialEq for Card {
    /// two cards are equal if they have the same rank
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
    /// compare cards.
    /// the suit does not matter, eg. two of spade == two of clubs.
    /// aces are highest.
    fn cmp(&self, other: &Self) -> Ordering {
        if self.rank == 1 {
            if other.rank == 1 {
                return Ordering::Equal;
            }
            return Ordering::Greater;
        }
        if other.rank == 1 {
            return Ordering::Less;
        }
        self.rank.cmp(&other.rank)
    }
}
impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} of {}", Card::display_rank(self.rank), self.suit)
    }
}

/// Wrapper for type [Card; 5], makes sure that hand is always sorted.
#[derive(Clone, Copy, Eq)]
pub struct Hand {
    cards: [Card; 5],

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
            cards: sorted,
            hand_type: None,
        }
    }
    fn into_iter(self) -> IntoIter<Card, 5> {
        self.cards.into_iter()
    }
    fn get_ranks_array(self) -> [Rank; 5] {
        self.cards
            .iter()
            .map(|card| card.rank)
            .collect::<Vec<Rank>>()
            .try_into()
            .unwrap()
    }
    /// returns the hand's type.
    /// O(1) if previously calculated.
    pub fn get_hand_type(self) -> HandType {
        // check if already calculated
        if let Some(hand_type) = self.hand_type {
            return hand_type;
        }

        HandType::get_hand(self)
    }
    /// returns all possible hand that can be made using current hole and community.
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
        &self.cards[index]
    }
}
impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_hand_type = self.get_hand_type();
        let other_hand_type = other.get_hand_type();
        let hand_type_cmp = self_hand_type.cmp(&other_hand_type);
        if hand_type_cmp.is_ne() {
            return hand_type_cmp; // return if there're no ties
        }
        // breaking tie, self_hand_type should be the same as other_hand_type

        /// helper function to compare the ranks of hands. takes in vec because
        /// sometimes we need to compare sections of hands.
        /// `h1` and `h2` has to be the same size and sorted low to high
        fn compare_ranks(h1: Vec<Card>, h2: Vec<Card>) -> Ordering {
            // going backwards from highest to lowest card of both hands
            for (c1, c2) in h1.iter().zip(h2).rev() {
                let compare_result = c1.cmp(&c2);
                if compare_result.is_ne() {
                    return compare_result;
                }
            }
            // all elements are equal
            Ordering::Equal
        }
        // closure that compare hands if there're no specific tie breaking rules
        let default_tie_break = || -> Ordering {
            return compare_ranks(self.cards.to_vec(), other.cards.to_vec());
        };

        fn straight_tie_break(self_straight_rank: Rank, other_straight_rank: Rank) -> Ordering {
            if self_straight_rank == 1 {
                if other_straight_rank == 1 {
                    return Ordering::Equal;
                }
                return Ordering::Greater;
            }
            if other_straight_rank == 1 {
                return Ordering::Less;
            }
            return self_straight_rank.cmp(&other_straight_rank);
        }
        let x_of_a_kind_tie_break = |self_rank: Rank, other_rank: Rank| -> Ordering {
            let compare_result = self_rank.cmp(&other_rank);
            if compare_result.is_ne() {
                return compare_result;
            }
            return compare_ranks(
                self.cards
                    .into_iter()
                    .filter(|card| card.rank != self_rank)
                    .collect(),
                other
                    .cards
                    .into_iter()
                    .filter(|card| card.rank != other_rank)
                    .collect(),
            );
        };
        fn full_house_tie_break(str: Rank, spr: Rank, otr: Rank, opr: Rank) -> Ordering {
            let compare_result = str.cmp(&otr);
            if compare_result.is_ne() {
                return compare_result;
            }
            spr.cmp(&opr)
        }
        let two_pair_tie_break = |spr1: Rank, spr2: Rank, opr1: Rank, opr2: Rank| -> Ordering {
            let cmp_res_1 = spr1.cmp(&opr1);
            if cmp_res_1.is_ne() {
                return cmp_res_1;
            }
            let cmp_res_2 = spr2.cmp(&opr2);
            if cmp_res_2.is_ne() {
                return cmp_res_2;
            }
            // remove the two pair and compare ranks if both pair are the same
            return compare_ranks(
                self.cards
                    .into_iter()
                    .filter(|card| card.rank != spr1 && card.rank != spr2)
                    .collect(), // should have one card left
                other
                    .cards
                    .into_iter()
                    .filter(|card| card.rank != opr1 && card.rank != opr2)
                    .collect(), // should have one card left
            );
        };

        match (self_hand_type, other_hand_type) {
            (HandType::RoyalFlush, HandType::RoyalFlush) => Ordering::Equal, // royal flushes are the same
            (HandType::StraightFlush(ssr), HandType::StraightFlush(osr)) => {
                straight_tie_break(ssr, osr)
            }
            (HandType::FourOfAKind(sfoakr), HandType::FourOfAKind(ofoakr)) => {
                x_of_a_kind_tie_break(sfoakr, ofoakr)
            }
            (HandType::FullHouse(str, spr), HandType::FullHouse(otr, opr)) => {
                full_house_tie_break(str, spr, otr, opr)
            }
            (HandType::Flush, HandType::Flush) => default_tie_break(),
            (HandType::Straight(ssr), HandType::Straight(osr)) => straight_tie_break(ssr, osr),
            (HandType::ThreeOfAKind(stoakr), HandType::ThreeOfAKind(otoakr)) => {
                x_of_a_kind_tie_break(stoakr, otoakr)
            }
            (HandType::TwoPair(spr1, spr2), HandType::TwoPair(opr1, opr2)) => {
                two_pair_tie_break(spr1, spr2, opr1, opr2)
            }
            (HandType::OnePair(spr), HandType::OnePair(opr)) => x_of_a_kind_tie_break(spr, opr),
            (HandType::HighCard(_), HandType::HighCard(_)) => default_tie_break(),
            _ => panic!(
                "Hands are not the same type so can't tie break. 
                \nSelf hand type: {:?}.
                \nOther hand type:{:?}",
                self_hand_type, other_hand_type
            ),
        }
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
        Card::new(index / 4 + 1, Suit::ALL_SUITS[index as usize % 4])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;
    use rand::seq::SliceRandom;

    fn create_hand(ranks: [Rank; 5], suits: [Suit; 5]) -> Hand {
        Hand::new(
            ranks
                .iter()
                .zip(suits.iter())
                .map(|(&rank, &suit)| Card::new(rank, suit))
                .collect::<Vec<Card>>()
                .try_into()
                .unwrap(),
        )
    }
    fn random_suits_no_flush() -> [Suit; 5] {
        let mut rng = rand::thread_rng();
        loop {
            let mut suits = Vec::new();
            for _ in 0..5 {
                suits.push(*Suit::ALL_SUITS.choose(&mut rng).unwrap());
            }
            if suits.iter().any(|&suit| suit != suits[0]) {
                return suits.try_into().unwrap();
            }
        }
    }

    #[test]
    fn hand_compare_tests() {
        let highcard1 = create_hand([3, 5, 2, 6, 9], random_suits_no_flush());
        let highcard2 = create_hand([1, 2, 3, 7, 4], random_suits_no_flush());

        let pair1 = create_hand([2, 3, 10, 2, 4], random_suits_no_flush());
        let pair2 = create_hand([5, 6, 7, 5, 2], random_suits_no_flush());
        let pair3 = create_hand([5, 6, 2, 5, 1], random_suits_no_flush());

        let toak1 = create_hand([2, 2, 2, 1, 4], random_suits_no_flush());

        let straight1 = create_hand([3, 4, 5, 6, 7], random_suits_no_flush());
        let straight2 = create_hand([1, 2, 3, 4, 5], random_suits_no_flush());

        let flush1 = create_hand([7, 2, 3, 4, 6], [Suit::Heart; 5]);
        let flush2 = create_hand([3, 4, 5, 7, 8], [Suit::Club; 5]);

        let full_house = create_hand([9, 9, 9, 7, 7], random_suits_no_flush());
        let foak = create_hand([3, 3, 3, 3, 9], random_suits_no_flush());
        let royal_flush = create_hand([10, 11, 12, 13, 1], [Suit::Heart; 5]);

        assert_eq!(highcard1.cmp(&highcard2), Ordering::Less);
        assert_eq!(pair1.cmp(&pair2), Ordering::Less);
        assert_eq!(pair2.cmp(&pair3), Ordering::Less);
        assert_eq!(highcard1.cmp(&pair2), Ordering::Less);
        assert_eq!(straight1.cmp(&straight2), Ordering::Greater);
        assert_eq!(flush1.cmp(&flush2), Ordering::Less);
        assert_eq!(toak1.cmp(&pair3), Ordering::Greater);
        assert_eq!(royal_flush.cmp(&foak), Ordering::Greater);
        assert_eq!(foak.cmp(&full_house), Ordering::Greater);
    }
}
