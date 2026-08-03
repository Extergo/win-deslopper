import { describe, expect, it } from 'vitest';

import capability from '../../../capabilities/main-window.json' with { type: 'json' };

describe('normal Tauri capability', () => {
  it('allows only the event listener lifecycle needed by inspection progress', () => {
    expect(capability.permissions).toEqual([
      'core:event:allow-listen',
      'core:event:allow-unlisten'
    ]);
  });

  it('does not grant frontend emission, shell, process, filesystem, registry, or mutation access', () => {
    expect(capability.permissions.join(' ')).not.toMatch(
      /emit|shell|process|fs:|filesystem|registry|mutation|\*/i
    );
  });
});
