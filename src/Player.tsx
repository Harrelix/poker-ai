import * as React from 'react';
import Card, { ICardProps, NULL_CARD } from './Card';


export interface IPlayerProps {
    name: string;
    hole: [ICardProps, ICardProps];
    bet_size: number;
    stack: number;
}

export const NULL_PLAYER: IPlayerProps = { name: "", hole: [NULL_CARD, NULL_CARD], bet_size: 0, stack: 0 }

export default function Player(props: IPlayerProps) {

    return (
        <div className='player'>
            <div className='playerInfo'>
                <span>{props.name}</span>
                <span>{"Bet: " + props.bet_size}</span>
                <span>{"Stack: " + props.stack}</span>

            </div>
            <div className='cardList' >
                <Card {...props.hole[0]} />
                <Card {...props.hole[1]} />
            </div>
        </div>
    )

}
