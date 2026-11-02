// One answer (PRD FR-7.74), in this order: the advice band when it is advice, the answer text with its
// figures, the tables and charts, the estimate band, the buttons, the source line and the next
// questions. The panel shows what the backend sent and nothing else: it never sums, never builds a
// figure, and never fills a gap in the model's words (CLAUDE.md rule 1, Code Standards F2, M12).

import { useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import type { Answer } from '../../../bindings/Answer';
import type { AskOutcome } from '../../../bindings/AskOutcome';
import { AnswerChartView } from './AnswerChartView';
import { AnswerTableView } from './AnswerTableView';
import { AnswerText } from './AnswerText';
import { AdviceBand, EstimateBand, GeneralAdviceBand } from './Bands';
import { RoundChip } from './RoundChip';
import { SourceButtons, SourceLine } from './SourceFooter';

interface AnswerBlockProps {
  answer: Answer;
  outcome: AskOutcome;
  /** A suggested next question was chosen: it is sent as a new question. */
  onAsk: (text: string) => void;
  /** A question is being worked on: the suggestions wait. */
  busy: boolean;
  /**
   * The answer is general advice (PRD FR-7.88, v1.19): the books had too little, so it rests on no figure
   * of the books. Its own calm band shows instead of the advice band. `ReplyBody` passes it from the
   * backend's marker (plan 52 item 11).
   */
  general?: boolean;
  /**
   * What goes right under the answer's words, before its tables: the change refused line of an answer
   * or advice whose question also asked for a change (PRD E98), so it is seen without scrolling.
   */
  afterText?: ReactNode;
}

export function AnswerBlock({
  answer,
  outcome,
  onAsk,
  busy,
  general = false,
  afterText,
}: AnswerBlockProps) {
  const { l10n } = useLocalization();
  return (
    <div className="flex flex-col gap-3" data-outcome={outcome}>
      {general ? (
        <GeneralAdviceBand />
      ) : answer.advice || outcome === 'advice' ? (
        <AdviceBand />
      ) : null}
      <AnswerText parts={answer.parts} />
      {afterText}
      {answer.tables.map((table) => (
        <AnswerTableView key={table.result_id} table={table} />
      ))}
      {answer.charts.map((chart) => (
        <AnswerChartView key={chart.result_id} chart={chart} />
      ))}
      {answer.estimate !== null ||
      outcome === 'estimate' ||
      answer.charts.some((chart) => chart.is_estimate) ? (
        <EstimateBand estimate={answer.estimate} />
      ) : null}
      <SourceButtons sources={answer.sources} />
      <SourceLine sources={answer.sources} readAt={answer.read_at} />
      {answer.next.length === 0 ? null : (
        <div className="flex flex-col gap-2">
          <p className="type-label text-ink-muted">{l10n.getString('assistant-next-label')}</p>
          <ul className="flex list-none flex-wrap gap-2">
            {answer.next.map((question) => (
              <li key={question}>
                <RoundChip
                  disabled={busy}
                  onClick={() => {
                    onAsk(question);
                  }}
                >
                  {question}
                </RoundChip>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
