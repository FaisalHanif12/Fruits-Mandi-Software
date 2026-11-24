// The class list of a list grid's box (plan 96 T2): the AG Grid theme class is always a word of its own, with the
// fill height of a long list beside it, never joined to it.

import { describe, expect, it } from 'vitest';
import {
  GRID_THEME_CLASS,
  LIST_GRID_CLASS,
  listGridClassName,
} from '../../src/shared/grid/listGridClass';

describe('listGridClassName', () => {
  it.each([
    ['fill', false, ['entry-grid', LIST_GRID_CLASS, GRID_THEME_CLASS, 'list-grid-fill']],
    ['auto', false, ['entry-grid', LIST_GRID_CLASS, GRID_THEME_CLASS]],
    [
      'fill',
      true,
      ['entry-grid', LIST_GRID_CLASS, GRID_THEME_CLASS, 'list-grid-fill', 'list-grid-wrap'],
    ],
    ['auto', true, ['entry-grid', LIST_GRID_CLASS, GRID_THEME_CLASS, 'list-grid-wrap']],
  ] as const)('%s height, growing text %s', (heightMode, growing, words) => {
    expect(listGridClassName(heightMode, growing).split(' ')).toEqual(words);
  });

  it('has no empty word and no double space', () => {
    expect(listGridClassName('auto', false)).toBe('entry-grid list-grid ag-theme-quartz');
  });
});
