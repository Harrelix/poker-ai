import * as React from 'react';
import Card, { ICardProps } from './Card'

export interface ICommunityProps {
  cards: ICardProps[];
  pot: number,
  total: number,
}

export default function Community(props: ICommunityProps) {

  return (
    <div className='community'>
      <span>
        POT SIZE: {props.pot}
        {props.total === props.pot
          ? ""
          : " (TOTAL: " + props.total + ")"}
      </span>
      <div className='cardList'>
        {props.cards.map((card, index) => (
          <Card key={index} {...card} />
        ))}
      </div>
    </div>

  );

}