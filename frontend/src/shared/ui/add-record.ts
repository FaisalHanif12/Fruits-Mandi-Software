// "Add it here" from a picker (plan 41, PRD FR-7.69): a missing record is added from the list where
// it is needed, with a short form, and is chosen in the field at once. The user never leaves his
// entry.
//
// Every entry picker ends its open list with one row, "+ New expense head". A click on it, `↓`
// then `Enter`, or `F4` in the field asks the app for the short form of that kind. The form opens
// over the screen, so a sale with its lines, a voucher or a grid keeps every value. When it
// closes, the picker takes the focus back. If a record was saved, the picker puts it in the field;
// if not (Esc), the field is as it was.
//
// The short forms live in the masters module, which `shared` must not import. So the app gives
// this context its dialogs (app/AddRecordHost.tsx), and a picker only asks for them.

import { createContext, useContext } from 'react';
import type { SearchScope } from '../search';
import type { PickerValue } from './PickerBase';

/**
 * The master lists a picker can add to: the nine masters of the Index. A trip and a garden are
 * found by a picker too (plan 34: the open ones for an entry, `trip_all` and `garden_all` for a
 * filter that also wants the closed ones) but are made on their own screens, never from a picker.
 */
export type MasterAddKind = Exclude<
  SearchScope,
  'seller' | 'buyer' | 'any_account' | 'trip' | 'garden' | 'trip_all' | 'garden_all'
>;

/** The scopes of the trips and gardens: found by a picker, never added from one. */
export type VentureScope = 'trip' | 'garden' | 'trip_all' | 'garden_all';

/**
 * What the form is for: a new seller, a new buyer, a new account of a type the user chooses in
 * the form (a field that takes any account), or a new record of a master list.
 */
export type AddRecordKind = 'seller' | 'buyer' | 'any' | MasterAddKind;

/**
 * The record that was saved, as much of it as a picker needs to find its own row for it (a master
 * hit or an account hit). The picker asks its own search for the row, so the name, the second
 * line and the "sold by weight" mark are exactly what the list shows.
 */
export interface AddedRecord {
  id: number;
  code: string;
}

export interface AddRecordRequest {
  kind: AddRecordKind;
  /** What was typed in the picker. It starts the name (or the mark text, for a مارکہ). */
  text: string;
  /**
   * `quick` (default): the short form. `full`: the full Index form, for fields the short form does
   * not ask (opening balance, credit limit and others).
   */
  form?: 'quick' | 'full' | undefined;
  /** A مارکہ added from a lot: the lot's seller is its owner. */
  owner?: PickerValue | null | undefined;
  /** The form closed: with the record that was saved, or null if the user left without saving. */
  onClosed: (added: AddedRecord | null) => void;
}

/** Opens a form for a request. Null where no dialog is given (a small test). */
export type AddRecord = (request: AddRecordRequest) => void;

export const AddRecordContext = createContext<AddRecord | null>(null);

/** The way to open the short form from a picker, or null if this part of the app has none. */
export function useAddRecord(): AddRecord | null {
  return useContext(AddRecordContext);
}
