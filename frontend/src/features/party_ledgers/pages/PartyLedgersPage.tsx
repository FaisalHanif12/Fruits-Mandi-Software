// پارٹی کھاتے (route /bikri/party-ledgers; PRD FR-12.43, FR-12.47, plans 66 and 74): each buyer's and each
// seller's own ledger. Level 1 is the list of parties of one kind with their balances; Enter on a party
// opens level 2, his statement: one row for each date with an entry, 30 days at a time, and Enter or a
// click on a date opens that day (level 3, the R35 slip) under its own row. The full-page slip of a date
// (`?date=`) is still a valid address, so another screen can link to it. The level is in the address
// (`?side=`, `?party=`, `?end=` for the day the 30 days end on, `?date=`), so Esc and the back buttons
// are plain steps back, a language switch keeps the level, and another screen can open a party's
// ledger. The screen only reads: it changes nothing and works out no figure (CLAUDE.md rule 1).

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
        end={search.end}
        onBack={(kind) => {
          go({ side: kind ?? side });
        }}
        onWindow={(end) => {
          // A window is not a step back: Esc goes to the list, and the address only remembers where it is.
          void navigate({
            to: '.',
            search: { side, party, ...(end === null ? {} : { end }) },
            replace: true,
          });
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
