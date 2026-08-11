import { describe, expect, it } from 'vitest';

import mutationPanel from './MutationAlphaPanel.svelte?raw';
import productPage from '../routes/+page.svelte?raw';

describe('normal Owner Mode presentation boundary', () => {
  it('exposes only closed Owner Mode M3 Apply and Undo while keeping the alpha harness gated', () => {
    expect(productPage).toMatch(/applyOwnerChange\(target\)/);
    expect(productPage).toMatch(/undoOwnerChange\(\)/);
    expect(productPage).toMatch(/productInfo\.buildMode === 'engineering mutation-alpha harness'/);
    expect(productPage).toMatch(/Experimental Apply & Undo/);
    expect(productPage).toMatch(/Owner Mode M3/);
    expect(productPage).toMatch(/Undo last \{selectedOwnerLabel\} change/);
    expect(productPage.match(/if \(ownerBusy/g)?.length).toBeGreaterThanOrEqual(2);
    expect(productPage).toMatch(/disabled=\{ownerBusy/);
    expect(productPage).toMatch(/set_taskbar_task_view_visibility/);
    expect(productPage).toMatch(/set_welcome_experience_enabled/);
    expect(productPage).toMatch(/set_tips_suggestions_enabled/);
    expect(productPage).toMatch(/set_notification_suggestions_enabled/);
    expect(productPage).toMatch(/set_settings_suggested_content_enabled/);
    expect(productPage).not.toMatch(/set_taskbar_show_desktop_enabled/);
    expect(productPage).not.toMatch(
      /CurrentVersion\\Explorer\\Advanced|TaskbarDa|ShowTaskViewButton|SubscribedContent|SoftLanding/
    );
    expect(mutationPanel).toMatch(/Internal Mutation Alpha/);
    expect(mutationPanel).toMatch(/This development machine is permanently blocked/);
    const historyMarkup = mutationPanel.split('Mutation transaction history')[1] ?? '';
    expect(historyMarkup).not.toMatch(/machineId|approvalNonce|approvalEvidenceSha256/);
  });

  it('contains no automatic network or upload path for diagnostics', () => {
    expect(productPage).not.toMatch(/fetch\(|XMLHttpRequest|axios|uploadFile|uploadDiagnostics/i);
    expect(productPage).toMatch(/showSaveFilePicker/);
    expect(productPage).toMatch(/Nothing was uploaded/);
  });

  it('provides keyboard-native controls and reduced-motion handling', () => {
    expect(productPage).toMatch(/<button/);
    expect(productPage).toMatch(/<dialog/);
    expect(productPage).toMatch(/prefers-reduced-motion/);
    expect(productPage).toMatch(/aria-live/);
  });
});
