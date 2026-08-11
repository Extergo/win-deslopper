# Owner Mode M4 AppX decision

**Decision:** accept four closed current-user package operations for Deslopper
`0.4.0`, with runtime refusal whenever exact identity, package health, or
removability cannot be proved.

## Candidate audit

| Component | Closed operation | Exact package name | Identity-set result | M4 result |
|---|---|---|---|---|
| Consumer Copilot | `remove_consumer_copilot_current_user` | `Microsoft.Copilot` | One canonical identity; no wildcard, legacy, successor, or companion rule | Accepted with runtime guards |
| Phone Link | `remove_phone_link_current_user` | `Microsoft.YourPhone` | One canonical identity; no wildcard, legacy, successor, or companion rule | Accepted with runtime guards |
| Clipchamp | `remove_clipchamp_current_user` | `Clipchamp.Clipchamp` | One canonical identity; no wildcard, legacy, successor, or companion rule | Accepted with runtime guards and prominent app-data warning |
| Solitaire | `remove_solitaire_current_user` | `Microsoft.MicrosoftSolitaireCollection` | One canonical identity; no wildcard, legacy, successor, or companion rule | Accepted with runtime guards and prominent app-data warning |

No candidate in the four-item M4 audit was rejected at operation-definition
time. Acceptance does not claim that every installation is removable. The
operation becomes `REMOVE_UNAVAILABLE` when direct inventory and the detector
do not resolve exactly one identical current-user package, or when Windows
reports a framework, resource package, bundle, non-removable state, unhealthy
state, or multiple current-user versions. Phone Link can therefore be present
but unavailable on a Windows build that protects its exact package.

The following package-backed components were deliberately rejected from M4:

- Microsoft 365 Copilot / Office Hub: known legacy/successor identities and
  broader product semantics need a later migration-aware design.
- Teams variants: legacy/successor and personal/work product boundaries are not
  narrow enough for this milestone.
- News and Weather: the detector intentionally represents two required
  companion identities, not one removal target.
- OneDrive: account, sync-root, KFM, filesystem, and executable semantics are
  not AppX-current-user-removal semantics.
- Store, App Installer, Web Experience Pack, Widgets, frameworks, runtimes,
  codecs, and servicing packages: removing them would violate the product and
  dependency boundary.

## Deployment implementation

M4 uses the native `Windows.Management.Deployment.PackageManager` projection
from `windows` 0.62.2. `FindPackagesForUser` with an empty user SID explicitly
inventories current-user registrations, `RemovePackageAsync` removes the exact
`PackageFullName` for the current user, and
`RegisterPackageByFullNameAsync` is used only for a transaction classified
`RESTORE_AVAILABLE`. The parameterless, cross-user `FindPackages` overload is
not used. No PowerShell deployment command was added.

This API was selected because it gives Deslopper a bounded Windows-supported
deployment surface without a shell, script, command string, wildcard, or
frontend-controlled package identity. The selected remove overload has no
all-user flag. The implementation does not call broad removal options,
provisioning APIs, image servicing, WindowsApps filesystem operations, or Store
acquisition.

References:

- <https://learn.microsoft.com/uwp/api/windows.management.deployment.packagemanager.removepackageasync>
- <https://learn.microsoft.com/uwp/api/windows.management.deployment.packagemanager.findpackagesforuser>
- <https://learn.microsoft.com/uwp/api/windows.management.deployment.packagemanager.registerpackagebyfullnameasync>
- <https://microsoft.github.io/windows-docs-rs/doc/windows/Management/Deployment/struct.PackageManager.html>

## First-class package state

Package transactions are separate from the established DWORD transaction
model. A package capture records:

- component and closed operation IDs;
- exact Name, PackageFullName, PackageFamilyName, version, architecture,
  publisher, publisher ID, and optional resource ID;
- current-user registration and a sorted current-user PackageFullName
  inventory;
- install-location presence, package health/status, framework/resource/bundle/
  optional flags;
- exact dependency names, full names, family names, versions, architectures,
  and framework/resource flags;
- detector current-user and other-user registration evidence;
- observational provisioning state and exact provisioned PackageFullName when
  available;
- deployment handler/version and restore capability; and
- capture time and integrity hash.

The install path itself is not persisted. Only its presence is needed for the
restore decision, avoiding unnecessary profile/path data.

## Dependency and verification model

Before removal, the native API captures the complete current-user package
inventory and the target's direct dependencies. Immediately before deployment,
the broker repeats inventory and refuses if the target or inventory changed.
After deployment, it captures actual inventory and computes all disappeared and
appeared PackageFullNames.

Only the exact target PackageFullName is expected to disappear. Windows may
remove an unused dependency as deployment behavior, but Deslopper does not
silently bless that outcome: every additional disappearance is recorded as
`unexpected_collateral_change` and the transaction becomes Needs Attention.
The matching detector must also report current-user absence. Provisioning must
remain unchanged.

## Restore classification and version drift

`RESTORE_AVAILABLE` requires all of the following captured evidence:

- the exact current PackageFullName is also the exact provisioned PackageName;
- Windows reports provisioning present;
- install location is present; and
- every direct dependency has a non-empty full identity present in the
  current-user inventory.

Otherwise an otherwise safe removal is `REINSTALL_REQUIRED`. No Restore button
is shown, and reinstall may require Microsoft Store or Windows servicing.
Thus none of the four operation definitions promises Restore unconditionally;
all four can support deterministic Restore only when their machine-specific
pre-state satisfies the rule above.

Restore is transaction-bound and owner-scope-bound. A conflicting package in
the same family is refused. Exact-version registration is recorded as exact
restore. If Windows has already registered or registers a strictly newer
version in the same family, the result is explicitly
`restored_newer_version`; it is never described as exact restoration.

## Current-user and data boundary

Removal means only “remove from this account.” Provisioning and other-user
evidence are observational. M4 does not elevate, remove provisioning, modify
the Windows image, target another user, alter WindowsApps ACLs, delete package
directories, modify HKLM, stop services, or weaken Store/servicing.

The UI states that removal may remove local app data. It does not promise to
preserve Clipchamp projects, game state, Phone Link state, Copilot state, or
other package data. One deliberate button click authorizes the fixed operation;
there is no phrase, modal ceremony, or generic package input.

## Implementation-time mutation statement

All automated package tests use a fake deployment backend. Build, test, lint,
and packaging commands do not invoke a package operation. No live AppX/MSIX
package was removed or registered while implementing M4.
