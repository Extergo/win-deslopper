import { describe, expect, it, vi } from 'vitest';

import {
  createBackendClient,
  describeCommandError,
  type AppView,
  type BackendInvoker
} from './backend';

const emptyView: AppView = {
  cleanupItems: [],
  plannedItems: [],
  catalogueCount: 3,
  plannedCount: 0,
  selectedFilter: 'all',
  selectedSection: 'windows_cleanup',
  reviewOpen: false
};

describe('backend client', () => {
  it('loads the Rust-owned view through the narrow read command', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue(emptyView);
    const client = createBackendClient(invokeCommand);

    await expect(client.getAppView()).resolves.toEqual(emptyView);
    expect(invokeCommand).toHaveBeenCalledWith('get_app_view');
  });

  it('sends typed actions through the single mutation command', async () => {
    const invokeCommand = vi.fn<BackendInvoker>().mockResolvedValue({
      ...emptyView,
      plannedCount: 1
    });
    const client = createBackendClient(invokeCommand);

    await client.dispatch({ type: 'toggle_planned', componentId: 'copilot' });

    expect(invokeCommand).toHaveBeenCalledWith('dispatch_app_action', {
      action: { type: 'toggle_planned', componentId: 'copilot' }
    });
  });
});

describe('describeCommandError', () => {
  it('uses a typed backend message when one is available', () => {
    expect(
      describeCommandError({
        code: 'state_unavailable',
        message: 'Preview state unavailable.'
      })
    ).toBe('Preview state unavailable.');
  });

  it('does not leak opaque values into the interface', () => {
    expect(describeCommandError({ detail: new Error('internal') })).toBe(
      'The local preview could not be updated. Please try again.'
    );
  });
});
