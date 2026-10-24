// Showing what the backend refused (plan 03 contract 4.8, Code Standards 4.3, NFR-USE-13,
// NFR-USE-14, NFR-OBS-3): the words in the app language, the reference number under whole-form
// problems, the field named by `field` turned red, several problems at once, a code with no
// message, and grouped numbers. Each runs in Urdu and in English.

import { screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { errorsForField, errorsOutsideFields, splitErrors, uiErrorOf } from '../../src/shared/api';
import { useGroupingStore } from '../../src/shared/format';
import { ErrorNotice, Field, QtyInput, TextInput, fieldError } from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';
import { TEST_REFERENCE, refusal } from '../helpers/ui-error';

const QTY_CELL = 'lots[0].sales[2].qty';
const RATE_CELL = 'lots[0].sales[2].rate';
const OTHER_QTY_CELL = 'lots[0].sales[1].qty';

/** The reference line as the user reads it: the helper leaves the <code> markup in. */
function referenceLine(language: (typeof LANGUAGES)[number]): string {
  return message(language, 'err-reference', { ref: TEST_REFERENCE }).replace(/<\/?code>/g, '');
}

afterEach(() => {
  useGroupingStore.setState({ grouping: 'international' });
});

describe('error helpers', () => {
  const error = refusal([
    { code: 'ERR_REQUIRED', field: QTY_CELL },
    { code: 'ERR_NOT_A_NUMBER', field: QTY_CELL },
    { code: 'ERR_DATE_CLOSED', field: null, values: { closed_up_to: '09-10-2026' } },
    { code: 'ERR_REQUIRED', field: '' },
    { code: 'ERR_REQUIRED', field: RATE_CELL },
  ]);

  it('splits whole-form problems from field problems, and groups a field’s problems', () => {
    const { general, byField } = splitErrors(error);
    expect(general.map((item) => item.code)).toEqual(['ERR_DATE_CLOSED', 'ERR_REQUIRED']);
    expect([...byField.keys()]).toEqual([QTY_CELL, RATE_CELL]);
    expect(byField.get(QTY_CELL)?.map((item) => item.code)).toEqual([
      'ERR_REQUIRED',
      'ERR_NOT_A_NUMBER',
    ]);
  });

  it('finds the problems of one field path, exactly', () => {
    expect(errorsForField(error, QTY_CELL)).toHaveLength(2);
    expect(errorsForField(error, OTHER_QTY_CELL)).toEqual([]);
    expect(errorsForField(error, 'lots[0].sales[2]')).toEqual([]);
    expect(errorsForField(undefined, QTY_CELL)).toEqual([]);
    expect(errorsForField(null, QTY_CELL)).toEqual([]);
  });

  it('keeps every problem that no shown field covers, so none can vanish', () => {
    expect(errorsOutsideFields(error, [QTY_CELL, RATE_CELL]).map((item) => item.code)).toEqual([
      'ERR_DATE_CLOSED',
      'ERR_REQUIRED',
    ]);
    expect(errorsOutsideFields(error, [QTY_CELL])).toHaveLength(3);
    expect(errorsOutsideFields(error, [])).toHaveLength(5);
  });

  it('turns anything thrown into an error that has a message', () => {
    expect(uiErrorOf(new Error('x')).errors.map((item) => item.code)).toEqual(['ERR_INTERNAL']);
  });
});

describe.each(LANGUAGES)('showing a backend error (%s)', (language) => {
  it('a general error shows its words and the reference number, never the code', () => {
    renderIn(
      language,
      <ErrorNotice
        error={refusal([{ code: 'ERR_DATE_CLOSED', values: { closed_up_to: '09-10-2026' } }])}
      />,
    );
    const alert = screen.getByRole('alert');
    const text = visible(alert.textContent);
    expect(text).toContain(message(language, 'err-date-closed', { closed_up_to: '09-10-2026' }));
    expect(text).toContain(referenceLine(language));
    expect(text).not.toContain('ERR_');
    expect(text).not.toContain('err-');
    // The reference reads left to right and keeps its order inside Urdu text.
    const ref = alert.querySelector('bdi[dir="ltr"]');
    expect(ref).toHaveTextContent(TEST_REFERENCE);
  });

  it('a day sent as YYYY-MM-DD reads as DD-MM-YYYY in the words of ERR_CLOSE_BEFORE_LAST_ENTRY and ERR_DATE_BEFORE_GARDEN', () => {
    renderIn(
      language,
      <ErrorNotice
        error={refusal([
          { code: 'ERR_CLOSE_BEFORE_LAST_ENTRY', values: { last_date: '2026-10-12' } },
          { code: 'ERR_DATE_BEFORE_GARDEN', values: { garden_date: '2026-10-10' } },
        ])}
      />,
    );
    const text = visible(screen.getByRole('alert').textContent);
    expect(text).toContain(
      message(language, 'err-close-before-last-entry', { last_date: '12-10-2026' }),
    );
    expect(text).toContain(
      message(language, 'err-date-before-garden', { garden_date: '10-10-2026' }),
    );
    expect(text).not.toContain('2026-10-12');
    expect(text).not.toContain('2026-10-10');
  });

  it('shows no reference line when the backend sent none', () => {
    renderIn(language, <ErrorNotice error={refusal([{ code: 'ERR_REQUIRED' }], '')} />);
    expect(visible(screen.getByRole('alert').textContent)).toBe(message(language, 'err-required'));
  });

  it('a field error turns the named field red and shows its words under it', () => {
    const error = refusal([
      { code: 'ERR_QTY_TOO_LARGE', field: QTY_CELL, values: { max: 1_000_000 } },
    ]);
    renderIn(
      language,
      <>
        <Field label={message(language, 'common-back')} error={fieldError(error, QTY_CELL)}>
          {(control) => <QtyInput {...control} value="2000000" onChange={vi.fn()} />}
        </Field>
        <Field label={message(language, 'common-loading')} error={fieldError(error, RATE_CELL)}>
          {(control) => <QtyInput {...control} value="600" onChange={vi.fn()} />}
        </Field>
        <Field
          label={message(language, 'common-notifications')}
          error={fieldError(error, OTHER_QTY_CELL)}
        >
          {(control) => <QtyInput {...control} value="20" onChange={vi.fn()} />}
        </Field>
      </>,
    );
    const named = screen.getByLabelText(message(language, 'common-back'));
    expect(named).toHaveAttribute('aria-invalid', 'true');
    expect(named).toHaveClass('border-due', 'bg-due-soft');
    const note = document.getElementById(named.getAttribute('aria-describedby') ?? '');
    expect(visible(note?.textContent)).toBe(
      message(language, 'err-qty-too-large', { max: '1,000,000' }),
    );
    expect(note?.textContent).not.toContain('ERR_');
    // The other cells of the same row, and the same column of another row, stay normal.
    for (const label of ['common-loading', 'common-notifications']) {
      const other = screen.getByLabelText(message(language, label));
      expect(other).not.toHaveAttribute('aria-invalid', 'true');
      expect(other).not.toHaveAttribute('aria-describedby');
    }
  });

  it('several errors at once: field ones sit at their fields, the rest stay in the notice', () => {
    const error = refusal([
      { code: 'ERR_REQUIRED', field: QTY_CELL },
      { code: 'ERR_NOT_A_NUMBER', field: QTY_CELL },
      { code: 'ERR_MUST_BE_MORE_THAN_ZERO', field: RATE_CELL },
      { code: 'ERR_DATE_CLOSED', values: { closed_up_to: '09-10-2026' } },
      // A field this screen does not draw must still be told, not dropped.
      { code: 'ERR_PHONE_INVALID', field: 'lots[0].sales[2].phone' },
    ]);
    renderIn(
      language,
      <>
        <Field label={message(language, 'common-back')} error={fieldError(error, QTY_CELL)}>
          {(control) => <TextInput {...control} value="" onChange={vi.fn()} />}
        </Field>
        <Field label={message(language, 'common-loading')} error={fieldError(error, RATE_CELL)}>
          {(control) => <TextInput {...control} value="" onChange={vi.fn()} />}
        </Field>
        <ErrorNotice error={error} fieldsShown={[QTY_CELL, RATE_CELL]} />
      </>,
    );
    // Two problems on one field show one under the other.
    const qtyNote = document.getElementById(
      screen.getByLabelText(message(language, 'common-back')).getAttribute('aria-describedby') ??
        '',
    );
    expect(visible(qtyNote?.textContent)).toContain(message(language, 'err-required'));
    expect(visible(qtyNote?.textContent)).toContain(message(language, 'err-not-a-number'));
    const rateNote = document.getElementById(
      screen.getByLabelText(message(language, 'common-loading')).getAttribute('aria-describedby') ??
        '',
    );
    expect(visible(rateNote?.textContent)).toBe(message(language, 'err-must-be-more-than-zero'));
    // The notice holds the whole-form problem and the field that has no input, once each.
    const notice = visible(screen.getByRole('alert').textContent);
    expect(notice).toContain(message(language, 'err-date-closed', { closed_up_to: '09-10-2026' }));
    expect(notice).toContain(message(language, 'err-phone-invalid'));
    expect(notice).not.toContain(message(language, 'err-required'));
    expect(notice).not.toContain(message(language, 'err-must-be-more-than-zero'));
    expect(notice).toContain(TEST_REFERENCE);
    expect(within(screen.getByRole('alert')).getAllByText(/E-20261010-0007/)).toHaveLength(1);
  });

  it('shows no notice when every problem is already at its field', () => {
    const error = refusal([{ code: 'ERR_REQUIRED', field: QTY_CELL }]);
    renderIn(language, <ErrorNotice error={error} fieldsShown={[QTY_CELL]} />);
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('without a list of shown fields, the notice tells every problem', () => {
    const error = refusal([
      { code: 'ERR_REQUIRED', field: QTY_CELL },
      { code: 'ERR_DATE_CLOSED', values: { closed_up_to: '09-10-2026' } },
    ]);
    renderIn(language, <ErrorNotice error={error} />);
    const text = visible(screen.getByRole('alert').textContent);
    expect(text).toContain(message(language, 'err-required'));
    expect(text).toContain(message(language, 'err-date-closed', { closed_up_to: '09-10-2026' }));
  });

  it('an unknown code falls back to the "something went wrong" words and still shows the reference', () => {
    renderIn(
      language,
      <ErrorNotice error={refusal([{ code: 'ERR_SOMETHING_NEW_FROM_A_LATER_VERSION' }])} />,
    );
    const alert = screen.getByRole('alert');
    const text = visible(alert.textContent);
    expect(text).toContain(message(language, 'err-internal'));
    expect(text).toContain(referenceLine(language));
    expect(text).not.toContain('ERR_');
    expect(text).not.toContain('SOMETHING');
  });

  it('an unknown code on a field still fills that field’s note with the fallback words', () => {
    const error = refusal([{ code: 'ERR_SOMETHING_NEW', field: QTY_CELL }]);
    renderIn(
      language,
      <Field label={message(language, 'common-back')} error={fieldError(error, QTY_CELL)}>
        {(control) => <QtyInput {...control} value="1" onChange={vi.fn()} />}
      </Field>,
    );
    const input = screen.getByLabelText(message(language, 'common-back'));
    expect(input).toHaveAttribute('aria-invalid', 'true');
    const note = document.getElementById(input.getAttribute('aria-describedby') ?? '');
    expect(visible(note?.textContent)).toBe(message(language, 'err-internal'));
  });

  it('an error with no problem listed still says something went wrong, with the reference', () => {
    renderIn(language, <ErrorNotice error={refusal([])} />);
    const text = visible(screen.getByRole('alert').textContent);
    expect(text).toContain(message(language, 'err-internal'));
    expect(text).toContain(TEST_REFERENCE);
  });

  it('groups a limit like every other number: 10,000,000,000 and 10,00,00,00,000', () => {
    const error = refusal([{ code: 'ERR_AMOUNT_TOO_LARGE', values: { max: 10_000_000_000 } }]);
    const { unmount } = renderIn(language, <ErrorNotice error={error} />);
    expect(visible(screen.getByRole('alert').textContent)).toContain(
      message(language, 'err-amount-too-large', { max: '10,000,000,000' }),
    );
    unmount();
    useGroupingStore.setState({ grouping: 'lakh' });
    renderIn(language, <ErrorNotice error={error} />);
    expect(visible(screen.getByRole('alert').textContent)).toContain(
      message(language, 'err-amount-too-large', { max: '10,00,00,00,000' }),
    );
  });

  it('groups a limit on a field too', () => {
    const error = refusal([
      { code: 'ERR_QTY_TOO_LARGE', field: QTY_CELL, values: { max: 1_000_000 } },
    ]);
    useGroupingStore.setState({ grouping: 'lakh' });
    renderIn(
      language,
      <Field label={message(language, 'common-back')} error={fieldError(error, QTY_CELL)}>
        {(control) => <QtyInput {...control} value="1" onChange={vi.fn()} />}
      </Field>,
    );
    const input = screen.getByLabelText(message(language, 'common-back'));
    const note = document.getElementById(input.getAttribute('aria-describedby') ?? '');
    expect(visible(note?.textContent)).toBe(
      message(language, 'err-qty-too-large', { max: '10,00,000' }),
    );
  });
});

describe.each(LANGUAGES)('the data-file errors of plan 03 (%s)', (language) => {
  it.each([
    ['ERR_DISK_FULL', 'err-disk-full'],
    ['ERR_DATA_CHECK_FAILED', 'err-data-check-failed'],
    ['ERR_MIGRATION_FAILED', 'err-migration-failed'],
    ['ERR_DB_BUSY', 'err-db-busy'],
  ])('%s has its own words, not the fallback, and the reference follows', (code, id) => {
    renderIn(language, <ErrorNotice error={refusal([{ code }])} />);
    const text = visible(screen.getByRole('alert').textContent);
    expect(text).toContain(message(language, id));
    expect(message(language, id)).not.toBe(message(language, 'err-internal'));
    expect(text).toContain(referenceLine(language));
    expect(text).not.toContain('ERR_');
  });
});
