# Owner Mode M4 current-user AppX smoke test

This is a manual physical-validation protocol for a disposable or low-risk app
on the owner Windows 11 x64 machine. It is not an implementation test and must
not be automated. Implementation and CI must never execute package deployment.

## Stop conditions

Stop without removing anything if Deslopper is not the intended `0.4.0` build,
the process is elevated, the Windows account/machine is wrong, package evidence
is incomplete, exact identities disagree, multiple versions match, Windows
reports protected/framework/resource/bundle state, important local app data may
exist, or recovery expectations are unclear.

Never test first with valuable Clipchamp projects, Solitaire progress, Phone
Link state, Copilot state, or other meaningful local data. Do not install an app
merely to remove it. Do not remove provisioning or another user's registration.

## A. Read-only inventory first

1. Record Deslopper product version, source commit, installer SHA-256, installed
   executable path, Windows edition/build/architecture, current-user execution,
   and confirmation that no UAC/elevation is active.
2. Run a fresh Deslopper inspection. Do not click Remove.
3. For every accepted candidate, record Deslopper and independent read-only
   evidence:

   | Component | Exact Name |
   |---|---|
   | Consumer Copilot | `Microsoft.Copilot` |
   | Phone Link | `Microsoft.YourPhone` |
   | Clipchamp | `Clipchamp.Clipchamp` |
   | Solitaire | `Microsoft.MicrosoftSolitaireCollection` |

4. Capture Name, PackageFullName, PackageFamilyName, Version, Architecture,
   Publisher/PublisherId, ResourceId if present, InstallLocation presence,
   current-user registration, other-user evidence, provisioned status and exact
   provisioned PackageName, framework/resource/bundle/non-removable flags,
   health/status, and every direct dependency full identity.
5. Confirm independent inventory uses exact names only. Do not use `*Copilot*`,
   `*Phone*`, or another wildcard.
6. Confirm Deslopper's current-user result and independent inventory agree.

Suggested independent **read-only** commands may use exact filters and
`Get-AppxPackage`, `Get-AppxPackage -AllUsers`, and
`Get-AppxProvisionedPackage -Online`. Do not run any Remove/Add command during
inventory.

## B. Restore assessment

Record Deslopper's classification before choosing a target:

- `Restore available` is valid only when the captured current PackageFullName
  exactly equals the provisioned PackageName, install location is present, and
  complete dependency full identities are captured and present.
- `Reinstall required` means the first test stops after verified removal and
  relaunch. Do not reinstall automatically.
- `Remove unavailable` means do not attempt removal.

If Deslopper says Restore available, write down the exact staged/provisioned and
dependency evidence that makes `RegisterPackageByFullNameAsync` deterministic.

## C. Choose one low-risk installed package

Choose only one installed, healthy, removable candidate with no important local
data. Record why it is lower risk than the other installed candidates. Physical
validation proceeds package-by-package; passing one package does not validate
the other three.

Before clicking, preserve:

- the complete current-user PackageFullName inventory or a cryptographic hash
  plus the target/dependency rows;
- provisioning evidence;
- Deslopper actionability and restore classification; and
- a recovery note appropriate to the classification.

## D. Remove once

Click **Remove from this account** exactly once. Do not double-click. Expected
progress is Checking → Removing → Verifying → Removed.

Record the exact transaction ID, result, target identity/version, restore
classification, detector verification, disappeared package list, provisioning
comparison, and any Windows deployment error.

## E. Independent verification

After Deslopper reports a result:

1. Verify the exact target PackageFullName is absent for the current user.
2. Verify Deslopper's matching detector reports current-user absence.
3. Verify provisioning is byte-for-byte/field-for-field unchanged.
4. Compare current-user package inventory and prove no unrelated registration
   disappeared. Any additional disappearance is a failed/Needs Attention test,
   even if it is an unused dependency.
5. Verify other-user evidence was not deliberately changed.
6. Verify normal history shows app, timestamp, exact identity/version, restore
   capability, and result.

Do not describe the app as completely removed from Windows.

## F. Process exit and relaunch

Close Deslopper completely. Confirm no Deslopper process remains. Relaunch the
installed executable and confirm the package transaction/history survives.

## G. Restore only when available

If the transaction says `Restore available`, click **Restore** once. Verify:

- the current-user package family is registered again;
- Deslopper's detector agrees;
- exact PackageFullName/version restoration is recorded as exact, or a newer
  same-family version is explicitly recorded as newer-version restoration;
- provisioning remains unchanged; and
- no unexpected package registration appeared or disappeared.

If the transaction says `Reinstall required`, do not reinstall during this
first test. Record the verified removal and stop. Store acquisition, downloads,
and arbitrary package installation are outside M4.

## Evidence disposition

Mark the tested operation PASS only if exact detection, durable pre-state,
current-user-only removal, direct and detector verification, collateral check,
unchanged provisioning, process exit/relaunch persistence, and (when offered)
Restore verification all pass. Otherwise record the exact failure category and
leave the transaction Needs Attention; do not elevate or weaken Windows
protections.

