// "From باقی مال": the lots with goods left, to carry into the بکری on the screen (plan 76 point 1, PRD
// FR-12.17, FR-12.48). A dialog over the screen with the same list as the goods-left page (the
// backend's `bikri_stock_list`, page by page, with a search box), minus the lots that are on the screen
// already. Enter, or a double click, on a row puts that lot on the screen and closes the dialog; Esc closes it and
// leaves everything as it was. Nothing is worked out here: the goods left are the backend's.

import { Localized, useLocalization } from '@fluent/react';
import { PackageX } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { StockRow } from '../../../bindings/StockRow';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, ListRegion, Pager, listHeightMode } from '../../../shared/grid';
import { SHORTCUTS } from '../../../shared/keyboard';
import {
  Button,
  EmptyState,
  ErrorState,
  Field,
  FormDialog,
  LoadingState,
  TextInput,
} from '../../../shared/ui';
import { useStockList } from '../hooks/useBikriReads';
import { STOCK_COLUMNS } from '../stock-columns';

/**
 * The columns the dialog shows: the box is narrower than the page, so only what tells one lot from
 * another (the بکری it came from, the seller, the item) and what is left of it.
 */
const PICKER_FIELDS = new Set(['number', 'seller', 'item', 'balance']);
const PICKER_COLUMNS = STOCK_COLUMNS.filter((column) => PICKER_FIELDS.has(column.field));

interface CarriedLotPickerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The ids of the lots on the screen already: they are not offered again. */
  taken: ReadonlySet<number>;
  /** The lot to carry onto the screen. */
  onPick: (row: StockRow) => void;
}

export function CarriedLotPicker({ open, onOpenChange, taken, onPick }: CarriedLotPickerProps) {
  const { l10n } = useLocalization();
  const [search, setSearch] = useState('');
  const text = search.trim();
  const list = useStockList(
    {
      date: null,
      from: null,
      to: null,
      seller_id: null,
      item_id: null,
      include_written_off: false,
      lot_id: null,
      search: text === '' ? null : text,
    },
    open,
  );
  const rows = useMemo(
    () =>
      (list.data?.pages.flatMap((page) => page.rows) ?? []).filter(
        (row) => row.balance > 0 && !taken.has(row.lot_id),
      ),
    [list.data, taken],
  );

  let body;
  if (list.isError) {
    body = (
      <ErrorState
        title={<Localized id="bikri-carried-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (list.isPending) {
    body = <LoadingState rows={4} />;
  } else if (rows.length === 0) {
    body = (
      <EmptyState
        icon={PackageX}
        title={<Localized id="bikri-carried-empty" />}
        help={<Localized id="bikri-carried-empty-help" />}
      />
    );
  } else {
    body = (
      <>
        <ListRegion rowCount={rows.length}>
          <ListGrid<StockRow>
            columns={PICKER_COLUMNS}
            rows={rows}
            getRowId={(row) => String(row.lot_id)}
            labelId="bikri-stock-label"
            onOpen={(row) => {
              onPick(row);
              onOpenChange(false);
            }}
            heightMode={listHeightMode(rows.length)}
            sort={{ value: list.sort, onChange: list.setSort }}
          />
        </ListRegion>
        <Pager pager={list.pager} />
      </>
    );
  }

  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="bikri-carried-title" />}
      description={<Localized id="bikri-carried-help" />}
    >
      <div className="flex w-full flex-col gap-3">
        <Field label={<Localized id="bikri-carried-search" />}>
          {(control) => (
            <TextInput
              {...control}
              value={search}
              onChange={setSearch}
              placeholder={l10n.getString('bikri-carried-search')}
            />
          )}
        </Field>
        {body}
        {/* A way out with the mouse too (NFR-USE-8): Esc and a click on the scrim close it as well. */}
        <div className="flex justify-end">
          <Button
            shortcut={SHORTCUTS.back}
            onClick={() => {
              onOpenChange(false);
            }}
          >
            <Localized id="common-back" />
          </Button>
        </div>
      </div>
    </FormDialog>
  );
}
