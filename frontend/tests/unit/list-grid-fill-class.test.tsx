// A list of more than ten rows is drawn in the 'fill' height mode (PagedList, plan 38 G2). Its box must keep the
// AG Grid theme class as a word of its own beside `list-grid-fill`: once a missing space joined them into one unknown
// class, the grid lost its theme and its height, and every long list (the روکڑ بک of Umar's books, 11 lines) was an
// empty strip (plan 96 T2). jsdom has no layout, so the classes are what a test can read.

import { describe, expect, it } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { ListGrid, type ListColumn } from '../../src/shared/grid';
import { listHeightMode } from '../../src/shared/grid/ListRegion';
import { GRID_THEME_CLASS } from '../../src/shared/grid/listGridClass';
import { useLanguageStore } from '../../src/shared/i18n';
import { renderIn } from '../helpers/render';

interface Line {
  id: string;
  name: string;
  amount: string;
}

const COLUMNS: readonly ListColumn<Line>[] = [
  { field: 'name', type: 'name', headerId: 'report-col-detail', value: (row) => row.name },
  { field: 'amount', type: 'money', headerId: 'report-col-balance', value: (row) => row.amount },
];

const rowsOf = (count: number): Line[] =>
  Array.from({ length: count }, (_, index) => ({
    id: String(index),
    name: `Line ${String(index + 1)}`,
    amount: '100',
  }));

function draw(language: Language, count: number) {
  useLanguageStore.setState({ language });
  return renderIn(
    language,
    <ListGrid<Line>
      columns={COLUMNS}
      rows={rowsOf(count)}
      getRowId={(row) => row.id}
      labelId="report-grid-label"
      onOpen={() => undefined}
      heightMode={listHeightMode(count)}
    />,
  );
}

const box = () => document.querySelector('.entry-grid');

describe.each(LANGUAGES)('the box of a list grid (%s)', (language) => {
  it('a list of 11 rows fills the room and keeps the AG Grid theme as its own class', () => {
    draw(language, 11);
    expect(box()).toHaveClass(GRID_THEME_CLASS);
    expect(box()).toHaveClass('list-grid-fill');
  });

  it('a short list keeps the theme too and is as tall as its rows', () => {
    draw(language, 4);
    expect(box()).toHaveClass(GRID_THEME_CLASS);
    expect(box()).not.toHaveClass('list-grid-fill');
  });
});
