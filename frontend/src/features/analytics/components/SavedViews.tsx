// Saved views (PRD FR-18.5): a filter set with a name ("کیلا کمیشن ماہانہ"), opened again with one
// choice. The list belongs to the user and is kept by the backend (`analytics_views_get` and
// `analytics_views_save`, which replaces the whole list). A view holds the filters only, never the
// figures. Saving a name that is already there replaces that view. The dashboard link comes with
// plan 19.
//
// The box lists the saved views; choosing one opens it (its view and its filters). "Save this view"
// asks for a name in a small dialog; "Delete" asks first (PRD FR-7.8). Both are refused by the
// backend in words, under the name field (an empty name, a name too long, a record that is gone).

import { Localized, useLocalization } from '@fluent/react';
import { useRef, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import {
  Button,
  ConfirmDialog,
  Field,
  FormDialog,
  FreeTextInput,
  Select,
  ErrorNotice,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useSavedViews, useSaveViews } from '../hooks';
import type { AnalyticsFilters, AnalyticsSavedView, AnalyticsView } from '../types';

interface SavedViewsProps {
  /** The view and the filters on screen now: what "save" keeps. */
  current: { view: AnalyticsView; filters: AnalyticsFilters };
  /** The name of the saved view that was opened or saved last, if any. */
  savedName: string | null;
  onOpen: (saved: AnalyticsSavedView) => void;
  onSaved: (name: string | null) => void;
}

export function SavedViews({ current, savedName, onOpen, onSaved }: SavedViewsProps) {
  const { l10n } = useLocalization();
  const views = useSavedViews();
  const save = useSaveViews();
  const [naming, setNaming] = useState(false);
  const [name, setName] = useState('');
  const [deleting, setDeleting] = useState(false);
  const saveButton = useRef<HTMLButtonElement>(null);

  const list = views.data ?? [];
  const error = save.isError ? uiErrorOf(save.error) : null;

  const keep = (next: AnalyticsSavedView[], then: () => void) => {
    save.mutate(next, { onSuccess: then });
  };

  const submit = () => {
    const wanted = name.trim();
    const view: AnalyticsSavedView = { name: wanted, view: current.view, filters: current.filters };
    // The same name replaces that view; a new name goes at the end.
    const at = list.findIndex((saved) => saved.name === wanted);
    const next =
      at === -1 ? [...list, view] : list.map((saved, index) => (index === at ? view : saved));
    keep(next, () => {
      setNaming(false);
      setName('');
      onSaved(wanted);
      showToast('success', l10n.getString('analytics-saved-done', { name: wanted }));
    });
  };

  // The index of the view being saved in the list the backend will read, for its field errors.
  const index = Math.max(
    list.findIndex((saved) => saved.name === name.trim()),
    -1,
  );
  const fieldIndex = index === -1 ? list.length : index;

  return (
    <div
      role="group"
      aria-label={l10n.getString('analytics-saved-label')}
      className="flex flex-wrap items-end gap-3"
    >
      <Field label={<Localized id="analytics-saved-label" />}>
        {(control) => (
          <Select<string>
            {...control}
            value={savedName ?? undefined}
            onChange={(chosen) => {
              const found = list.find((saved) => saved.name === chosen);
              if (found !== undefined) onOpen(found);
            }}
            options={list.map((saved) => ({ value: saved.name, label: saved.name }))}
            placeholder={
              <Localized id={list.length === 0 ? 'analytics-saved-none' : 'analytics-saved-open'} />
            }
          />
        )}
      </Field>
      <Button
        ref={saveButton}
        onClick={() => {
          setName(savedName ?? '');
          save.reset();
          setNaming(true);
        }}
      >
        <Localized id="analytics-saved-save" />
      </Button>
      {savedName === null ? null : (
        <Button
          variant="danger"
          onClick={() => {
            setDeleting(true);
          }}
        >
          <Localized id="analytics-saved-delete" />
        </Button>
      )}
      {views.isError ? (
        <Localized id="analytics-saved-failed">
          <p role="alert" className="type-body text-due" />
        </Localized>
      ) : null}

      <FormDialog
        open={naming}
        onOpenChange={setNaming}
        title={<Localized id="analytics-saved-dialog-title" />}
        returnFocusTo={saveButton}
      >
        <form
          className="flex flex-col gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            submit();
          }}
        >
          <Field
            label={<Localized id="analytics-saved-name" />}
            hint={<Localized id="analytics-saved-name-hint" />}
            error={fieldError(error, `views[${String(fieldIndex)}].name`)}
          >
            {(control) => <FreeTextInput {...control} value={name} onChange={setName} />}
          </Field>
          {error === null ? null : (
            <ErrorNotice error={error} fieldsShown={[`views[${String(fieldIndex)}].name`]} />
          )}
          <div className="flex justify-end gap-2">
            <Button
              onClick={() => {
                setNaming(false);
              }}
            >
              <Localized id="common-back" />
            </Button>
            <Button variant="primary" busy={save.isPending} onClick={submit}>
              <Localized id="analytics-saved-confirm" />
            </Button>
          </div>
        </form>
      </FormDialog>

      <ConfirmDialog
        open={deleting}
        onOpenChange={setDeleting}
        title={<Localized id="analytics-saved-delete-title" />}
        description={
          <Localized id="analytics-saved-delete-body" vars={{ name: savedName ?? '' }} />
        }
        confirmLabel={<Localized id="analytics-saved-delete" />}
        tone="danger"
        onConfirm={() => {
          keep(
            list.filter((saved) => saved.name !== savedName),
            () => {
              onSaved(null);
              showToast('success', l10n.getString('analytics-saved-deleted'));
            },
          );
        }}
      />
    </div>
  );
}
