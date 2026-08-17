# Windows safety model

Every real operation follows this lifecycle without skipped technical stages:

`Inspect → Determine compatibility → Explain impact → Deliberate Apply → Capture and persist original state → Recheck → Apply through a fixed supported mechanism → Capture actual state → Verify directly and through the matching detector → Persist the receipt → Offer conflict-safe rollback → Verify rollback`

Optional Windows features, Windows capabilities, services, AppX/MSIX packages, classic applications, registry-backed settings, policies, scheduled tasks, startup entries, shell integrations, and drivers/devices are distinct mechanisms. They must not be represented as one generic module operation.

Deslopper forbids deleting files as uninstall, deleting registry keys without prior-state capture, service changes sourced only from internet lists, wildcard AppX removal without dependency analysis, assuming one Windows build represents all builds, silent application, claiming removal when only a surface was hidden, calling undocumented hacks supported, or bypassing anti-cheat, security controls, licensing, or Windows integrity protections.

Inspection, applicability, desired-state selection, history, drift, and preview planning remain available for the 21-component catalogue. Owner Mode M4 retains the six M1-M3 setting lifecycles and adds exact current-user package removal for Consumer Copilot, Phone Link, Clipchamp, and Solitaire. Package state is not represented as a DWORD. Each operation resolves one internally owned exact name, captures target/dependency/current-user inventory, uses native current-user deployment, verifies direct absence and the matching detector, and treats additional disappearance as Needs Attention. Provisioning and other users remain observational. Restore exists only when exact staged/provisioned identity and dependencies make registration deterministic; otherwise reinstall is required. Every other component remains inspection-only. The four M3 cleanup operations still require an exact DWORD 0/1 preference, refuse configured matching Cloud Content policy, and treat a missing or non-binary representation as unsupported. Lock-screen suggestions remain inspection-only because their broader Spotlight policy does not prove exact detector/handler parity.

The internal mutation-alpha harness retains three closed taskbar handlers for
engineering validation; its Show Desktop operation remains non-productized. Cancellation is
allowed before mutation; an atomic write, verification, or rollback is never
blindly interrupted. Automatic drift repair remains prohibited. Failed
verification may trigger an immediate exact rollback only when the attempted
state is directly observed and safely attributable to the current transaction.
