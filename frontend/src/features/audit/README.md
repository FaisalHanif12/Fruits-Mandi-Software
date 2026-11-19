# audit (plan 22, PRD FR-21.3, NFR-SEC-5)

The audit viewer: every change with who did it, when, what it was, the reason, and the old and new
values. It is read only (`audit_list`, `audit_options`); nothing here edits or deletes. It sits in
Settings as its own tab (`/settings?tab=audit`) and is exported as `AuditLog`.

- `pages/AuditPage.tsx`: the viewer. The filters (who, action, area, a record of an area, dates, words)
  and the open entry are kept in `store.ts`, so a language switch loses nothing. A filter change asks
  the backend once, 250 ms after the last key. A refusal that points at a box shows under that box.
- `components/AuditResults.tsx`: the list one page at a time (the shared pager) with the backend's
  count of the whole filter ("26 to 50 of 340"), the four states, Enter or a double click opens an entry.
- `components/AuditDetail.tsx`: one entry with the fields that changed, before and after. A secret is
  `hidden` from the backend and shown only as "Changed (not shown)". Money goes through the list's
  formatter. Esc goes back.
- `words.ts`: the actions and areas that have words in both languages (`audit.ftl`); a code the app does
  not know is shown as it is. `columns.ts`: the list columns by layout step.
- The free-text filter looks in the reason and in the old and new values (not in the action or area,
  which have their own boxes).
