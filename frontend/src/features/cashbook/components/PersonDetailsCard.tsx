// The type and the link of a person who is lent money (PRD FR-16.4, FR-16.13, plan 76 point 10), above his loans:
// staff, garden owner, market-trip person or other, and what his loans are linked to (the shop or one
// partner). A button changes them. The link is fixed while he owes something: the dialog shows it and says
// why, and the backend refuses a different one too (`ERR_LINK_FIXED_WHILE_OWED`). The type is a label: it
// changes no amount. Nothing here works out money (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { UserCog } from 'lucide-react';
import { useState } from 'react';
import type { LentPerson } from '../../../bindings/LentPerson';
import type { PersonType } from '../../../bindings/PersonType';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  ErrorNotice,
  Fact,
  Field,
  FormDialog,
  Select,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useUpdateLentPerson } from '../hooks';
import {
  LINK_FIELD_PATHS,
  PERSON_LINK_KINDS,
  PERSON_TYPES,
  linkDraftOf,
  linkOf,
  personTypeId,
  type LinkDraft,
} from '../link-draft';
import { LinkChoice } from './LinkChoice';
import { LinkWords } from './LinkWords';

const FIELD_PATHS = ['person_account_id', ...LINK_FIELD_PATHS];

function DetailsForm({
  person,
  owes,
  onClose,
}: {
  person: LentPerson;
  owes: boolean;
  onClose: () => void;
}) {
  const { l10n } = useLocalization();
  const [type, setType] = useState<PersonType>(person.person_type);
  const [link, setLink] = useState<LinkDraft>(() => linkDraftOf(person.link));
  const [error, setError] = useState<UiError | null>(null);
  const update = useUpdateLentPerson();
  const flight = useSingleFlight();
  const submit = async () => {
    setError(null);
    try {
      await update.mutateAsync({
        person_account_id: person.account.id,
        person_type: type,
        // While he owes, his link stays as it is.
        link: owes ? null : linkOf(link),
      });
      showToast('success', l10n.getString('cashbook-person-saved'));
      onClose();
    } catch (failure: unknown) {
      setError(uiErrorOf(failure));
    }
  };
  const submitNow = () => {
    void flight.run(submit, undefined);
  };
  const navigation = useFormNavigation(submitNow);
  useShortcut(SHORTCUTS.save, submitNow);
  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-4"
    >
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
      <Field
        label={<Localized id="cashbook-person-type" />}
        error={fieldError(error, 'person_type')}
      >
        {(props) => (
          <Select<PersonType>
            {...props}
            value={type}
            onChange={setType}
            options={PERSON_TYPES.map((candidate) => ({
              value: candidate,
              label: <Localized id={personTypeId(candidate)} />,
            }))}
          />
        )}
      </Field>
      <LinkChoice
        draft={link}
        onChange={setLink}
        error={error}
        kinds={PERSON_LINK_KINDS}
        labelId="cashbook-link-for"
        helpId="cashbook-link-hint-person"
        locked={owes}
      />
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || update.isPending}
          onClick={submitNow}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
    </form>
  );
}

export function PersonDetailsCard({ person, owes }: { person: LentPerson; owes: boolean }) {
  const [open, setOpen] = useState(false);
  return (
    <div className="flex flex-wrap items-end justify-between gap-3">
      <dl className="grid grid-cols-2 gap-3">
        <Fact label={<Localized id="cashbook-person-type" />}>
          <Localized id={personTypeId(person.person_type)} />
        </Fact>
        <Fact label={<Localized id="cashbook-link-for" />}>
          <LinkWords link={person.link} />
        </Fact>
      </dl>
      <Button
        onClick={() => {
          setOpen(true);
        }}
      >
        <UserCog aria-hidden="true" className="size-icon-md" />
        <Localized id="cashbook-person-change" />
      </Button>
      <FormDialog
        open={open}
        onOpenChange={setOpen}
        title={<Localized id="cashbook-person-change" />}
      >
        {open ? (
          <DetailsForm
            person={person}
            owes={owes}
            onClose={() => {
              setOpen(false);
            }}
          />
        ) : null}
      </FormDialog>
    </div>
  );
}
