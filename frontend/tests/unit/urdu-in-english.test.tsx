// Urdu text inside the English app, and the size of what is clicked (D1+D2 final recheck, N1 and
// N2). NFR-USE-20: Urdu text at least 18 px. NFR-URD-11: line height at least 1.8. NFR-USE-23:
// click targets at least 32 px. Real pixel sizes are measured in a browser; these tests check what
// can be checked without layout: the lang of every Urdu name, the rule that gives such a name the
// Urdu tokens, and the token values.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { StatusBar } from '../../src/app/layout/StatusBar';
import { UserMenu } from '../../src/app/layout/UserMenu';
import { LANGUAGES } from '../../src/i18n';
import { scriptOf } from '../../src/shared/text/scriptHint';
import { LinkButton, ScriptText } from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';
import { URDU_USER_NAME, USER_NAME } from '../helpers/sample';

const URDU_NAME = URDU_USER_NAME;
const LATIN_NAME = USER_NAME;

const styles = (file: string) =>
  readFileSync(path.join(__dirname, '../../src/styles', file), 'utf8');
const tokenNumber = (name: string) =>
  Number(new RegExp(`${name}:\\s*([0-9.]+)`).exec(styles('tokens.css'))?.[1]);

describe('scriptOf', () => {
  it.each([
    [URDU_NAME, 'ur'],
    [LATIN_NAME, 'en'],
    ['', 'en'],
    ['12 - 3', 'en'],
    [`${LATIN_NAME} ${URDU_NAME}`, 'ur'],
  ] as const)('%s is %s', (text, script) => {
    expect(scriptOf(text)).toBe(script);
  });
});

describe('the rule for Urdu inside English text', () => {
  const css = styles('globals.css');
  const base = /@layer base \{([\s\S]*?)\n\}\n/.exec(css)?.[1] ?? '';

  it('gives a lang="ur" element inside English text the Urdu font, size and line height', () => {
    // The tokens are read on the Urdu element itself, so they are the Urdu ones there.
    expect(base).toMatch(/:lang\(en\) \[lang='ur'\] \{[^}]*font-family: var\(--font-text\)/);
    expect(base).toMatch(/:lang\(en\) \[lang='ur'\] \{[^}]*font-size: var\(--text-body\)/);
    expect(base).toMatch(/:lang\(en\) \[lang='ur'\] \{[^}]*line-height: var\(--leading-text\)/);
  });

  it('is in the base layer, so an element with its own type class keeps its size', () => {
    expect(css).toContain('@layer base');
    // It never reaches the Urdu app: only an Urdu element below English text is matched.
    expect(base).not.toMatch(/^\s*:lang\(ur\)/m);
  });

  it('uses Urdu tokens that meet the NFR: 18 px at the least, line height 1.8 at the least', () => {
    expect(tokenNumber('--text-urdu-1')).toBeGreaterThanOrEqual(18);
    expect(tokenNumber('--leading-urdu')).toBeGreaterThanOrEqual(1.8);
  });
});

describe('click targets', () => {
  it('the smallest target token is 32 px (NFR-USE-23)', () => {
    expect(tokenNumber('--target-min')).toBeGreaterThanOrEqual(32);
  });

  it('a link button takes the target height', () => {
    renderIn('en', <LinkButton onClick={vi.fn()}>{LATIN_NAME}</LinkButton>);
    expect(screen.getByRole('button', { name: LATIN_NAME }).className).toContain('min-h-target');
  });

  it('the day state in the status bar takes the target height', () => {
    renderIn(
      'en',
      <StatusBar userName={LATIN_NAME} date="10-10-2026" day="open" onOpenDay={vi.fn()} />,
    );
    const button = screen.getByRole('button', { name: message('en', 'layout-status-day-open') });
    expect(button.className).toContain('min-h-target');
  });
});

describe.each(LANGUAGES)('an owner with an Urdu name in the %s app', (language) => {
  it('marks the name and its first letter in the user menu as Urdu', () => {
    renderIn(
      language,
      <UserMenu
        userName={URDU_NAME}
        onChangePassword={vi.fn()}
        onGuide={vi.fn()}
        onLock={vi.fn()}
        onSignOut={vi.fn()}
      />,
    );
    const button = screen.getByRole('button', { name: message(language, 'layout-user-menu') });
    const marked = Array.from(button.querySelectorAll('span[lang]'));
    expect(marked.map((item) => item.getAttribute('lang'))).toEqual(['ur', 'ur']);
    // The first letter has the normal line height of its text, not the chip height of 1.
    expect(marked[0]?.className).not.toContain('leading-chip');
  });

  it('marks a Latin name as Latin, and leaves it as it was typed', () => {
    renderIn(
      language,
      <UserMenu
        userName={LATIN_NAME}
        onChangePassword={vi.fn()}
        onGuide={vi.fn()}
        onLock={vi.fn()}
        onSignOut={vi.fn()}
      />,
    );
    const button = screen.getByRole('button', { name: message(language, 'layout-user-menu') });
    const marked = Array.from(button.querySelectorAll('span[lang]'));
    expect(marked.map((item) => item.getAttribute('lang'))).toEqual(['en', 'en']);
    expect(marked[1]?.textContent).toBe(LATIN_NAME);
  });

  it('marks the name in the status bar as Urdu, inside the sentence that holds it', () => {
    renderIn(language, <StatusBar userName={URDU_NAME} date="10-10-2026" day="open" />);
    const name = document.querySelector('footer bdi[lang]');
    expect(name?.getAttribute('lang')).toBe('ur');
    expect(visible(name?.textContent)).toBe(URDU_NAME);
  });

  it('ScriptText takes the lang from the letters it holds', () => {
    renderIn(
      language,
      <p>
        <ScriptText>{URDU_NAME}</ScriptText>
        <ScriptText>{LATIN_NAME}</ScriptText>
      </p>,
    );
    const items = Array.from(document.querySelectorAll('p bdi'));
    expect(items.map((item) => item.getAttribute('lang'))).toEqual(['ur', 'en']);
  });
});
