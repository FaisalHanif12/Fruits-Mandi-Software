// The party card of the بکری screen follows the picker cell that is open (PRD FR-7.38): the record
// the list highlights, else the chosen one. Where the card is for a چکتا buyer, he is told by his
// names and phone as well as the account id, because all of them are on the account چکتا کھاتہ
// (PRD FR-13.6). The setter is stable, so a cell that closes can clear it without a loop.

import { useCallback, useState } from 'react';
import type { CardChukta } from '../../../shared/accounts';

/** Who the card is for while a picker cell is open. */
export interface EditorCard {
  accountId: number;
  chukta: CardChukta | null;
}

export function useEditorCard() {
  const [editor, setState] = useState<EditorCard | null>(null);
  const setEditor = useCallback((accountId: number | null, chukta: CardChukta | null = null) => {
    setState(accountId === null ? null : { accountId, chukta });
  }, []);
  return [editor, setEditor] as const;
}
