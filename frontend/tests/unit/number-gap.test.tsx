// The room around a number inside words (plan 72, Umar 12-11-2026). A range such as "03-10-2026 تا
// 09-10-2026" is two dates in their own left-to-right runs with a word between them. Gulzar's space is
// almost nothing next to the wide digits of Inter, so in Urdu the three ran together on screen. The text
// is right (one space each side, in both languages); the room comes from one token, `--number-gap`, that
// is 4 px in Urdu and 0 in English and is read by every running number that has words beside it.
// Where the range is drawn: the divider, the heading of the earlier days and the two week buttons.

import { screen } from '@testing-library/react';
import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { PastDivider } from '../../src/features/bikri/components/PastDivider';
import { PastSectionHeader } from '../../src/features/bikri/components/PastSectionHeader';
import { PastWeekNav } from '../../src/features/bikri/components/PastWeekNav';
import { LANGUAGES, type Language } from '../../src/i18n';
import { paisaForTest } from '../helpers/paisa';
import { message, renderIn, visible } from '../helpers/render';

const STYLES = path.resolve(import.meta.dirname, '../../src/styles');
const TOKENS = fs.readFileSync(path.join(STYLES, 'tokens.css'), 'utf8');
const GLOBALS = fs.readFileSync(path.join(STYLES, 'globals.css'), 'utf8');

const TO_WORD: Record<Language, string> = { ur: 'تا', en: 'to' };

/** The one run of text that holds both dates of the range, as the screen shows it. */
function rangeTexts(root: HTMLElement): string[] {
  return Array.from(root.querySelectorAll('span'))
    .filter((span) => span.querySelectorAll(':scope > bdi[dir="ltr"]').length === 2)
    .map((span) => visible(span.textContent));
}

describe.each(LANGUAGES)('the range of days in %s', (language: Language) => {
  const expected = `03-10-2026 ${TO_WORD[language]} 09-10-2026`;

  it('is the message with one space on each side of the word between the dates', () => {
    expect(message(language, 'bikri-past-range', { from: '03-10-2026', to: '09-10-2026' })).toBe(
      expected,
    );
  });

  it('keeps that one space on each side in the divider', () => {
    renderIn(language, <PastDivider from="2026-10-03" to="2026-10-09" />);
    expect(rangeTexts(document.body)).toEqual([expected]);
  });

  it('keeps that one space on each side in the heading of the earlier days', () => {
    renderIn(
      language,
      <PastSectionHeader
        part="window"
        from="2026-10-03"
        to="2026-10-09"
        count={3}
        gross={paisaForTest(2_940_000)}
      />,
    );
    expect(rangeTexts(document.body)).toEqual([expected]);
  });

  it('keeps that one space on each side under both week buttons', () => {
    renderIn(
      language,
      <PastWeekNav
        end="2026-10-09"
        today="2026-10-10"
        onEarlier={() => undefined}
        onNearer={() => undefined}
        onLatest={() => undefined}
      />,
    );
    const texts = rangeTexts(screen.getByRole('group'));
    // Earlier goes to 26-09 to 02-10; the later button is off on the latest window, so it has none.
    expect(texts).toEqual([`26-09-2026 ${TO_WORD[language]} 02-10-2026`]);
  });
});

describe('the room beside a number inside words', () => {
  /** The declarations of the `:lang(xx) { … }` block of tokens.css that sets the per-script values. */
  const block = (lang: Language): string => {
    const start = TOKENS.indexOf(`:lang(${lang}) {`);
    expect(start).toBeGreaterThan(-1);
    return TOKENS.slice(start, TOKENS.indexOf('\n}', start));
  };

  it('is none in English, where Inter already has a wide space', () => {
    expect(block('en')).toMatch(/--number-gap:\s*0px;/);
  });

  it('is the 4 px token in Urdu, where Gulzar has almost no space', () => {
    expect(block('ur')).toMatch(/--number-gap:\s*var\(--space-1\);/);
    expect(TOKENS).toMatch(/--space-1:\s*4px;/);
  });

  it('is given by every running number that has words beside it, and not by one that stands alone', () => {
    const utility = /@utility number-run \{([\s\S]*?)\n\}/.exec(GLOBALS)?.[1] ?? '';
    expect(utility).toContain('&:not(:only-child)');
    expect(utility).toMatch(/margin-inline:\s*var\(--number-gap\);/);
  });
});
