import "./App.css";
import React from "react";
import Community from "./Community";
import { invoke } from '@tauri-apps/api/tauri'
import Action from "./Action";
import Player, { IPlayerProps, NULL_PLAYER } from "./Player"
import { ICardProps } from "./Card";

export interface IAppProps {
}

export interface Game {
  players: IPlayerProps[],
  community: ICardProps[],
  pot_size: number,
}
export interface NumRange {
  start: number,
  end: number,
}
export interface IAppState {
  game: Game,
  possible_actions: string[],
  call_amount: number,
  raise_or_bet_range: NumRange,
}


export default class App extends React.Component<IAppProps, IAppState> {
  constructor(props: IAppProps) {
    super(props);

    this.state = {
      game: {
        players: [NULL_PLAYER, NULL_PLAYER],
        community: [],
        pot_size: 0,
      },
      possible_actions: [],
      call_amount: 0,
      raise_or_bet_range: {
        start: -1, end: -1
      }
    }
  }
  componentDidMount(): void {
    invoke('get_new_game').then((game) => this.updateGame(game as Game));
  }

  public render() {
    return (
      <div className="app">
        <Player {...this.state.game.players[1]} />
        <Community
          cards={this.state.game.community}
          pot={this.state.game.pot_size}
          total={this.state.game.pot_size +
            this.state.game.players.reduce((acc, player) => acc + player.bet_size, 0)
          }
        />
        <Player {...this.state.game.players[0]} />
        <Action
          possible_actions={this.state.possible_actions}
          on_call={() => this.on_call()}
          on_check={() => this.on_check()}
          on_bet={(amount) => this.on_bet(amount)}
          on_raise={(amount) => this.on_raise(amount)}
          call_amount={this.state.call_amount}
          raise_or_bet_range={this.state.raise_or_bet_range}
        />
      </div>
    );
  }

  updateGame(game: Game) {
    invoke('get_possible_actions', { game: game }).then(
      (possible_actions) => this.setState({ possible_actions: possible_actions as string[] })
    );
    invoke('get_call_amount', { game: game }).then(
      (call_amount) => this.setState({ call_amount: call_amount as number })
    );
    invoke('get_raise_or_bet_range', { game: game }).then(
      (raise_range) => {
        console.log(raise_range);
        this.setState({ raise_or_bet_range: raise_range as NumRange });
      }
    );

    this.setState({ game: game });
  }

  on_call() {
    invoke("act", { game: this.state.game, action: "Call" }).then((game) =>
      this.updateGame(game as Game)
    )
  }
  on_check() {
    invoke("act", { game: this.state.game, action: "Check" }).then((game) =>
      this.updateGame(game as Game)
    )
  }
  on_bet(amount: number) {
    invoke("act", { game: this.state.game, action: { Bet: amount } }).then((game) =>
      this.updateGame(game as Game)
    )
  }
  on_raise(amount: number) {
    invoke("act", { game: this.state.game, action: { Raise: amount } }).then((game) => {
      this.updateGame(game as Game)
    })
  }
}