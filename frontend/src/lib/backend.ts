import { invoke } from '@tauri-apps/api/core';

export type ComponentId = 'onedrive' | 'copilot' | 'promotional_content';
export type CatalogueFilter = 'all' | 'cloud' | 'ai' | 'promotions';
export type NavigationDestination = 'windows_cleanup' | 'extensions';

export interface CleanupItemView {
  id: ComponentId;
  iconLabel: string;
  name: string;
  description: string;
  category: string;
  status: string;
  risk: string;
  restart: string;
  compatibility: string;
  proposedActions: string;
  planned: boolean;
}

export interface AppView {
  cleanupItems: CleanupItemView[];
  plannedItems: CleanupItemView[];
  catalogueCount: number;
  plannedCount: number;
  selectedFilter: CatalogueFilter;
  selectedSection: NavigationDestination;
  reviewOpen: boolean;
}

export type AppAction =
  | { type: 'search_changed'; query: string }
  | { type: 'filter_changed'; filter: CatalogueFilter }
  | { type: 'toggle_planned'; componentId: ComponentId }
  | { type: 'navigate'; destination: NavigationDestination }
  | { type: 'open_review' }
  | { type: 'close_review' };

export interface CommandError {
  code: string;
  message: string;
}

export type BackendInvoker = (command: string, args?: Record<string, unknown>) => Promise<unknown>;

export interface BackendClient {
  getAppView(): Promise<AppView>;
  dispatch(action: AppAction): Promise<AppView>;
}

const tauriInvoker: BackendInvoker = (command, args) => invoke<unknown>(command, args);

export function createBackendClient(invokeCommand: BackendInvoker = tauriInvoker): BackendClient {
  return {
    getAppView: async () => (await invokeCommand('get_app_view')) as AppView,
    dispatch: async (action) => (await invokeCommand('dispatch_app_action', { action })) as AppView
  };
}

export function describeCommandError(error: unknown): string {
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message;
  }

  if (typeof error === 'string' && error.trim().length > 0) {
    return error;
  }

  return 'The local preview could not be updated. Please try again.';
}
