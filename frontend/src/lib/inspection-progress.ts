import type { UnlistenFn } from '@tauri-apps/api/event';

import type { BackendClient, InspectionProgress } from './backend';

export const TERMINAL_INSPECTION_PHASES: ReadonlySet<InspectionProgress['phase']> = new Set([
  'completed',
  'completed_with_partial_failures',
  'cancelled',
  'failed'
]);

export function isTerminalInspectionPhase(phase: InspectionProgress['phase']): boolean {
  return TERMINAL_INSPECTION_PHASES.has(phase);
}

export interface InspectionProgressSubscription {
  onProgress: (progress: InspectionProgress) => void;
  onError: (message: string) => void;
}

export function installInspectionProgressListener(
  backend: BackendClient,
  subscription: InspectionProgressSubscription
): UnlistenFn {
  let disposed = false;
  let unlisten: UnlistenFn | undefined;

  void backend
    .subscribeInspectionProgress((progress) => {
      if (!disposed) subscription.onProgress(progress);
    })
    .then((stop) => {
      if (disposed) {
        stop();
      } else {
        unlisten = stop;
      }
    })
    .catch(() => {
      if (!disposed) {
        subscription.onError(
          'Live inspection progress is unavailable. Persisted inspection history remains authoritative.'
        );
      }
    });

  return () => {
    disposed = true;
    unlisten?.();
  };
}
