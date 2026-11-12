// Development page /dev/statement-print (plan 74 T3): one party's statement print with a double of the
// backend's answers (tests/helpers/dev-bridge-statement.ts), so the box and the page can be opened in a plain
// browser tab and photographed in both languages. Without flags it shows the button "اسٹیٹمنٹ پرنٹ" and its
// box. `?window&language=ur|en&paper=a4|a5&data=example|thirty|akram|rahim|empty` shows the page the way the hidden
// print window makes it (with the print style on, "emulate print media", it is the page that is printed).
// Never shipped: the route exists only when import.meta.env.DEV is true (app/router.tsx).

import { useRouterState } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import { installStatementDevBridge } from '../../../tests/helpers/dev-bridge-statement';
import { StatementPrintAction } from '../../features/party_ledgers';
import { StatementPrintView } from '../../features/party_ledgers/print/StatementPrintView';
import { useLanguageStore } from '../../shared/i18n';

/** The party and the period of each stand-in answer; `example`, `akram`, `rahim` and `empty` are the backend's own. */
const CASES = {
  example: { account_id: 20, from: '01-11-2026', to: '06-11-2026' },
  thirty: { account_id: 20, from: '14-10-2026', to: '12-11-2026' },
  akram: { account_id: 20, from: '01-10-2026', to: '05-12-2026' },
  rahim: { account_id: 18, from: '01-09-2026', to: '05-12-2026' },
  empty: { account_id: 20, from: '20-11-2026', to: '30-11-2026' },
} as const;

export function StatementPrintDevPage() {
  // The stand-in is installed before the page asks its first question.
  useState(() => {
    installStatementDevBridge();
    return true;
  });
  // The app starts in English when the address says so (the page has no language switch). Done after the
  // first draw, not while drawing, so nothing else is asked to redraw in the middle of it.
  useEffect(() => {
    if (new URLSearchParams(window.location.hash.split('?')[1] ?? '').get('language') === 'en') {
      useLanguageStore.getState().setLanguage('en');
    }
  }, []);
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  const search = new URLSearchParams(searchStr);
  if (!search.has('window')) {
    return (
      <div className="flex flex-col gap-4 p-6">
        <div>
          <StatementPrintAction accountId={20} />
        </div>
      </div>
    );
  }
  const data = search.get('data');
  const chosen =
    data === 'example' || data === 'akram' || data === 'rahim' || data === 'empty'
      ? CASES[data]
      : CASES.thirty;
  const paper = search.get('paper') === 'a5' ? 'a5' : 'a4';
  return (
    <StatementPrintView
      input={{
        ...chosen,
        language: search.get('language') === 'en' ? 'en' : 'ur',
        paper,
      }}
    />
  );
}
