use std::iter;

use super::Hand;
pub fn get_all_hand_types() -> impl Iterator<Item = Box<dyn HandType>> {
    const ALL_HAND_TYPES: [Box<dyn HandType>; 2] = [Box::new(Flush), Box::new(Straight)];
    ALL_HAND_TYPES.into_iter()
}
type Ranking = u8;
pub trait HandType {
    fn check(self, hand: Hand) -> bool;
    fn get_ranking(self) -> Ranking;
}
struct Flush;
impl HandType for Flush {
    fn check(self, hand: Hand) -> bool {
        // check if hand is a flush
        // cards' suits are the same with the first one
        hand.iter().all(|card| card.suit == hand[0].suit)
    }

    fn get_ranking(self) -> Ranking {
        5
    }
}

struct Straight;
impl HandType for Straight {
    fn check(self, hand: Hand) -> bool {
        // check if hand is straight
        if hand[4].rank == 1 {
            // check for 12345 and TJQKA straight
            let a = hand.get_ranks_array();
            if a == [2, 3, 4, 5, 1] || a == [10, 11, 12, 13, 1] {
                return true;
            }
            return false;
        }
        // check if the ranks aren't consecutive
        for i in 1..5 {
            if hand[i].rank != hand[i - 1].rank + 1 {
                return false;
            }
        }
        // hand is straight
        true
    }
    fn get_ranking(self) -> Ranking {
        6
    }
}
struct OnePair;
impl HandType for OnePair {
    fn check(self, hand: Hand) -> bool {
        // checks if hand has one pair
        // since hand is sorted, only compare to the next card
        for i in 0..4 {
            if hand[i].rank == hand[i + 1].rank {
                return true;
            }
        }
        // no pairs
        false
    }

    fn get_ranking(self) -> Ranking {
        9
    }
}
struct HighCard;
impl HandType for HighCard {
    fn check(self, hand: Hand) -> bool {
        // hand is high card if there's no flush, no straight and no pair
        !(hand.check(Flush) || hand.check(Straight) || hand.check(OnePair))
    }

    fn get_ranking(self) -> Ranking {
        10
    }
}
