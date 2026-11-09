// پارٹی کھاتے (route /bikri/party-ledgers; PRD FR-12.43, plan 66): each buyer's and each seller's own
// ledger in three levels. Level 1 is the list of parties of one kind with their balances; Enter on a
// party opens level 2, his statement, one row for each date with an entry; Enter on a date opens
// level 3, that day's full slip, which prints. The level is in the address (`?side=`, `?party=`,
// `?date=`), so Esc and the back buttons are plain steps back, a language switch keeps the level,
// and another screen can open a party's ledger. The screen only reads: it changes nothing and works
// out no figure (CLAUDE.md rule 1).

import { useNavigate, useSearch } from '@tanstack/react-router';
import { useCallback } from 'react';
import type { LedgerSide } from '../../../bindings/LedgerSide';
import { PartyDay } from '../components/PartyDay';
import { PartyList } from '../components/PartyList';
import { PartyStatement } from '../components/PartyStatement';
import { parseLedgerSearch, type LedgerSearch } from '../search';

export function PartyLedgersPage() {
  const navigate = useNavigate();
  const search = parseLedgerSearch(useSearch({ strict: false }));
  const side: LedgerSide = search.side ?? 'buyer';
  const go = useCallback(
    (next: LedgerSearch) => {
      void navigate({ to: '.', search: next });
    },
    [navigate],
  );

  if (search.party === undefined) {
    return (
      <PartyList
        side={side}
        onSide={(next) => {
          go({ side: next });
        }}
        onOpen={(row) => {
          go({ side, party: row.account_id });
        }}
      />
    );
  }
  const party = search.party;
  // The kind the backend says the party is wins over the address, so a link that left the side out
  // still goes back to the right list.
  if (search.date === undefined) {
    return (
      <PartyStatement
        key={party}
        accountId={party}
        onBack={(kind) => {
          go({ side: kind ?? side });
        }}
        onOpenDay={(date) => {
          go({ side, party, date });
        }}
      />
    );
  }
  const date = search.date;
  return (
    <PartyDay
      key={`${String(party)}-${date}`}
      accountId={party}
      date={date}
      onBack={(kind) => {
        go({ side: kind ?? side, party });
      }}
    />
  );
}
