use std::{cmp::Ordering, fmt::Display};

use super::Card;
use super::Hand;
use super::Rank;

type Ranking = u8;

#[derive(Clone, Copy, Eq, Debug)]
pub enum HandType {
    RoyalFlush,
    StraightFlush(Rank),   // Rank is the rank of highest card in the straight
    FourOfAKind(Rank),     // Rank is the four of a kind rank
    FullHouse(Rank, Rank), // first Rank is the trio's rank, second is the pair's
    Flush,
    Straight(Rank),      // Rank is the rank of highest card in the straight
    ThreeOfAKind(Rank),  // Rank is the rank of the three of a kind
    TwoPair(Rank, Rank), // Rank are the two ranks of the pair, the first Rank is higher
    OnePair(Rank),       // Rank is the rank of highest pair (usually the only one)
    HighCard(Rank),      // Rank is the highest card
}
impl HandType {
    pub fn get_hand(hand: Hand) -> HandType {
        // cards' suits are the same with the first one
        let is_flush = hand.into_iter().all(|card| card.suit == hand[0].suit);

        // ====royal flush====
        if hand.get_ranks_array() == [10, 11, 12, 13, 1] && is_flush {
            return HandType::RoyalFlush;
        }
        // ====straight flush====
        let straight_rank = if hand[4].rank == 1 {
            // if hand has a 1, there're two cases
            // check for 12345 and TJQKA straight
            let a = hand.get_ranks_array();
            if a == [2, 3, 4, 5, 1] {
                Some(5)
            } else if a == [10, 11, 12, 13, 1] {
                Some(1)
            } else {
                None
            }
        } else {
            // check if the ranks are consecutive
            if (1..5).all(|i| hand[i].rank == hand[i - 1].rank + 1) {
                Some(hand[4].rank) // hand is straight
            } else {
                None
            }
        };
        if let Some(r) = straight_rank {
            if is_flush {
                return HandType::StraightFlush(r);
            }
        }
        // ====four of a kind====
        let x_of_a_kind = |x: usize| -> Option<Rank> {
            for i in ((x - 1)..5).rev() {
                // sorted so only need to check the two ends
                if hand[i].rank == hand[i - (x - 1)].rank {
                    return Some(hand[i].rank);
                }
            }
            None
        };
        if let Some(r) = x_of_a_kind(4) {
            return HandType::FourOfAKind(r);
        }

        // ====full house====
        // two possible configuration: [A, A, A, B, B] or [B, B, A, A, A]
        if hand[0].rank == hand[2].rank && hand[3].rank == hand[4].rank {
            return HandType::FullHouse(hand[0].rank, hand[3].rank);
        }
        if hand[0].rank == hand[1].rank && hand[2].rank == hand[4].rank {
            return HandType::FullHouse(hand[2].rank, hand[0].rank);
        }

        // ====flush====
        if is_flush {
            return HandType::Flush;
        }

        // ====straight====
        if let Some(r) = straight_rank {
            return HandType::Straight(r);
        }

        // ====three of a kind====
        if let Some(r) = x_of_a_kind(3) {
            return HandType::ThreeOfAKind(r);
        }

        // ====two pair====
        let mut prev_pair = None;
        for i in (1..5).rev() {
            // sorted so only need to check neighboring cards
            if hand[i].rank == hand[i - 1].rank {
                match prev_pair {
                    Some(r) => return HandType::TwoPair(r, hand[i].rank),
                    None => prev_pair = Some(hand[i].rank),
                }
            }
        }
        // ====one pair====
        // check if found one pair
        if let Some(r) = prev_pair {
            return HandType::OnePair(r);
        }

        // ====high card====
        // reached the end, meaning it's not any other type
        HandType::HighCard(hand[4].rank)
    }
    pub fn get_ranking(self) -> Ranking {
        match self {
            HandType::RoyalFlush => 1,
            HandType::StraightFlush(_) => 2,
            HandType::FourOfAKind(_) => 3,
            HandType::FullHouse(_, _) => 4,
            HandType::Flush => 5,
            HandType::Straight(_) => 6,
            HandType::ThreeOfAKind(_) => 7,
            HandType::TwoPair(_, _) => 8,
            HandType::OnePair(_) => 9,
            HandType::HighCard(_) => 10,
        }
    }
}
impl PartialEq for HandType {
    fn eq(&self, other: &Self) -> bool {
        self.get_ranking() == other.get_ranking()
    }
}
impl PartialOrd for HandType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HandType {
    fn cmp(&self, other: &Self) -> Ordering {
        // this hand type is greater if it's ranking is smaller and vice versa
        self.get_ranking().cmp(&other.get_ranking()).reverse()
    }
}
impl Display for HandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            HandType::RoyalFlush => write!(f, "Royal Flush"),
            HandType::StraightFlush(r) => {
                write!(f, "Straight Flush (rank {})", Card::display_rank(r))
            }
            HandType::FourOfAKind(r) => {
                write!(f, "Four Of A Kind (rank {})", Card::display_rank(r))
            }
            HandType::FullHouse(tr, pr) => write!(
                f,
                "Straight Flush (trio rank {}; pair rank {})",
                Card::display_rank(tr),
                Card::display_rank(pr)
            ),

            HandType::Flush => write!(f, "Flush"),
            HandType::Straight(r) => write!(f, "Straight (rank {})", Card::display_rank(r)),
            HandType::ThreeOfAKind(r) => {
                write!(f, "Three Of A Kind (rank {})", Card::display_rank(r))
            }
            HandType::TwoPair(p1r, p2r) => write!(
                f,
                "Two pair (pair 1 rank {}, pair 2 rank {})",
                Card::display_rank(p1r),
                Card::display_rank(p2r)
            ),
            HandType::OnePair(r) => write!(f, "One pair (rank {})", Card::display_rank(r)),
            HandType::HighCard(r) => write!(f, "High card (rank {})", Card::display_rank(r)),
        }
    }
}
