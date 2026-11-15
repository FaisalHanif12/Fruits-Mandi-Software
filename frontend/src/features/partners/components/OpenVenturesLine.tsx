// One partner's open gardens and trips as ONE compact line on the overview (PRD 17.3, FR-12.29, plan 76
// point 11): "کھلے: سفر 1، باغ 0", with a button that opens the full list (`OpenVentures`) under it and
// shuts it again. The page stays short however many trips and gardens there are. The numbers of the line
// are the two counts the backend sent with the overview; the list is read from the backend when the
// button is pressed (`partners_open_ventures`), with its own loading and error words, and its figures
// are the backend's, only shown (CLAUDE.md rule 1). A partner in none shows nothing.

import { Localized } from '@fluent/react';
import { ChevronDown } from 'lucide-react';
import { useId, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Button, ErrorState, LoadingState, RUNNING_NUMBER } from '../../../shared/ui';
import { usePartnerOpenVentures } from '../hooks';
import { OpenVentures } from './OpenVentures';

interface OpenVenturesLineProps {
  /** The partner, to read his name in the language of the app. */
  partner: { partner_id: number; name_ur: string; name_en: string };
  /** How many open trips and gardens he is in, as the backend counted them. */
  trips: number;
  gardens: number;
}

export function OpenVenturesLine({ partner, trips, gardens }: OpenVenturesLineProps) {
  const language = useLanguage();
  const [open, setOpen] = useState(false);
  const listId = useId();
  const list = usePartnerOpenVentures(partner.partner_id, open);
  if (trips + gardens === 0) return null;
  const name = displayName(partner, language);
  let body = null;
  if (open) {
    if (list.isError) {
      body = (
        <ErrorState
          title={<Localized id="partners-ventures-failed" />}
          error={uiErrorOf(list.error)}
          onRetry={() => {
            void list.refetch();
          }}
        />
      );
    } else if (list.data === undefined) {
      body = <LoadingState rows={2} />;
    } else {
      body = <OpenVentures ventures={list.data} partnerName={name} />;
    }
  }
  return (
    <div className="flex flex-col gap-2" data-open-ventures-line="">
      <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <p className="flex flex-wrap items-baseline gap-x-3 type-body">
          <span className="type-section" lang={language}>
            {name}
          </span>
          <Localized
            id="partners-ventures-line"
            vars={{ trips: String(trips), gardens: String(gardens) }}
            elems={RUNNING_NUMBER}
          >
            <span className="text-ink-muted" />
          </Localized>
        </p>
        <Button
          expanded={open}
          controls={listId}
          onClick={() => {
            setOpen(!open);
          }}
        >
          <ChevronDown
            aria-hidden="true"
            className={`size-icon-md transition-calm ${open ? 'rotate-180' : ''}`}
          />
          <Localized id={open ? 'partners-ventures-hide' : 'partners-ventures-show'} />
        </Button>
      </div>
      {open ? <div id={listId}>{body}</div> : null}
    </div>
  );
}
