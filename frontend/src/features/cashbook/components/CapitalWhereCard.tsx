// "Where the capital is" (سرمایہ کہاں ہے), a card on the روکڑ page (PRD FR-16.14, plans 78 E and 87). It reads like a
// statement of the books as they are now (or on a chosen day):
//   1. a headline, the partners' total capital, with ✓ "both sides are equal";
//   2. where the money is, in two blocks with no minus sign: to receive and in hand, and to pay, each with its
//      subtotal, then the net;
//   3. whose it is: each partner's کل رقم with his share and a bar, the opening capital, the advances still open;
//   4. when the opening difference is not 0, an amber warning with the accounts that make it and, if a partner's
//      opening capital sits on it, the button that puts it into روکڑ with a reason and the Owner's sign.
// Every line opens its page. The card is a folded section, shut at first, so the list of cash rows stays on top; it
// reads nothing until it is opened. It is behind the partner code (FR-17.11): while the section is locked the
// answer is `ERR_PARTNER_LOCKED` and the box asks for the code in its place.
//
// Nothing is worked out here: every amount, share, subtotal and the headline come from the backend. Four states:
// loading, an error with a retry (the code box when locked), nothing recorded yet (a message and a button), data.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Landmark, Printer } from 'lucide-react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate, isZeroPaisa } from '../../../shared/format';
import { SHORTCUTS, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  EmptyState,
  ErrorState,
  FoldSection,
  GoToDate,
  LinkButton,
  LoadingState,
  showToast,
} from '../../../shared/ui';
import { useOpenCapitalTarget } from '../capital-links';
import type { CapitalWhere, CapitalWhereKind } from '../capital-types';
import { useCapitalWhere, usePlaceOpeningCapital } from '../hooks';
import { CapitalBlock } from './capital/CapitalBlock';
import { CapitalHeadline, CapitalNet } from './capital/CapitalHeadline';
import { CapitalWhose } from './capital/CapitalWhose';
import { OpeningDifferenceNotice } from './capital/OpeningDifferenceNotice';
import { PlaceCapitalDialog } from './capital/PlaceCapitalDialog';

/** The lines of each block that show even at 0 (PRD FR-16.14): the main ones. */
const MAIN_HAVE: ReadonlySet<CapitalWhereKind> = new Set(['cash', 'buyers_owe', 'seller_advances']);
const MAIN_OWE: ReadonlySet<CapitalWhereKind> = new Set(['sellers_payable']);

/** Nothing is recorded: every line of the card is 0. The backend sends the lines; the screen only asks if they are 0. */
function isEmptyCard(card: CapitalWhere): boolean {
  return (
    card.difference === null &&
    [...card.have_lines, ...card.owe_lines, ...card.whose_lines].every((line) =>
      isZeroPaisa(line.amount_paisa),
    )
  );
}

interface CapitalWhereBodyProps {
  today: string;
  onShowCash: () => void;
  onPrint: (asOf: string | null) => void;
}

function CapitalWhereBody({ today, onShowCash, onPrint }: CapitalWhereBodyProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  // The day the card is as of, `YYYY-MM-DD`: null is today (the backend's day, passed in as `today`). The backend reads
  // the day as typed, `DD-MM-YYYY`, so it is written that way when asked.
  const [asOf, setAsOf] = useState<string | null>(null);
  const [boxOpen, setBoxOpen] = useState(false);
  const asOfText = asOf === null ? null : isoToDisplayDate(asOf);
  const read = useCapitalWhere(true, asOfText);
  const place = usePlaceOpeningCapital();
  const open = useOpenCapitalTarget(onShowCash);
  useShortcut(SHORTCUTS.print, () => {
    onPrint(asOfText);
  });

  if (read.isError) {
    return (
      <ErrorState
        title={<Localized id="cashbook-capital-failed" />}
        error={uiErrorOf(read.error)}
        onRetry={() => {
          void read.refetch();
        }}
      />
    );
  }
  const card = read.data;
  if (card === undefined) return <LoadingState rows={4} />;

  const closeBox = (next: boolean) => {
    setBoxOpen(next);
    if (!next) place.reset();
  };
  const confirmPlace = (reason: string, sign: string) => {
    if (card.difference === null) return;
    place.mutate(
      {
        partner_ids: card.difference.placeable_partners.map((partner) => partner.id),
        reason,
        sign,
      },
      {
        onSuccess: () => {
          setBoxOpen(false);
          showToast('success', <Localized id="cashbook-capital-placed-toast" />);
        },
      },
    );
  };

  if (isEmptyCard(card)) {
    return (
      <EmptyState
        icon={Landmark}
        title={<Localized id="cashbook-capital-empty" />}
        help={<Localized id="cashbook-capital-empty-help" />}
        action={
          <Button
            variant="primary"
            onClick={() => {
              void navigate({ to: '/partners/capital' });
            }}
          >
            <Localized id="cashbook-capital-empty-action" />
          </Button>
        }
      />
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-end justify-between gap-x-4 gap-y-2">
        <Localized id="cashbook-capital-help">
          <p className="type-body min-w-0 flex-1 text-ink-muted" />
        </Localized>
        <div className="flex flex-wrap items-end gap-3">
          <GoToDate
            today={today}
            onGo={(day) => {
              setAsOf(day === today ? null : day);
            }}
            label={<Localized id="cashbook-capital-go-label" />}
            info={<Localized id="cashbook-capital-go-info" />}
            placeholder={l10n.getString('cashbook-capital-go-placeholder')}
          />
          {asOf === null ? null : (
            <LinkButton
              onClick={() => {
                setAsOf(null);
              }}
            >
              <Localized id="cashbook-capital-go-today" />
            </LinkButton>
          )}
          <Button
            shortcut={SHORTCUTS.print}
            onClick={() => {
              onPrint(asOfText);
            }}
          >
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="report-preview" />
          </Button>
        </div>
      </div>
      <CapitalHeadline
        net={card.net_paisa}
        side={card.net_side}
        mark={!card.sides_equal ? 'unequal' : card.difference === null ? 'equal' : 'none'}
        asOfDate={card.as_of}
      />
      {card.difference === null ? null : (
        <OpeningDifferenceNotice
          difference={card.difference}
          onPlace={() => {
            setBoxOpen(true);
          }}
          onOpen={open}
        />
      )}
      <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
        <CapitalBlock
          titleId="cashbook-capital-receive-title"
          lines={card.have_lines}
          total={card.have_total_paisa}
          main={MAIN_HAVE}
          onOpen={open}
        />
        <CapitalBlock
          titleId="cashbook-capital-owe-title"
          lines={card.owe_lines}
          total={card.owe_total_paisa}
          main={MAIN_OWE}
          onOpen={open}
        />
      </div>
      <CapitalNet net={card.net_paisa} side={card.net_side} />
      <CapitalWhose
        lines={card.whose_lines}
        total={card.whose_lines_total_paisa}
        totalSide={card.whose_lines_side}
        onOpen={open}
      />
      <PlaceCapitalDialog
        open={boxOpen}
        onOpenChange={closeBox}
        error={place.isError ? uiErrorOf(place.error) : null}
        busy={place.isPending}
        onConfirm={confirmPlace}
      />
    </div>
  );
}

interface CapitalWhereCardProps {
  /** The backend's today, `YYYY-MM-DD`. */
  today: string;
  /** The cash line opens the cash rows: the page is already there, so it brings them into view. */
  onShowCash: () => void;
  /** Print, PDF: the page shows the preview of the card as of the day chosen, as typed `DD-MM-YYYY` (null = today). */
  onPrint: (asOf: string | null) => void;
}

/** The card on the روکڑ page: shut at first, and nothing is read until it is opened. */
export function CapitalWhereCard({ today, onShowCash, onPrint }: CapitalWhereCardProps) {
  return (
    <FoldSection title={<Localized id="cashbook-capital-title" />}>
      <CapitalWhereBody today={today} onShowCash={onShowCash} onPrint={onPrint} />
    </FoldSection>
  );
}
