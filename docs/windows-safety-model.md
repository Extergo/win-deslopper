# Windows safety model

Every future real operation follows this lifecycle without skipped stages:

`Inspect → Determine compatibility → Create a change plan → Explain impact → Capture original state → Request explicit user approval → Apply through a supported mechanism → Verify the result → Persist an operation receipt → Offer rollback → Verify rollback`

Optional Windows features, Windows capabilities, services, AppX/MSIX packages, classic applications, registry-backed settings, policies, scheduled tasks, startup entries, shell integrations, and drivers/devices are distinct mechanisms. They must not be represented as one generic module operation.

Deslopper forbids deleting files as uninstall, deleting registry keys without prior-state capture, service changes sourced only from internet lists, wildcard AppX removal without dependency analysis, assuming one Windows build represents all builds, silent application, claiming removal when only a surface was hidden, calling undocumented hacks supported, or bypassing anti-cheat, security controls, licensing, or Windows integrity protections.

In `ui-preview`, all component data, detection labels, compatibility values, plan actions, and review output are mock-only. No lifecycle stage invokes Windows.

