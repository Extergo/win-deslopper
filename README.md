# Deslopper

**Windows, without the slop.**

Deslopper is a lightweight Windows utility that quietly lives in your system tray and keeps Windows clean from the things you never asked for.

Think of it as **Ninite for Windows itself**.

Instead of manually running dozens of PowerShell scripts after every fresh install—or every time Microsoft decides to put everything back—Deslopper continuously keeps your system the way *you* configured it.

No registry hunting. No copy-pasting random GitHub scripts. No wondering if the next Windows update will undo everything.

---

## Why?

Modern Windows ships with features that many users simply don't want:

* OneDrive integration
* Copilot
* Start Menu Bing search
* Ads & recommendations
* Edge promotions & previews
* Maps
* Consumer experiences
* Telemetry (where safely configurable)
* Various Windows "helpful" experiences

Today your options are usually:

* Run a huge PowerShell script from GitHub
* Spend hours clicking through Group Policy and Registry Editor
* Re-run everything after every major Windows update

That isn't a great experience.

Deslopper automates it.

---

## Gaming First

Our first priority is **gamers**.

Many existing debloat scripts aggressively remove Windows components, often breaking:

* Anti-cheat systems
* Xbox services
* Game Pass
* Windows Gaming features
* Store dependencies

Deslopper takes a different approach.

Every tweak is categorized and tested, allowing you to remove the things you don't want **without sacrificing compatibility**.

The default configuration is designed to remain friendly with modern anti-cheat systems and Xbox gaming components.

---

## You're Still In Control

Deslopper isn't about forcing opinions.

Every component can be enabled or disabled individually.

Want to remove OneDrive but keep Edge?

Go ahead.

Want Copilot gone but keep telemetry?

That's fine too.

You choose what your Windows installation looks like.

---

## Survives Windows Updates

The biggest problem with existing debloat tools isn't removing the bloat.

It's **keeping it removed**.

Major Windows updates frequently restore:

* Copilot
* OneDrive
* Bing Search
* Recommended content
* Consumer experiences
* Various Microsoft defaults

Deslopper continuously monitors your configuration and automatically reapplies your chosen settings whenever Windows resets them.

Configure it once.

Forget about it.

---

## Built Properly

This isn't another collection of PowerShell scripts wrapped in a GUI.

Deslopper is built as a native Windows application from the ground up.

* 🦀 Written entirely in Rust
* ⚡ Native performance
* 🧠 Low memory footprint
* 🔒 Memory safe
* 🎮 Designed around gaming compatibility
* 🪶 Runs quietly in the system tray

---

## Open Source

Deslopper is developed entirely in the open.

We're building it publicly with contributors from across the systems programming community, including people experienced in:

* Operating systems
* Windows internals
* Systems programming
* Rust
* Native desktop development

Every change is reviewable.

Every decision is transparent.

---

## Philosophy

Windows should work for **you**—not the other way around.

Deslopper doesn't try to replace Windows.

It simply removes the noise.

Install Windows.

Install Deslopper.

Choose what you want.

Get back to gaming, developing, or working.

No scripts.

No registry hacks.

No reinstall rituals after every Windows update.

Just Windows.. without the slop.
