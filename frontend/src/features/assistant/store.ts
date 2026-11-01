// What the question panel keeps above its screens (plan 49, PRD FR-7.73, FR-7.78, Code Standards
// F39): whether it is open and how tall, what is typed in the box, the questions asked and their
// answers. It lives here and not in the panel's components, so moving between screens, switching the
// language (which draws the screens again) or closing the panel loses nothing. It is cleared when
// the Owner starts a new question, the app locks or he signs out (FR-7.73).
//
// It keeps only what the backend sent, as it sent it. Nothing here adds up or builds a figure.

import { create } from 'zustand';
import type { AnswerName } from '../../bindings/AnswerName';
import type { AskReply } from '../../bindings/AskReply';
import type { AssistantStep } from '../../bindings/AssistantStep';
import type { QuickChip } from '../../bindings/QuickChip';
import type { UiError } from '../../bindings/UiError';

/** What the Owner asked: typed words, a quick question, or an old question read again. */
export type Asked =
  | { type: 'text'; text: string }
  | { type: 'chip'; chip: QuickChip; party: AnswerName | null }
  | { type: 'log'; text: string };

/** One question of the conversation, and what came of it. */
export interface DockEntry {
  /** A running number of this panel, never reused. */
  id: number;
  asked: Asked;
  /** True from the moment it is sent until its answer has come (PRD FR-7.73). */
  working: boolean;
  /** The step lines of the backend so far, oldest first (NFR-AI-3). */
  steps: readonly AssistantStep[];
  /** The backend's reply, or null while it is being worked on or when the call itself failed. */
  reply: AskReply | null;
  /** The row of the question log, once a reply has named it: `assistant_pick` takes it. */
  questionId: number | null;
  /** The command's refusal or failure (the assistant is off, a question is running ...). */
  error: UiError | null;
}

interface AssistantStore {
  open: boolean;
  /** The tall height (about two thirds of the window); false is the short one (a third). */
  tall: boolean;
  /** What is typed in the question box. */
  draft: string;
  /** The conversation, oldest first; the newest is at the bottom of the panel. */
  entries: readonly DockEntry[];
  /** The conversation the backend keeps for follow-up questions (PRD FR-7.78), or null for a new one. */
  conversationId: string | null;
  /** The questions asked, oldest first, for `↑` in the box. */
  asked: readonly string[];
  /** Where `↑` and `↓` are in `asked`, or null when the box is not recalling. */
  recall: number | null;
  nextId: number;
  openDock: () => void;
  closeDock: () => void;
  setTall: (tall: boolean) => void;
  setDraft: (draft: string) => void;
  /** Adds a question to the conversation as being worked on and clears the box. Returns its id. */
  startEntry: (asked: Asked) => number;
  /** A step line of the question being worked on. */
  addStep: (step: AssistantStep) => void;
  /** The backend's reply has come. */
  setReply: (id: number, reply: AskReply) => void;
  /** The call failed or was refused. */
  setError: (id: number, error: UiError) => void;
  /** A question that waited goes on (after a choice or the partner code): it is being worked on again. */
  resume: (id: number) => void;
  /** `↑` (older) or `↓` (newer) in the box: puts a question asked before into the box. */
  recallQuestion: (direction: 'older' | 'newer') => void;
  /** "نیا سوال": forgets the conversation (the questions for `↑` stay). */
  newConversation: () => void;
  /** Everything forgotten: the app locked, or the Owner signed out. */
  reset: () => void;
}

const FRESH = {
  open: false,
  tall: false,
  draft: '',
  entries: [] as readonly DockEntry[],
  conversationId: null as string | null,
  asked: [] as readonly string[],
  recall: null as number | null,
  nextId: 1,
};

/** The words of a question for `↑`, or null for a quick question (it has no words of its own). */
function askedText(asked: Asked): string | null {
  return asked.type === 'chip' ? null : asked.text;
}

function updated(
  entries: readonly DockEntry[],
  id: number,
  change: (entry: DockEntry) => DockEntry,
): readonly DockEntry[] {
  return entries.map((entry) => (entry.id === id ? change(entry) : entry));
}

export const useAssistantStore = create<AssistantStore>((set, get) => ({
  ...FRESH,
  openDock: () => {
    set({ open: true });
  },
  closeDock: () => {
    set({ open: false });
  },
  setTall: (tall) => {
    set({ tall });
  },
  setDraft: (draft) => {
    // Typing ends a recall: the next `↑` starts from the newest question again.
    set({ draft, recall: null });
  },
  startEntry: (asked) => {
    const { nextId, entries, asked: history } = get();
    const text = askedText(asked);
    // The same question twice in a row is one stop for `↑`.
    const keep = text === null || history.at(-1) === text ? history : [...history, text];
    set({
      entries: [
        ...entries,
        { id: nextId, asked, working: true, steps: [], reply: null, questionId: null, error: null },
      ],
      asked: keep,
      nextId: nextId + 1,
      // A typed question empties the box; a chip leaves what is being typed alone.
      ...(asked.type === 'text' ? { draft: '', recall: null } : {}),
    });
    return nextId;
  },
  addStep: (step) => {
    const { entries } = get();
    const current = entries.findLast((entry) => entry.working);
    if (current === undefined) return;
    set({
      entries: updated(entries, current.id, (entry) => ({
        ...entry,
        steps: [...entry.steps, step],
      })),
    });
  },
  setReply: (id, reply) => {
    set({
      entries: updated(get().entries, id, (entry) => ({
        ...entry,
        working: false,
        reply,
        questionId: reply.question_id,
        error: null,
      })),
      conversationId: reply.conversation_id,
    });
  },
  setError: (id, error) => {
    set({
      entries: updated(get().entries, id, (entry) => ({
        ...entry,
        working: false,
        reply: null,
        error,
      })),
    });
  },
  resume: (id) => {
    set({
      entries: updated(get().entries, id, (entry) => ({
        ...entry,
        working: true,
        steps: [],
        reply: null,
        error: null,
      })),
    });
  },
  recallQuestion: (direction) => {
    const { asked, recall } = get();
    if (asked.length === 0) return;
    if (direction === 'older') {
      const index = recall === null ? asked.length - 1 : Math.max(recall - 1, 0);
      set({ recall: index, draft: asked[index] ?? '' });
      return;
    }
    if (recall === null) return;
    if (recall >= asked.length - 1) {
      set({ recall: null, draft: '' });
      return;
    }
    set({ recall: recall + 1, draft: asked[recall + 1] ?? '' });
  },
  newConversation: () => {
    set({ entries: [], draft: '', recall: null, conversationId: null });
  },
  reset: () => {
    set({ ...FRESH });
  },
}));
