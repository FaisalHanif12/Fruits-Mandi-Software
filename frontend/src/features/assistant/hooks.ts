// What the question panel does, as hooks (Code Standards F6): the status the status bar and the
// panel read, the steps a running question reports, and the runner that sends a question, a quick
// question, a choice, or an old question read again. The store (store.ts) keeps what comes back; the
// screens only show it.

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo } from 'react';
import type { AnswerName } from '../../bindings/AnswerName';
import type { AskReply } from '../../bindings/AskReply';
import type { AssistantKeyRemoveInput } from '../../bindings/AssistantKeyRemoveInput';
import type { AssistantKeySaveInput } from '../../bindings/AssistantKeySaveInput';
import type { AssistantSettingsSetInput } from '../../bindings/AssistantSettingsSetInput';
import type { AssistantStatus } from '../../bindings/AssistantStatus';
import type { PickChoice } from '../../bindings/PickChoice';
import type { QuickChip } from '../../bindings/QuickChip';
import { uiErrorOf } from '../../shared/api';
import { usePagedQuery } from '../../shared/hooks';
import {
  assistantAsk,
  assistantKeyRemove,
  assistantKeySave,
  assistantLogList,
  assistantNewConversation,
  assistantPick,
  assistantQuick,
  assistantRerun,
  assistantSettingsSet,
  assistantStatus,
  assistantTest,
  listenSteps,
} from './api';
import { useAssistantStore, type Asked } from './store';

export const ASSISTANT_STATUS_KEY = ['assistant_status'] as const;
export const ASSISTANT_LOG_KEY = ['assistant_log'] as const;

/** On or off, the key, the model and today's count. A failure leaves the panel without a count. */
export function useAssistantStatus() {
  return useQuery({
    queryKey: ASSISTANT_STATUS_KEY,
    queryFn: assistantStatus,
    staleTime: Infinity,
    retry: false,
  });
}

/** Keeps the day's count in step with a reply: the backend's number, put where the bar reads it. */
function useCountSync() {
  const queryClient = useQueryClient();
  return useCallback(
    (reply: AskReply) => {
      queryClient.setQueryData<AssistantStatus>(ASSISTANT_STATUS_KEY, (status) =>
        status === undefined
          ? status
          : { ...status, count_today: reply.count_today, limit: reply.limit },
      );
      // The log has a new row.
      void queryClient.invalidateQueries({ queryKey: ASSISTANT_LOG_KEY });
    },
    [queryClient],
  );
}

/** Puts the step lines of a running question into the store while the panel is on screen. */
export function useStepLines() {
  useEffect(() => {
    let stop: (() => void) | null = null;
    let gone = false;
    void listenSteps((step) => {
      useAssistantStore.getState().addStep(step);
    }).then((off) => {
      if (gone) off();
      else stop = off;
    });
    return () => {
      gone = true;
      stop?.();
    };
  }, []);
}

/** What the panel can send. Each one adds its question at once, then waits for the backend. */
export interface AssistantRunner {
  /** A typed question (counts in the 20 a day). */
  ask: (text: string) => void;
  /** A quick question (no model, not counted). */
  quick: (chip: QuickChip, party: { id: number; name: AnswerName } | null) => void;
  /** A choice for a question that waits: a match, or the partner code. */
  pick: (entryId: number, choice: PickChoice) => void;
  /** An old question of the log, read again on today's books. */
  rerun: (questionId: number, text: string) => void;
}

export function useAssistantRunner(): AssistantRunner {
  const syncCount = useCountSync();
  // One object for the life of the panel, so an effect that holds it does not run again each render.
  return useMemo<AssistantRunner>(() => {
    const run = (asked: Asked, call: () => Promise<AskReply>, existing?: number) => {
      const store = useAssistantStore.getState();
      let id: number;
      if (existing === undefined) id = store.startEntry(asked);
      else {
        id = existing;
        store.resume(id);
      }
      call().then(
        (reply) => {
          useAssistantStore.getState().setReply(id, reply);
          syncCount(reply);
        },
        (failure: unknown) => {
          useAssistantStore.getState().setError(id, uiErrorOf(failure));
        },
      );
    };
    return {
      ask: (text) => {
        run({ type: 'text', text }, () =>
          assistantAsk({ text, conversation_id: useAssistantStore.getState().conversationId }),
        );
      },
      quick: (chip, party) => {
        run({ type: 'chip', chip, party: party === null ? null : party.name }, () =>
          assistantQuick({
            chip,
            party_id: party === null ? null : party.id,
            conversation_id: useAssistantStore.getState().conversationId,
          }),
        );
      },
      pick: (entryId, choice) => {
        const entry = useAssistantStore.getState().entries.find((found) => found.id === entryId);
        if (entry?.questionId == null) return;
        const questionId = entry.questionId;
        run(entry.asked, () => assistantPick({ question_id: questionId, choice }), entryId);
      },
      rerun: (questionId, text) => {
        run({ type: 'log', text }, () => assistantRerun({ question_id: questionId }));
      },
    };
  }, [syncCount]);
}

/**
 * "New question" (PRD FR-7.78): the backend is told first, with the id of the conversation the panel
 * leaves, so it drops that conversation's memory at once. Then the panel forgets it, so the next
 * question is sent with no conversation (`conversation_id` null). If the backend cannot be told the
 * panel forgets the conversation all the same: the next question starts a new one, and there is
 * nothing the Owner could do about it.
 */
export function useStartNewConversation(): () => void {
  return useCallback(() => {
    const leaving = useAssistantStore.getState().conversationId;
    void (async () => {
      try {
        await assistantNewConversation({ conversation_id: leaving });
      } catch {
        // Not shown: see above.
      }
    })();
    useAssistantStore.getState().newConversation();
  }, []);
}

// ---- Settings (PRD FR-7.85) ----

/** Turns questions on or off and sets the model. The answer is the new status. */
export function useSetAssistant() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: AssistantSettingsSetInput) => assistantSettingsSet(input),
    onSuccess: (status) => {
      queryClient.setQueryData(ASSISTANT_STATUS_KEY, status);
    },
  });
}

/**
 * Saves or replaces the key. The pasted key is the mutation's variable and nothing else: it is not
 * put into a store, the cache, a log or a message, and the answer (the status) never holds it.
 */
export function useSaveAssistantKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: AssistantKeySaveInput) => assistantKeySave(input),
    onSuccess: (status) => {
      queryClient.setQueryData(ASSISTANT_STATUS_KEY, status);
    },
  });
}

export function useRemoveAssistantKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: AssistantKeyRemoveInput) => assistantKeyRemove(input),
    onSuccess: (status) => {
      queryClient.setQueryData(ASSISTANT_STATUS_KEY, status);
    },
  });
}

/** The connection test. It counts as a question, so the count is read again. */
export function useTestAssistant() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => assistantTest(),
    onSuccess: (result) => {
      queryClient.setQueryData<AssistantStatus>(ASSISTANT_STATUS_KEY, (status) =>
        status === undefined
          ? status
          : { ...status, count_today: result.count_today, limit: result.limit },
      );
      void queryClient.invalidateQueries({ queryKey: ASSISTANT_LOG_KEY });
    },
  });
}

/** The question log page by page, newest first, with the count over the whole log (B31, FR-7.84). */
export function useAssistantLog() {
  return usePagedQuery({
    screen: 'assistant.log',
    queryKey: [...ASSISTANT_LOG_KEY],
    fetchPage: (offset, limit) => assistantLogList({ offset, limit }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}
