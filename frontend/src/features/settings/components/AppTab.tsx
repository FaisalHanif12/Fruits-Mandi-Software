// The app area of Settings: updates and the support file (plan 22, NFR-INS-3, NFR-OBS-4).

import { SupportSection } from './SupportSection';
import { UpdatesSection } from './UpdatesSection';

export function AppTab() {
  return (
    <div className="flex max-w-reading flex-col gap-6">
      <UpdatesSection />
      <SupportSection />
    </div>
  );
}
