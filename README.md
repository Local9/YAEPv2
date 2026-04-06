# YAEP - Yet Another EVE Preview

YAEP helps you manage multiple EVE Online clients on Windows.
It is built to make switching between clients faster and easier during play.

## Why this exists

YAEP is a personal quality-of-life project for multi-boxing in EVE Online.
Other tools exist, but this one focuses on the workflow I wanted for daily use.

Feature suggestions are welcome. Please note this is still primarily a personal project, so suggestions are reviewed before being accepted.

## What YAEP does

- Shows live preview thumbnails for your open EVE clients.
- Lets you click a thumbnail to bring that client to the front.
- Includes widget-style overlays you can place where they work best for your layout.
- Supports hotkeys so you can switch clients quickly.
- Supports profiles, so you can organize client setups your way.
- Saves your settings between sessions.

## Additional features

- Profiles with hotkey switching
  - Track different executables (default is `exefile` for EVE Online)
  - Set different hotkeys for different groups
- Groups with a GUI to manage clients and shortcuts
  - Group thumbnails and assign shortcuts per group
- Widget overlay controls
  - Move and place overlays to suit your setup
  - Resize thumbnails quickly using mouse + hotkeys
- SQLite settings storage
  - No manual `.json` config editing
- Grid tooling with multi-monitor support
  - Arrange thumbnails in a grid automatically
  - Select groups so only those thumbnails are affected
- Mumble Links widget
- Character and user settings management
  - Inspired by [EANM](https://github.com/FontaineRiant/EANM) to reduce folder hunting

## Who this is for

YAEP is for EVE players who run more than one client and want faster window switching.

## Platform

- Windows 10/11

## Notes

- YAEP only activates and focuses client windows.
- It does not control gameplay actions.

## For developers

If you are working on the app locally:

- Install dependencies: `pnpm install`
- Run the app in development: `pnpm tauri:dev`

## Typical hotkeys

| Hotkey                        | Description                |
| ----------------------------- | -------------------------- |
| Right Click on Thumbnail      | Move thumbnail             |
| CTRL+Right Click on Thumbnail | Move all thumbnails        |
| CTRL+Wheel Up                 | Increase size of thumbnail |
| CTRL+Wheel Down               | Decrease size of thumbnail |

# CCP Copyright Notice
EVE Online and the EVE logo are the registered trademarks of CCP hf. All rights are reserved worldwide. All other trademarks are the property of their respective owners. EVE Online, the EVE logo, EVE and all associated logos and designs are the intellectual property of CCP hf. All artwork, screenshots, characters, vehicles, storylines, world facts or other recognizable features of the intellectual property relating to these trademarks are likewise the intellectual property of CCP hf. CCP hf. has granted permission to EVE University to use EVE Online and all associated logos and designs for promotional and information purposes on its website but does not endorse, and is not in any way affiliated with, EVE University. CCP is in no way responsible for the content on or functioning of this software, nor can it be liable for any damage arising from the use of this software.