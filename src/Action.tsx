import * as React from 'react';
import { invoke } from '@tauri-apps/api/tauri'
import { NumRange } from './App';

export enum ActionState {
  Default, Betting, Raising
}
interface Raise {
  Raise: number
}
interface Bet {
  Bet: number
}
export interface IActionProps {
  possible_actions: (string | Raise)[]
  on_call: () => void
  on_check: () => void
  on_bet: (amount: number) => void
  on_raise: (amount: number) => void
  raise_or_bet_range: NumRange
  call_amount: number
}

function isRaise(obj: any): obj is Raise {
  return "Raise" in obj;
}
function isBet(obj: any): obj is Bet {
  return "Bet" in obj;
}

export default function Action(props: IActionProps) {
  let [current_state, set_current_state] = React.useState(ActionState.Default);
  let [slider_value, set_slider_value] = React.useState(-1);

  // update slider everytime range change
  React.useEffect(
    () => {
      if (props.raise_or_bet_range !== null) {
        set_slider_value(props.raise_or_bet_range.start)
      }
    },
    [props.raise_or_bet_range]
  )
  // handle ESC
  React.useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        set_current_state(ActionState.Default);
      }
    }
    window.addEventListener('keydown', handleKeyDown);
    // cleanup
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    }
  }, []);


  const raise_or_bet_render = () => {
    let text: string;
    let f: () => void;
    switch (current_state) {
      case ActionState.Default: return (<></>);
      case ActionState.Betting: {
        text = "Bet";
        f = () => props.on_raise(slider_value);
        break;
      }
      case ActionState.Raising: {
        text = "Raise";
        f = () => props.on_bet(slider_value);
        break;
      }
    }
    return (
      <div className='raiseActions'>
        <div className='slider_container'>
          <span>{text} amount: {slider_value}</span>
          <input
            type='range'
            value={slider_value}
            min={props.raise_or_bet_range.start}
            max={props.raise_or_bet_range.end}
            onChange={(e) => set_slider_value(Number(e.target.value))}
          ></input>
        </div>
        <button
          className='green'
          onClick={() => {
            f();
            set_current_state(ActionState.Default);
          }}
        >
          {text}
        </button>
        <button
          className='white'
          onClick={() => set_current_state(ActionState.Default)}
        >
          Cancel
        </button>
      </div >
    )
  }

  switch (current_state) {
    case ActionState.Raising: return raise_or_bet_render();

    case ActionState.Betting: return raise_or_bet_render();

    case ActionState.Default: break;
  }

  // Default state, render buttons

  let disable_call = true;
  // bet and raise are mutually exclusive
  let disable_bet = true;
  let disable_raise = true;

  let disable_check = true;
  let disable_fold = true;

  // disable illegal actions
  for (let e of props.possible_actions) {
    if (typeof e === "string") {
      if (e === "Call") {
        disable_call = false;
      } else if (e === "Check") {
        disable_check = false;
      } else if (e === "Fold") {
        disable_fold = false;
      }
    } else if (typeof e === "object") {
      if (isBet(e)) {
        disable_bet = false;
      } else if (isRaise(e)) {
        disable_raise = false;
      }
    }
  }

  // raise/bet button
  let raise = () => { };
  let bet = () => { };
  let text_suffix = "";
  if (props.raise_or_bet_range !== null) {
    if (props.raise_or_bet_range.start === props.raise_or_bet_range.end) {
      raise = () => props.on_raise(props.raise_or_bet_range.start);
      bet = () => { };
      text_suffix = " " + props.raise_or_bet_range.start;
    } else {
      raise = () => set_current_state(ActionState.Raising);
      bet = () => set_current_state(ActionState.Betting);
    }
  }
  const bet_button = disable_bet
    ? <button
      className='green'
      disabled={disable_raise}
      onClick={raise}
    >
      {"Raise" + text_suffix}
    </button>
    : <button
      className='green'
      onClick={bet}
    >
      {"Bet" + text_suffix}
    </button>;
  // render buttons
  return (
    <div className='actions'>
      <button
        className='green'
        disabled={disable_call}
        onClick={props.on_call}
      >
        {"Call" + (props.call_amount <= 0 ? "" : " " + props.call_amount)}
      </button>
      {bet_button}
      <button
        className='green'
        disabled={disable_check}
        onClick={props.on_check}
      >
        Check
      </button>
      <button className='red' disabled={disable_fold}>Fold</button>
    </div >
  );
}