// What is wrong, and what is worth a look, in words (PRD FR-7.6, NFR-USE-12). A cell turns red or
// amber, but colour is never the only signal, so every problem is also written here, with the lot
// and line it is about. A problem that a field on the screen already shows under itself (the
// header, the "more" panel, the details of the line) is not told twice.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { CircleAlert, TriangleAlert } from 'lucide-react';
import { ErrorMessage, LinkButton } from '../../../shared/ui';
import { LINE_GRID_FIELD, LOT_GRID_FIELD, type Problem } from '../problems';

interface ProblemListProps {
  problems: readonly Problem[];
  /** The lot whose "more" panel is open, and the line whose details are open (indexes). */
  selectedLot: number;
  activeLine: number;
}

/** Is the problem shown by a field with words under it? Then the list leaves it out. */
function shownByField(problem: Problem, selectedLot: number, activeLine: number): boolean {
  const { where } = problem;
  if (where.kind === 'header') return true;
  if (where.kind === 'lot')
    return where.lot === selectedLot && LOT_GRID_FIELD[where.sub] === undefined;
  if (where.kind === 'line') {
    return (
      where.lot === selectedLot &&
      where.line === activeLine &&
      LINE_GRID_FIELD[where.sub] === undefined
    );
  }
  return false;
}

export function ProblemList({ problems, selectedLot, activeLine }: ProblemListProps) {
  const navigate = useNavigate();
  const listed = problems.filter((problem) => !shownByField(problem, selectedLot, activeLine));
  if (listed.length === 0) return null;
  return (
    <ul className="flex flex-col gap-1">
      {listed.map((problem, index) => {
        const warning = problem.severity === 'warning';
        const { where } = problem;
        return (
          <li
            key={`${problem.item.code}-${problem.item.field}-${String(index)}`}
            role={warning ? 'status' : 'alert'}
            className={`flex items-start gap-2 type-body ${warning ? 'text-warn' : 'text-due'}`}
          >
            {warning ? (
              <TriangleAlert aria-hidden="true" className="mt-1 size-icon-sm shrink-0" />
            ) : (
              <CircleAlert aria-hidden="true" className="mt-1 size-icon-sm shrink-0" />
            )}
            <span>
              {where.kind === 'line' ? (
                <Localized
                  id="bikri-where-line"
                  vars={{ lot: String(where.lot + 1), line: String(where.line + 1) }}
                >
                  <span className="type-label" />
                </Localized>
              ) : where.kind === 'lot' ? (
                <Localized id="bikri-where-lot" vars={{ lot: String(where.lot + 1) }}>
                  <span className="type-label" />
                </Localized>
              ) : null}{' '}
              <ErrorMessage item={problem.item}>
                <span />
              </ErrorMessage>
              {/* The shares are set on another screen, so the problem points the way (FR-17.1). */}
              {problem.item.code === 'ERR_SHARES_NOT_100' ? (
                <LinkButton
                  onClick={() => {
                    void navigate({ to: '/partners/setup' });
                  }}
                >
                  <Localized id="bikri-shares-open-setup" />
                </LinkButton>
              ) : null}
            </span>
          </li>
        );
      })}
    </ul>
  );
}
