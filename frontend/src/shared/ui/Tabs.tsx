// Tabs (design system 6, board UsersUr): a row of sections, one open. Built on Radix Tabs, so
// ← and → move between tabs in the direction the user sees, Home and End jump, and the panel is
// linked for screen readers. The open tab is in primary.

import { Tabs as RadixTabs } from 'radix-ui';
import type { ReactNode } from 'react';
import { useUiLanguage } from '../i18n';

export interface TabItem {
  id: string;
  label: ReactNode;
  content: ReactNode;
}

interface TabsProps {
  items: readonly TabItem[];
  value: string;
  onValueChange: (id: string) => void;
  /** What the row of tabs is, read by screen readers. Comes from Fluent. */
  label: string;
}

export function Tabs({ items, value, onValueChange, label }: TabsProps) {
  const { direction } = useUiLanguage();
  return (
    <RadixTabs.Root
      value={value}
      onValueChange={onValueChange}
      dir={direction}
      className="flex flex-col gap-5"
    >
      <RadixTabs.List aria-label={label} className="flex flex-wrap gap-2 border-b border-border">
        {items.map((item) => (
          <RadixTabs.Trigger
            key={item.id}
            value={item.id}
            className="type-control h-control rounded-t-2 px-4 text-ink-muted transition-calm data-[state=active]:bg-primary-soft data-[state=active]:text-primary"
          >
            {item.label}
          </RadixTabs.Trigger>
        ))}
      </RadixTabs.List>
      {items.map((item) => (
        <RadixTabs.Content key={item.id} value={item.id} className="flex flex-col gap-4">
          {item.content}
        </RadixTabs.Content>
      ))}
    </RadixTabs.Root>
  );
}
