// The help button for every main screen (PRD FR-7.89, plan 52 item 4). The backend names 19 main
// screens in `EntryScreen`; each one must open its own route, empty, with no party. The routes in
// the table are read against two things that can drift: the doc comment of the generated type (the
// backend's own list) and the real router.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AskReply } from '../../src/bindings/AskReply';
import type { EntryScreen } from '../../src/bindings/EntryScreen';
import * as assistantApi from '../../src/features/assistant/api';
import {
  isMainScreen,
  MAIN_SCREEN_PATHS,
  type MainScreen,
} from '../../src/features/assistant/navigation';
import { useAssistantStore } from '../../src/features/assistant/store';
import { press } from '../helpers/app';
import * as real from '../helpers/assistant-real';
import {
  boxOf,
  panelOf,
  renderShellRouted,
  resetAssistantFakes,
  routeNow,
  setLanguage,
} from '../helpers/assistant-shell';
import { message } from '../helpers/render';

vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/assistant/api');
vi.mock('../../src/features/partners/api');

beforeEach(() => {
  resetAssistantFakes();
  setLanguage('en');
});

afterEach(() => {
  useAssistantStore.getState().reset();
});

const SRC = path.resolve(__dirname, '../../src');
const SCREENS = Object.keys(MAIN_SCREEN_PATHS) as MainScreen[];

describe('the table of main screens', () => {
  it('has the 19 main screens the backend names, and no entry screen', () => {
    const generated = readFileSync(path.join(SRC, 'bindings/EntryScreen.ts'), 'utf8');
    const union = [...generated.matchAll(/\|\s*'(\w+)'/g)].map((m) => m[1] as EntryScreen);
    // 19 main screens and 21 entry screens: the ten of plan 55 Section 11.1 (a garden's and a trip's
    // entries, a later sale, edit and cancel of a بکری) came after the first thirty.
    expect(union.length).toBe(40);
    const main = union.filter(isMainScreen);
    expect(main.sort()).toEqual([...SCREENS].sort());
    expect(main).toHaveLength(19);
    for (const entry of [
      'bikri',
      'wasooli',
      'kharcha',
      'account',
      'garden_installment',
      'garden_cost',
      'garden_sale',
      'garden_close',
      'trip_purchase',
      'trip_cost',
      'trip_sale',
      'bikri_later_sale',
      'bikri_edit',
      'bikri_cancel',
    ] as const) {
      expect(isMainScreen(entry)).toBe(false);
    }
  });

  it("matches the routes the backend's own doc comment lists", () => {
    const generated = readFileSync(path.join(SRC, 'bindings/EntryScreen.ts'), 'utf8');
    // "`dashboard` `/`, `settings` `/settings`, ..." (the comment wraps lines, so squash the stars).
    const comment = generated.replaceAll(/\n\s*\*/g, ' ');
    const listed = new Map(
      [...comment.matchAll(/`(\w+)`\s+`(\/[^`]*)`/g)].map((m) => [m[1], m[2]] as const),
    );
    for (const screen of SCREENS) {
      expect(listed.get(screen), screen).toBe(MAIN_SCREEN_PATHS[screen]);
    }
  });

  it('points only to routes the app really has', () => {
    const router = readFileSync(path.join(SRC, 'app/router.tsx'), 'utf8');
    const routes = new Set([...router.matchAll(/path: '([^']+)'/g)].map((m) => m[1]));
    for (const screen of SCREENS) {
      expect(routes.has(MAIN_SCREEN_PATHS[screen]), screen).toBe(true);
    }
  });
});

describe('the button of a help answer opens each main screen, empty', () => {
  it.each(SCREENS)('%s', async (screen) => {
    await renderShellRouted();
    const reply: AskReply = { ...real.REPLY_HELP_ROUTE, entry_route: { screen, party: null } };
    vi.mocked(assistantApi.assistantAsk).mockResolvedValue(reply);
    const user = userEvent.setup();
    press('F1');
    await user.type(boxOf('en'), 'how do I open this{Enter}');
    await waitFor(() => {
      expect(useAssistantStore.getState().entries.at(-1)?.working).toBe(false);
    });
    await user.click(
      within(panelOf('en')).getByRole('button', {
        name: message('en', 'assistant-open-screen'),
      }),
    );
    expect(routeNow().path).toBe(MAIN_SCREEN_PATHS[screen]);
    // No party and no search: a main screen opens as it is.
    expect(routeNow().search).toEqual({});
  });
});
