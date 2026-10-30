// The two actions on a person who owes the shop and has died or left the business (PRD FR-10.10,
// plan 40): "ڈوبی رقم میں ڈالیں" (move him) and "دوبارہ فعال کریں" (make him active again).
//
// The party card, the account form and the dues reports show the buttons, but the dialogs and the
// commands belong to the masters module, which `shared` must not import. So the app gives this
// context its two dialogs (app/DeadAmountHost.tsx) and a button only asks for them. Where no host is
// given (a small test, a gallery page) the buttons are simply not there.

import { createContext, useContext } from 'react';

/** The names a dialog says: the screen already has them, so the dialog does not ask again. */
export interface DeadPartyNames {
  name_ur: string;
  name_en: string;
  tag?: string | null | undefined;
}

export interface DeadMoveRequest {
  accountId: number;
  /** The person inside a پختہ کھاتہ account, as stored. Null for any other account. */
  personKey: string | null;
  names: DeadPartyNames;
  /** The dialog closed; `moved` is true when the person was moved. */
  onClosed?: ((moved: boolean) => void) | undefined;
}

export interface DeadUndoRequest {
  markId: number;
  names: DeadPartyNames;
  /** The dialog closed; `undone` is true when the person is active again. */
  onClosed?: ((undone: boolean) => void) | undefined;
}

export interface DeadAmountActions {
  move: (request: DeadMoveRequest) => void;
  undo: (request: DeadUndoRequest) => void;
}

export const DeadAmountContext = createContext<DeadAmountActions | null>(null);

/** The way to open the two dialogs, or null where this part of the app has none. */
export function useDeadAmountActions(): DeadAmountActions | null {
  return useContext(DeadAmountContext);
}
