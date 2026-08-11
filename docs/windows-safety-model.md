# Windows safety model

Every real operation follows this lifecycle without skipped technical stages:

`Inspect → Determine compatibility → Explain impact → Deliberate Apply → Capture and persist original state → Recheck → Apply through a fixed supported mechanism → Capture actual state → Verify directly and through the matching detector → Persist the receipt → Offer conflict-safe rollback → Verify rollback`

Optional Windows features, Windows capabilities, services, AppX/MSIX packages, classic applications, registry-backed settings, policies, scheduled tasks, startup entries, shell integrations, and drivers/devices are distinct mechanisms. They must not be represented as one generic module operation.

Deslopper forbids deleting files as uninstall, deleting registry keys without prior-state capture, service changes sourced only from internet lists, wildcard AppX removal without dependency analysis, assuming one Windows build represents all builds, silent application, claiming removal when only a surface was hidden, calling undocumented hacks supported, or bypassing anti-cheat, security controls, licensing, or Windows integrity protections.

Inspection, applicability, desired-state selection, history, drift, and preview planning remain available for the 20-component catalogue. Owner Mode M1 completes the lifecycle only for `TaskbarWidgets`; every other component remains inspection-only.

The internal mutation-alpha harness retains three closed taskbar handlers for
engineering validation, but only Widgets is product-authorized. Cancellation is
allowed before mutation; an atomic write, verification, or rollback is never
blindly interrupted. Automatic drift repair remains prohibited. Failed
verification may trigger an immediate exact rollback only when the attempted
state is directly observed and safely attributable to the current transaction.
