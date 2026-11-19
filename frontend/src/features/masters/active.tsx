// The فعال / غیر فعال switch of a master (PRD Section 10.3, v1.23): on each row of the staff list and
// in the staff form. An active munshi prints in every header; an inactive one does not, so the switch
// must be easy to find and impossible to misread. It is the shared segmented control with the two words
// in it, never a colour alone (NFR-USE-12), and it saves through the ordinary master command
// `masters_set_active`: nothing is posted and it works on any day. One change at a time.

import { Localized, useLocalization } from '@fluent/react';
import { useCallback, useRef } from 'react';
import type { ListColumn } from '../../shared/grid';
import { SegmentedControl, showToast } from '../../shared/ui';
import { useSetMasterActive } from './hooks';
import type { MasterKind, MasterRecord } from './types';

/** The two values of the switch. */
type ActiveValue = 'on' | 'off';

/** Makes a master active or inactive, says so in a toast, and ignores a change while one is on its way. */
export function useActiveChange(kind: MasterKind) {
  const { l10n } = useLocalization();
  const setActive = useSetMasterActive(kind);
  const busy = useRef(false);
  const { mutate } = setActive;
  // Made again only when the language changes (a new grid is made then anyway), so a list that holds
  // it in its columns is not rebuilt under a click.
  const change = useCallback(
    (record: MasterRecord, next: boolean) => {
      if (busy.current || record.is_active === next) return;
      busy.current = true;
      mutate(
        { id: record.id, isActive: next },
        {
          onSuccess: () => {
            showToast(
              'success',
              l10n.getString(next ? 'masters-made-active' : 'masters-made-inactive'),
            );
          },
          onError: () => {
            showToast('error', l10n.getString('masters-active-failed'));
          },
          onSettled: () => {
            busy.current = false;
          },
        },
      );
    },
    [l10n, mutate],
  );
  return { change, isPending: setActive.isPending };
}

interface ActiveSwitchProps {
  record: MasterRecord;
  onChange: (record: MasterRecord, next: boolean) => void;
  /** A change is on its way: the switch waits. */
  busy?: boolean | undefined;
}

export function ActiveSwitch({ record, onChange, busy = false }: ActiveSwitchProps) {
  const { l10n } = useLocalization();
  return (
    <SegmentedControl<ActiveValue>
      groupLabel={l10n.getString('masters-active-switch-label')}
      value={record.is_active ? 'on' : 'off'}
      disabled={busy}
      options={[
        { value: 'on', label: <Localized id="masters-status-active" /> },
        { value: 'off', label: <Localized id="masters-status-inactive" /> },
      ]}
      onChange={(next) => {
        onChange(record, next === 'on');
      }}
    />
  );
}

/**
 * The status column of a list that has the switch on each row. `Space` on the focused cell of the column
 * does what pressing the other word does; Enter still opens the row.
 */
export function activeColumn(
  onChange: (record: MasterRecord, next: boolean) => void,
): ListColumn<MasterRecord> {
  return {
    field: 'status',
    // The two words side by side need the width of a short text (about 175 px), not of one word.
    type: 'text',
    headerId: 'masters-col-status',
    value: (row, { text }) =>
      text(row.is_active ? 'masters-status-active' : 'masters-status-inactive'),
    control: {
      render: (row) => <ActiveSwitch record={row} onChange={onChange} />,
      onSpace: (row) => {
        onChange(row, !row.is_active);
      },
    },
  };
}
