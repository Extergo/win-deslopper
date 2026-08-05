import type {
  DesiredState,
  DetectionObservation,
  DriftEvent,
  InspectionProgress,
  PlatformDashboard,
  ProductComponent
} from './backend';

export type ProductSection =
  'overview' | 'components' | 'desired' | 'drift' | 'history' | 'settings';

export type ComponentFilter =
  | 'all'
  | 'changed'
  | 'desired_differs'
  | 'managed'
  | 'permission_limited'
  | 'unknown'
  | 'failed'
  | 'package'
  | 'policy'
  | 'preference';

export const onboardingPrinciples = [
  'Deslopper inspects before it proposes anything.',
  'This Product Alpha is read-only and never silently alters Windows.',
  'Unknown does not mean broken, and permission-limited does not mean absent.',
  'Inspection history stays local by default and sensitive evidence is redacted.'
] as const;

export const diagnosticsCategories = [
  'Product and schema versions',
  'Windows edition and build',
  'Inspection and detector summaries',
  'Redacted evidence and errors',
  'Read-only capability summary'
] as const;

export function stateText(state: Record<string, unknown> | undefined): string {
  if (!state) return 'Not observed';
  const [kind, raw] = Object.entries(state)[0] ?? ['Unknown', null];
  if (kind === 'UserPreference' && isRecord(raw) && typeof raw.enabled === 'boolean') {
    return raw.enabled ? 'Enabled' : 'Disabled';
  }
  if (kind === 'Policy' && isRecord(raw)) {
    if (raw.configured === false) return 'Not configured';
    return raw.enabled === true ? 'Enabled by policy' : 'Disabled by policy';
  }
  if (kind === 'Package' && isRecord(raw)) {
    return `Current user: ${words(String(raw.current_user ?? 'unknown'))}; provisioned: ${words(String(raw.provisioned ?? 'unknown'))}`;
  }
  if (kind === 'OneDrive' && isRecord(raw)) {
    return raw.installed === true ? 'Installed' : 'Not installed';
  }
  if (kind === 'Unsupported') return 'Unsupported';
  if (kind === 'Managed') return 'Externally managed';
  if (kind === 'Unknown') return 'Unknown';
  return words(kind);
}

export function observationStatus(observation: DetectionObservation | undefined): string {
  if (!observation) return 'Not inspected';
  if (observation.detectorStatus === 'failed') return 'Failed inspection';
  if (observation.detectorStatus === 'cancelled') return 'Incomplete';
  if (!observation.applicable) return 'Not applicable';
  if (isPermissionLimited(observation)) return 'Permission limited';
  if (observation.detectorStatus === 'unknown' || stateText(observation.current) === 'Unknown') {
    return 'Unknown';
  }
  if (isExternallyManaged(observation)) return 'Externally managed';
  return 'Observed';
}

export function isPermissionLimited(observation: DetectionObservation | undefined): boolean {
  if (!observation) return false;
  return (
    observation.packageCompleteness === 'permission_limited' ||
    observation.packages.some((item) =>
      [item.currentUser, item.otherUsers, item.provisioning].includes('permission_limited')
    )
  );
}

export function isExternallyManaged(observation: DetectionObservation | undefined): boolean {
  return Boolean(
    observation && ['DomainPolicy', 'Mdm', 'LocalPolicy'].includes(observation.authority)
  );
}

export function desiredDiffers(
  desired: DesiredState | undefined,
  observation: DetectionObservation | undefined
): boolean {
  if (!desired || !observation) return false;
  return JSON.stringify(desired.state) !== JSON.stringify(observation.current);
}

export function filterComponents(
  catalogue: ProductComponent[],
  observations: DetectionObservation[],
  desiredStates: DesiredState[],
  drift: DriftEvent[],
  filter: ComponentFilter,
  query: string
): ProductComponent[] {
  const normalizedQuery = query.trim().toLowerCase();
  return catalogue.filter((component) => {
    const observation = observations.find((item) => item.componentId === component.componentId);
    const desired = desiredStates.find((item) => item.component_id === component.componentId);
    const changed = drift.some(
      (event) => event.component_id === component.componentId && !event.resolved
    );
    const matchesFilter =
      filter === 'all' ||
      (filter === 'changed' && changed) ||
      (filter === 'desired_differs' && desiredDiffers(desired, observation)) ||
      (filter === 'managed' && isExternallyManaged(observation)) ||
      (filter === 'permission_limited' && isPermissionLimited(observation)) ||
      (filter === 'unknown' && observationStatus(observation) === 'Unknown') ||
      (filter === 'failed' && observation?.detectorStatus === 'failed') ||
      (filter === 'package' && component.isPackage) ||
      (filter === 'policy' && component.configuration.toLowerCase().includes('policy')) ||
      (filter === 'preference' && component.configuration.toLowerCase().includes('preference'));
    const matchesQuery =
      normalizedQuery.length === 0 ||
      [component.name, component.category, component.purpose, component.componentId]
        .join(' ')
        .toLowerCase()
        .includes(normalizedQuery);
    return matchesFilter && matchesQuery;
  });
}

export function inspectionFreshness(timestamp: string | undefined, now = Date.now()): string {
  if (!timestamp) return 'No inspection yet';
  const observed = Number(timestamp);
  if (!Number.isFinite(observed)) return 'Freshness unknown';
  const ageMinutes = Math.max(0, (now - observed) / 60_000);
  if (ageMinutes < 15) return 'Fresh';
  if (ageMinutes < 24 * 60) return 'Inspected today';
  if (ageMinutes < 7 * 24 * 60) return 'Older than a day';
  return 'Stale - inspect again before relying on previews';
}

export function inspectionProgressDetail(
  progress: InspectionProgress,
  currentComponentName?: string
): string {
  if (progress.phase === 'completed') return 'Inspection complete; results saved locally';
  if (progress.phase === 'completed_with_partial_failures') {
    return 'Inspection complete with partial evidence; results saved locally';
  }
  if (progress.phase === 'cancelled') return 'Inspection cancelled; completed evidence preserved';
  if (progress.phase === 'failed') return 'Inspection stopped; previous saved data preserved';
  if (currentComponentName) return `Checking ${currentComponentName}`;
  return 'Preparing shared read-only evidence';
}

export function dashboardStatement(platform: PlatformDashboard): string {
  if (!platform.snapshot) return 'I have not inspected this PC yet.';
  if (platform.driftCount > 0) {
    return `I found ${platform.driftCount} setting${platform.driftCount === 1 ? '' : 's'} that changed since an earlier inspection.`;
  }
  if (platform.permissionLimitedCount > 0 || platform.unknownCount > 0) {
    return 'I completed the inspection, with some results that remain unknown or permission limited.';
  }
  return 'I found no unreviewed configuration changes in the saved inspection history.';
}

export function previewIsNonExecutable(preview: Record<string, unknown>): boolean {
  return preview.executorEnabled === false && typeof preview.cannotExecute === 'string';
}

function words(value: string): string {
  return value
    .replaceAll('_', ' ')
    .replaceAll(/([a-z])([A-Z])/g, '$1 $2')
    .toLowerCase();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}
