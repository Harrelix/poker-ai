import * as React from 'react';

export interface ICardProps {
  rank: number;
  suit: string;
}

export const NULL_CARD: ICardProps = { rank: -1, suit: "" };

export default function Card(props: ICardProps) {
  let img: string;
  let color: string;
  switch (props.suit) {
    case "Spade": { img = "./spade.svg"; color = "black"; break; }
    case "Club": { img = "./club.svg"; color = "black"; break; }
    case "Diamond": { img = "./diamond.svg"; color = "red"; break; }
    case "Heart": { img = "./heart.svg"; color = "red"; break; }
    default: {
      return (
        <div className='card cardBack'></div>
      );
    }
  }
  return (
    <div className='card' style={{ color: color }}>
      <div className='cardRank'>{props.rank}</div>
      <img src={img} className='cardSuit' />
    </div>
  );

}