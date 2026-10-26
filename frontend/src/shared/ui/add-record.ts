// "Add a new one" from a picker that found nothing (plan 32, R8).
//
// A picker with an empty list shows a button, for example "Add a new Seller". It opens the Index
// form of that kind in a dialog above the screen, with what was typed already in the name box.
// The screen under the dialog never closes, so an entry in progress (a sale with its lines, a
// voucher) keeps every value. When the form closes, saved or not, the picker takes the focus back,
// shows the same typed text and opens its list, so the new record is there to be picked. The
// picker never fills the new record in by itself.
//
// The Index forms live in the masters module, which `shared` must not import. So the app gives
// this context its dialog (app/AddRecordHost.tsx), and a picker only asks for it.

import { createContext, useContext } from 'react';
import type { SearchScope } from '../search';

/** The master lists a picker can add to: the nine masters of the Index. */
export type MasterAddKind = Exclude<SearchScope, 'seller' | 'buyer' | 'any_account'>;

/**
 * What the Index form is for: a new seller, a new buyer, a new account of a type the user chooses
 * in the form (a field that takes any account), or a new record of a master list.
 */
export type AddRecordKind = 'seller' | 'buyer' | 'any' | MasterAddKind;

export interface AddRecordRequest {
  kind: AddRecordKind;
  /** What was typed in the picker. It starts the name (or the mark, for a مارکہ). */
  text: string;
  /** The form closed, saved or not. */
  onClosed: (saved: boolean) => void;
}

/** Opens the Index form for a request. Null where no dialog is given (a small test, a nested form). */
export type AddRecord = (request: AddRecordRequest) => void;

export const AddRecordContext = createContext<AddRecord | null>(null);

/** The way to open the Index form from a picker, or null if this part of the app has none. */
export function useAddRecord(): AddRecord | null {
  return useContext(AddRecordContext);
}
