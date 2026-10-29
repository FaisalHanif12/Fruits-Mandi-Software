// The rules as hooks (Code Standards F6). A save or a change of active state refreshes the list;
// a change of the global defaults refreshes both the settings and the list (the defaults are rules).

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { usePagedQuery } from '../../shared/hooks';
import { rulesGet, rulesList, rulesSave, rulesSetActive, rulesTest } from './api';
import type { RuleTestInput } from '../../bindings/RuleTestInput';
import type { RuleDraft, RuleFilter } from './types';

const RULES_KEY = ['rules'] as const;

/** The rules for the filter, page by page, with the count and the totals of the whole filter. */
export function useRuleList(filter: RuleFilter) {
  return usePagedQuery({
    screen: 'rules.list',
    queryKey: [...RULES_KEY, 'list', filter],
    fetchPage: (offset, limit) => rulesList(filter, offset, limit),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

export function useRule(id: number) {
  return useQuery({ queryKey: [...RULES_KEY, 'one', id], queryFn: () => rulesGet(id) });
}

export function useSaveRule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (draft: RuleDraft) => rulesSave(draft),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: RULES_KEY }),
  });
}

export function useSetRuleActive() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, isActive }: { id: number; isActive: boolean }) =>
      rulesSetActive(id, isActive),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: RULES_KEY }),
  });
}

/** Testing a rule asks the backend and saves nothing, so it is a mutation the user starts. */
export function useTestRule() {
  return useMutation({ mutationFn: (input: RuleTestInput) => rulesTest(input) });
}
