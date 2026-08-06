import { describe, expect, it } from 'vitest';

import mutationPanel from './MutationAlphaPanel.svelte?raw';
import productPage from '../routes/+page.svelte?raw';

describe('normal Product Alpha presentation boundary', () => {
  it('contains no direct mutation calls and gates the internal entry point on build metadata', () => {
    expect(productPage).not.toMatch(/getMutation|executeMutation|rollbackMutation|mutationAlpha/);
    expect(productPage).toMatch(/productInfo\.buildMode === 'internal mutation-alpha compile'/);
    expect(productPage).toMatch(/Experimental Apply & Undo/);
    expect(productPage).toMatch(/Read-Only Product Alpha/);
    expect(productPage).toMatch(/Automatic restoration is not available/);
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
