import * as React from 'react';
import Card, { ICardProps } from './Card'

export interface ICommunityProps {
  cards: ICardProps[];
}

export default function Community(props: ICommunityProps) {

  return (
    <div className='cardList community'>
      {props.cards.map((card, index) => (
        <Card key={index} {...card} />
      ))}
    </div>
  );

}

