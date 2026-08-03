import { describe, expect, it, vi } from 'vitest';

import type { UnlistenFn } from '@tauri-apps/api/event';

import type { BackendClient, InspectionProgress } from './backend';
import {
  installInspectionProgressListener,
  isTerminalInspectionPhase
} from './inspection-progress';

function progress(phase: InspectionProgress['phase'], completedWork: number): InspectionProgress {
  return {
    inspectionId: 'inspection-test',
    phase,
    completedWork,
    totalWork: 20,
    currentComponent: phase === 'running_detectors' ? 'onedrive' : null,
    detectorResultStatus: null,
    warningCount: 0,
    errorCount: 0,
    cancellationAvailable: !isTerminalInspectionPhase(phase),
    status: phase,
    updatedAt: '1'
  };
}

describe('inspection progress subscription', () => {
  it('delivers progress and terminal completion, then removes the listener', async () => {
    let emit: ((value: InspectionProgress) => void) | undefined;
    const stop = vi.fn<UnlistenFn>();
    const backend = {
      subscribeInspectionProgress: vi.fn(async (handler: (value: InspectionProgress) => void) => {
        emit = handler;
        return stop;
      })
    } as unknown as BackendClient;
    const states: InspectionProgress[] = [];
    const cleanup = installInspectionProgressListener(backend, {
      onProgress: (value) => states.push(value),
      onError: vi.fn()
    });
    await Promise.resolve();

    emit?.(progress('running_detectors', 4));
    emit?.(progress('completed_with_partial_failures', 20));
    cleanup();

    expect(states.map(({ phase }) => phase)).toEqual([
      'running_detectors',
      'completed_with_partial_failures'
    ]);
    expect(isTerminalInspectionPhase(states.at(-1)!.phase)).toBe(true);
    expect(stop).toHaveBeenCalledOnce();
  });

  it('reports listener denial honestly without inventing completion', async () => {
    const onProgress = vi.fn();
    const onError = vi.fn();
    const backend = {
      subscribeInspectionProgress: vi.fn().mockRejectedValue(new Error('event.listen not allowed'))
    } as unknown as BackendClient;

    installInspectionProgressListener(backend, { onProgress, onError });
    await Promise.resolve();
    await Promise.resolve();

    expect(onProgress).not.toHaveBeenCalled();
    expect(onError).toHaveBeenCalledWith(expect.stringContaining('Persisted inspection history'));
  });

  it('unlistens if the page closes while listener registration is pending', async () => {
    let resolveListen: ((stop: UnlistenFn) => void) | undefined;
    const stop = vi.fn<UnlistenFn>();
    const backend = {
      subscribeInspectionProgress: vi.fn(
        () => new Promise<UnlistenFn>((resolve) => (resolveListen = resolve))
      )
    } as unknown as BackendClient;

    const cleanup = installInspectionProgressListener(backend, {
      onProgress: vi.fn(),
      onError: vi.fn()
    });
    cleanup();
    resolveListen?.(stop);
    await Promise.resolve();

    expect(stop).toHaveBeenCalledOnce();
  });
});
