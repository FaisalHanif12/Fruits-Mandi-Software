// Development page /dev/grid (plan 04): proves the shared entry grid with the keys of PRD 7.2
// in both languages. Sale-line style columns: buyer, qty, rate, "different rate" (y or n), buyer
// rate (open only when "different rate" is y), a note (editable, not an Enter stop) and one
// calculated cell. The calculated cell only joins the typed text, it works out no money. Never
// shipped: the route exists only when import.meta.env.DEV is true (app/router.tsx).

import { Localized, useLocalization } from '@fluent/react';
import { Link } from '@tanstack/react-router';
import { useMemo } from 'react';
import { EntryGrid, createEntryStore, type EntryColumn } from '../../shared/grid';
import { useLanguage } from '../../shared/i18n';
import { SHORTCUTS, useEscapeDispatcher } from '../../shared/keyboard';
import { Button } from '../../shared/ui';
import { LanguageSwitch } from '../layout/LanguageSwitch';
import { GalleryLocalization } from './GalleryLocalization';

const DIFFERENT_RATE = 'y';

/** One store for the page. It outlives the grid, so a language switch loses nothing (F39). */
export const gridPageStore = createEntryStore();

function Content() {
  const { l10n } = useLocalization();
  useEscapeDispatcher();
  const columns = useMemo<readonly EntryColumn[]>(
    () => [
      { field: 'buyer', type: 'account', headerId: 'gridpage-col-buyer' },
      { field: 'qty', type: 'qty', headerId: 'gridpage-col-qty' },
      {
        field: 'rate',
        type: 'rate',
        headerId: 'gridpage-col-rate',
        placeholder: () => l10n.getString('gridpage-rate-hint'),
      },
      { field: 'different', type: 'text', headerId: 'gridpage-col-different', enterStop: false },
      {
        field: 'buyerRate',
        type: 'money',
        headerId: 'gridpage-col-buyer-rate',
        editable: (row) => row.typed['different'] === DIFFERENT_RATE,
      },
      { field: 'note', type: 'text', headerId: 'gridpage-col-note', enterStop: false },
      { field: 'summary', type: 'calc', headerId: 'gridpage-col-summary' },
    ],
    [l10n],
  );
  const rows = gridPageStore((state) => state.rows);
  const calc = useMemo(
    () =>
      Object.fromEntries(
        rows.map((row) => [
          row.rowId,
          { summary: [row.typed['qty'], row.typed['rate']].filter(Boolean).join(' @ ') },
        ]),
      ),
    [rows],
  );
  return (
    <main className="mx-auto flex w-full max-w-page flex-col gap-5 p-5">
      <header className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex flex-col gap-1">
          <Localized id="gallery-back-to-start">
            <Link to="/" className="type-body text-primary underline" />
          </Localized>
          <Localized id="gridpage-title">
            <h1 className="type-title" />
          </Localized>
          <Localized id="gridpage-note">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        <LanguageSwitch />
      </header>
      <EntryGrid
        columns={columns}
        store={gridPageStore}
        labelId="gridpage-grid-label"
        calc={calc}
        copyAbove
        heightMode="auto"
      />
      <div>
        <Button
          shortcut={SHORTCUTS.newEntry}
          onClick={() => {
            gridPageStore.getState().addRow();
          }}
        >
          <Localized id="gridpage-add-row" />
        </Button>
      </div>
    </main>
  );
}

export function GridPage() {
  const language = useLanguage();
  return (
    <GalleryLocalization language={language}>
      <Content />
    </GalleryLocalization>
  );
}
