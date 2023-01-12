use std::cmp::Ordering;

use super::Hand;

type Ranking = u8;

#[derive(Clone, Copy, Eq)]
pub enum HandType {
    Flush,
    Straight(u8), // u8 is the rank of highest card in the straight
    OnePair(u8),  // u8 is the rank of highest pair (usually the only one)
    HighCard,
}
impl HandType {
    pub fn get_hand(hand: Hand) -> HandType {
        // ====flush====
        // cards' suits are the same with the first one
        if hand.into_iter().all(|card| card.suit == hand[0].suit) {
            return HandType::Flush;
        }
        // ====straight====
        // if hand has a 1, there're two cases
        if hand[4].rank == 1 {
            // check for 12345 and TJQKA straight
            let a = hand.get_ranks_array();
            if a == [2, 3, 4, 5, 1] {
                return HandType::Straight(5);
            }
            if a == [10, 11, 12, 13, 1] {
                return HandType::Straight(1);
            }
        }
        // check if the ranks are consecutive
        if (1..5).all(|i| hand[i].rank == hand[i - 1].rank + 1) {
            return HandType::Straight(hand[4].rank); // hand is straight
        }

        // ====one pair====
        // since hand is sorted, only compare to the neighboring card
        for i in (1..5).rev() {
            if hand[i].rank == hand[i - 1].rank {
                return HandType::OnePair(hand[i].rank);
            }
        }

        // ====high card====
        // reached the end, meaning it's not any other type
        HandType::HighCard
    }
    pub fn get_ranking(self) -> Ranking {
        match self {
            HandType::Flush => 5,
            HandType::Straight(_) => 6,
            HandType::OnePair(_) => 9,
            HandType::HighCard => 10,
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
