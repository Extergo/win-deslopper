# Dependency review: Windows AppX projection

**Decision:** approve direct, Windows-target-only dependencies `windows`
`0.62.2` and `windows-collections` `0.3.2` for Owner Mode M4.

## Purpose

Deslopper needs a strongly typed projection of
`Windows.Management.Deployment.PackageManager` for current-user package
inventory, exact current-user removal, and bounded local registration. The
collections crate constructs the typed `IIterable<HSTRING>` dependency list
required by `RegisterPackageByFullNameAsync`.

## Why existing dependencies are insufficient

`winreg` only implements the established DWORD setting store. Tauri does not
project WinRT deployment APIs. The existing PowerShell runner is deliberately
read-only; extending it to deployment would add shell parsing and command
surface that native WinRT avoids.

## Maintenance and provenance

Both crates are official Rust for Windows project artifacts maintained under
Microsoft's `microsoft/windows-rs` project. Versions are exact in `Cargo.lock`.
The API surface used by Deslopper is limited by Cargo features to
ApplicationModel, Foundation/collections, Management.Deployment, Storage
metadata, and System architecture types.

## License

The crates use the MIT or Apache-2.0 license model published by the Rust for
Windows project and are compatible with this repository's dependency policy.
Release review should continue to verify lockfile provenance and license
metadata.

## Windows support and behavior

The dependency is compiled only for the Windows target. The selected API is a
Windows-supported current-user deployment API available on the Windows 11
builds Deslopper supports. Package deployment can trigger Windows servicing
behavior, including removal of an unneeded dependency; Deslopper therefore
captures complete current-user inventory before and after and treats any
additional disappearance as Needs Attention.

## Security and supply chain

The projection performs in-process WinRT calls. It adds no subprocess, shell,
network client, updater, Store scraper, download, telemetry, or elevation path.
Generated projection code contains the FFI/unsafe implementation maintained by
windows-rs; Deslopper's package module contains no handwritten unsafe block.
The runtime receives only broker-resolved exact PackageFullNames after a closed
operation-to-name check.

## Binary size

Feature selection is narrow and target-specific. The normal release/NSIS size
is measured by the release gate; no separate runtime or DLL is bundled by this
dependency.

## External/network behavior

`FindPackagesForUser` with the empty current-user SID, `RemovePackageAsync`, and
`RegisterPackageByFullNameAsync` call local Windows deployment services.
Deslopper does not request cross-user package inventory, Store acquisition, or
network content. Windows may perform its ordinary independent servicing, but
M4 neither starts nor disables servicing.

## Alternatives considered

- Fixed PowerShell `Remove-AppxPackage`: rejected because native WinRT is
  practical and avoids introducing a deployment shell surface.
- DISM/provisioning APIs: rejected because M4 is current-user-only.
- Manual WindowsApps deletion/ACL changes: rejected as unsupported and unsafe.
- Store download or package sideloading: rejected; M4 has no acquisition path.

## Removal plan

If windows-rs becomes unmaintained, incompatible, or materially broadens the
binary/security footprint, disable package action registration, retain
read-only package inspection and durable history, remove the two direct
dependencies, and do not substitute a generic script runner.
