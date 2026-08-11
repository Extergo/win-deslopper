import { describe, expect, it } from 'vitest';

import type { DetectionObservation, PlatformDashboard, ProductComponent } from './backend';
import {
  dashboardStatement,
  filterComponents,
  inspectionFreshness,
  inspectionProgressDetail,
  observationStatus,
  onboardingPrinciples,
  previewIsNonExecutable
} from './product-ui';

const component: ProductComponent = {
  componentId: 'consumer_copilot',
  name: 'Consumer Copilot',
  category: 'AI',
  purpose: 'Observe the optional package.',
  benefit: 'Understand the current state.',
  support: 'Support',
  risk: 'Low',
  configuration: 'Exact package registration',
  restart: 'None',
  rollback: 'Future reviewed source required',
  privileges: 'User',
  gamingNotes: 'None',
  enterpriseNotes: 'Policy remains authoritative',
  documentation: 'Microsoft Learn',
  isPackage: true
};

function observation(status = 'successful'): DetectionObservation {
  return {
    componentId: component.componentId,
    current: { Package: { current_user: 'present', provisioned: 'unknown' } },
    authority: 'WindowsServicing',
    authorityAttribution: {
      authority: 'WindowsServicing',
      confidence: 'Moderate',
      exactSourceProven: false,
      evidence: [],
      alternatives: []
    },
    applicable: true,
    applicability: { status: 'applicable', reason: 'Supported', representation: '', source: '' },
    evidence: [],
    detectedAt: '1',
    warnings: [],
    error: null,
    policyState: null,
    preferenceState: null,
    provisioningState: null,
    packageIdentities: [],
    packageCompleteness: 'permission_limited',
    detectorStatus: status,
    packages: [],
    controlPrecedence: {
      documentedDefault: null,
      userPreference: null,
      localPolicy: null,
      domainPolicy: null,
      mdmPolicy: null,
      effectiveState: null,
      conflictingEvidence: false
    }
  };
}

describe('Owner Mode presentation rules', () => {
  it('explains the explicit Widgets and honest unknown boundary during onboarding', () => {
    expect(onboardingPrinciples.join(' ')).toMatch(/Widgets visibility.*explicit click/i);
    expect(onboardingPrinciples.join(' ')).toMatch(/Unknown does not mean broken/i);
    expect(onboardingPrinciples.join(' ')).toMatch(/redacted/i);
  });

  it('names permission-limited and failed states without treating them as absence', () => {
    expect(observationStatus(observation())).toBe('Permission limited');
    expect(observationStatus(observation('failed'))).toBe('Failed inspection');
  });

  it('filters the real catalogue by evidence-backed package state', () => {
    expect(filterComponents([component], [observation()], [], [], 'package', '')).toHaveLength(1);
    expect(
      filterComponents([component], [observation()], [], [], 'permission_limited', '')
    ).toHaveLength(1);
    expect(filterComponents([component], [observation()], [], [], 'failed', '')).toHaveLength(0);
  });

  it('uses plain-language freshness instead of a fabricated health percentage', () => {
    expect(inspectionFreshness('1000', 1000 + 5 * 60_000)).toBe('Fresh');
    expect(inspectionFreshness('1000', 1000 + 8 * 24 * 60 * 60_000)).toMatch(/Stale/);
  });

  it('never labels a terminal inspection as preparing', () => {
    const detail = inspectionProgressDetail({
      inspectionId: 'inspection-1',
      phase: 'completed_with_partial_failures',
      completedWork: 20,
      totalWork: 20,
      currentComponent: null,
      detectorResultStatus: null,
      warningCount: 2,
      errorCount: 0,
      cancellationAvailable: false,
      status: 'Inspection completed with partial evidence',
      updatedAt: '1'
    });
    expect(detail).toMatch(/complete with partial evidence/i);
    expect(detail).not.toMatch(/preparing/i);
  });

  it('summarises uncertainty without fear-based wording', () => {
    const platform = {
      snapshot: { timestamp: '1' },
      driftCount: 0,
      permissionLimitedCount: 2,
      unknownCount: 1
    } as PlatformDashboard;
    expect(dashboardStatement(platform)).toMatch(/unknown or permission limited/);
    expect(dashboardStatement(platform)).not.toMatch(/critical|broken|slow|bloated/i);
  });

  it('requires previews to carry a hard non-executable state', () => {
    expect(
      previewIsNonExecutable({
        executorEnabled: false,
        cannotExecute: 'Automatic restoration is unavailable in Product Alpha.'
      })
    ).toBe(true);
  });
});
