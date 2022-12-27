import "./App.css";
import React from "react";
import Community from "./Community";
import { invoke } from '@tauri-apps/api/tauri'
import Actions from "./Actions";
import Player, { IPlayerProps, NULL_PLAYER } from "./Player"
import { ICardProps } from "./Card";

export interface IAppProps {
}

export interface IAppState {
  players: IPlayerProps[],
  possible_actions: string[],
  community: ICardProps[],
}

export default class App extends React.Component<IAppProps, IAppState> {
  constructor(props: IAppProps) {
    super(props);

    this.state = {
      players: [NULL_PLAYER, NULL_PLAYER],
      possible_actions: [],
      community: [],
    }
  }
  componentDidMount(): void {
    invoke('get_new_game', { gameNumber: 0 }).then((game) => { this.updateGame(game as IAppState) });

  }

  public render() {
    return (
      <div className="app">
        <Player {...this.state.players[1]} />
        <Community cards={[]} />
        <Player {...this.state.players[0]} />
        <Actions possible_actions={this.state.possible_actions} on_call={(amount) => this.on_call(amount)} />
      </div>
    );
  }
  updateGame(game: IAppState) {
    invoke('get_possible_actions', { game: game }).then((r) => {
      this.setState({
        possible_actions: r as string[]
      }, () => { console.log(this.state); });
    });
    this.setState(game);
  }
  on_call(amount: number) {
    console.log(this);
    invoke("act", { game: this.state, action: { Call: amount } }).then((game) => {
      this.updateGame(game as IAppState);
    });
  }
}

