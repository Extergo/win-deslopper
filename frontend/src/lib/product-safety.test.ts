import { describe, expect, it } from 'vitest';

import productPage from '../routes/+page.svelte?raw';

describe('normal Product Alpha presentation boundary', () => {
  it('contains no callable or visible mutation workflow', () => {
    expect(productPage).not.toMatch(/getMutation|executeMutation|rollbackMutation|mutationAlpha/);
    expect(productPage).not.toMatch(/Approve, apply|Apply changes|enable this session/);
    expect(productPage).toMatch(/Read-Only Product Alpha/);
    expect(productPage).toMatch(/Automatic restoration is not available/);
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
