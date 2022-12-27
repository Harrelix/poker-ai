import * as React from 'react';
import { invoke } from '@tauri-apps/api/tauri'


export interface IActionsProps {
    possible_actions: (string | Call | Raise)[];
    on_call: (amount: number) => void;
}
interface Call {
    Call: number
}
function isCall(obj: any): obj is Call {
    return 'Call' in obj;
}
interface Raise {
    Raise: number
}
function isRaise(obj: any): obj is Raise {
    return "Raise" in obj;
}
interface Bet {
    Bet: number
}
function isBet(obj: any): obj is Bet {
    return "Bet" in obj;
}

export default function Actions(props: IActionsProps) {
    let disable_call = true; let call_amount = -1;
    // bet and raise are mutually exclusive
    let disable_bet = true;
    let disable_raise = true;
    // check and fold are mutually exclusive
    let disable_check = true;
    let disable_fold = true;

    for (let e of props.possible_actions) {
        if (typeof e === "string") {
            if (e === "Check") {
                disable_check = false;
            } else if (e === "Fold") {
                disable_fold = false;
            }
        } else if (typeof e === "object") {
            if (isCall(e)) {
                disable_call = false;
                call_amount = e.Call;
            } else if (isBet(e)) {
                disable_bet = false;
            } else if (isRaise(e)) {
                disable_raise = false;
            }
        }
    }

    return (
        <div className='actions'>
            <button className='green' disabled={disable_call} onClick={() => props.on_call(call_amount)}>
                {"Call" + (call_amount <= 0 ? "" : call_amount)}
            </button>
            {(disable_bet)
                ? <button className='green' disabled={disable_raise}> Raise </button>
                : <button className='green'> Bet </button>
            }

            <button className='green' disabled={disable_check}> Check </button>
            <button className='red' disabled={disable_fold}> Fold </button>
        </div>
    );
}
