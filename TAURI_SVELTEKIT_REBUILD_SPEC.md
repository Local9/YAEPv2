# YAEP (Yet Another EVE Preview) -> Tauri + SvelteKit Reimplementation Spec

## 1. Product Summary
YAEP is a Windows desktop utility that:
- Scans running EVE Online client processes (configurable per "Profile").
- Creates always-on-top "thumbnail windows" that show live previews of each client via Windows DWM live thumbnails.
- Adds an overlay per thumbnail (focus border + optional title label).
- Tracks keyboard-focus to highlight the currently focused thumbnail process (with a QoL behavior: keep the last focused border when focus moves to non-YAEP apps).
- Provides global hotkeys to:
  - Switch active Profiles.
  - Cycle through client "ClientGroups" (next/previous) by activating the corresponding EVE client windows.
  - Open configured Mumble links.
- Offers a UI to manage:
  - Profiles + per-profile hotkeys
  - Which process names to preview per profile
  - Thumbnail sizing/border/title overlay behavior (default + per-window-title overrides)
  - Client grouping (members are window titles; ordering and cycle hotkeys)
  - Grid layout automation (bulk reposition/resize thumbnails)
  - Mumble links + grouping into server-groups + a draggable overlay window
  - A "drawer" overlay UI (slides in from screen edge, showing Mumble links for a selected server-group)
  - EVE Online profile copying utilities (reads/writes local EVE profile folders; uses ESI for character names)

Windows-only behavior is core because live thumbnails are implemented via Win32 DWM.

## 2. User-Facing Features (must reproduce)
1. **Single-instance desktop app**: prevents multiple app instances using a named mutex.
2. **Tray icon + context menu**
   - "Show" restores main window
   - Per-profile menu items: mark active profile and activate it when clicked
   - "Exit" stops services and shuts down app
3. **Main window navigation pages**
   - Profiles
   - Thumbnail Settings
   - Client Grouping
   - Grid Layout
   - Process Management
   - Mumble Links
   - Eve Online Profiles
   - Settings (theme, thumbnail dragging enabled, start hidden toggle stored in DB)
4. **Thumbnail live preview windows**
   - One thumbnail window per tracked EVE client process (keyed by PID)
   - Always on top
   - No Alt+Tab presence (WS_EX_TOOLWINDOW)
5. **Overlay behavior**
   - Focus border visible only for the focused thumbnail process
   - Optional title overlay (character name) in top-left of each thumbnail
   - Overlay is click-through (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW)
6. **Thumbnail interactions**
   - **Right mouse drag** moves thumbnail window (if dragging enabled in DB)
   - **Ctrl+Right drag** performs group-drag: all thumbnails move relative to the dragged one
   - Saves new positions to DB on drag end
   - **Ctrl + mouse wheel** resizes thumbnail maintaining aspect ratio
   - **Hover** sets thumbnail opacity to 1.0 (otherwise uses configured opacity)
   - **Left click** activates the underlying EVE client window (no focus needed; overlay stays on top)
7. **Thumbnail creation rules**
   - Profile controls which process names are scanned
   - Window title filtering:
     - Exclude title exactly `EVE` (base window without character name)
     - Only include titles like `EVE - CharacterName`
   - Title changes can occur after startup; the app updates thumbnail window title/settings accordingly
8. **Focus tracking timer**
   - Runs frequently and identifies which tracked process matches the foreground window’s process
   - When a thumbnail gets focus: sets IsFocused true and clears previous
   - If a non-thumbnail app is focused: does NOT clear the last focused border (QoL)
9. **Profile switching**
   - Hotkey switches DB "IsActive" Profile
   - Thumbnail service pauses monitoring, updates existing thumbnails to new profile settings, resumes, then rescan
10. **Client Group cycling**
   - Each ClientGroup has:
     - ordered members (window titles)
     - forward hotkey + backward hotkey
   - On hotkey:
     - find currently active client title among group members
     - choose next/prev (wrap-around)
     - activate that EVE client window via Win32 SetForegroundWindow/SetFocus (+ restore if minimized)
11. **Grid Layout**
   - User selects:
     - cell width/height or aspect ratio
     - grid start X/Y
     - number of columns
     - optional monitor selection
     - optional "Only affect active thumbnails"
     - optional group filter
   - Generates a preview of new positions (ordering based on group display order + member display order when available)
   - Apply:
     - pauses thumbnail monitoring
     - saves updated thumbnail settings for each title
     - updates active thumbnails live
     - clamps thumbnails back into valid monitor bounds if needed
12. **Thumbnail settings**
   - Per-profile default config + per-window-title overrides in SQLite
   - Live preview while editing:
     - border color/thickness updates all relevant thumbnails
     - size/opacity updates thumbnails live
   - Debounced DB saving for default config (300ms)
   - Updating default config applies:
     - new width/height/opacity/border/showTitleOverlay to all thumbnail settings in the profile
     - preserves X/Y per-window position
13. **Process Management**
   - Per-profile list of process names to scan (ProcessesToPreview)
   - Removing a process also deletes thumbnail settings entries that match that process name by title heuristics
14. **Mumble Links**
   - Manage Mumble links:
     - Name, Mumble URL, display order
     - IsSelected controls whether they appear in the overlay window
     - Hotkey per link to open URL
   - Manage Mumble Server Groups:
     - links can belong to many groups (many-to-many)
     - group ordering
   - Mumble overlay window:
     - appears only when one or more links are selected
     - draggable via right mouse
     - auto height depends on number of selected links
     - top-most toggle stored in DB
     - combo box lists "unselected links" to add/trigger open immediately
15. **Drawer overlay**
   - Sliding drawer anchored to left/right edge of a chosen monitor
   - Shows links filtered by selected Mumble Server Group
   - Drawer window:
     - slides in/out by animating width
     - hover indicator triggers opening after delay
     - hides from Alt+Tab
     - does not slide out if dropdown is open
   - Drawer settings persisted in SQLite via AppSettings keys
16. **EVE Online profile tools**
   - Reads local `%LOCALAPPDATA%\CCP\EVE\*` server folders
   - Detects profiles:
     - settings folders named `settings_*`
     - "Default" if profile name is exactly `Default`
   - Can:
     - copy a whole profile folder to a new profile name (creates settings_<name>)
     - in a profile, copy character/user files:
       - core_char_* and core_user_* files
       - optionally fetch character names via ESI API
       - overwrites target files with source files
   - Prevents these actions if the EVE client is running (process name `exefile`)

## 3. Technical Architecture (original YAEP)
### 3.1 High-level runtime components
- **UI (Avalonia + MVVM + SukiUI)**
  - Pages and windows are bound to ViewModels
- **DatabaseService (SQLite)**
  - Owns schema creation + CRUD
  - Holds current active Profile + emits `ProfileChanged`
- **ThumbnailWindowService**
  - Maintains a dictionary of active thumbnail windows keyed by PID
  - Runs timers:
    - monitoring scan every **2000ms**
    - focus check every **100ms**
  - Uses desktop-window manager (DWM) to display live thumbnails
  - Updates overlays + caches thumbnail settings for fast UI operations
- **HotkeyService**
  - Creates a Win32 message-only window on a dedicated thread
  - Registers global hotkeys using `RegisterHotKey`
  - Also installs `WH_KEYBOARD_LL` keyboard hook to suppress interfering key input
- **DrawerWindowService**
  - Creates and animates drawer windows if enabled
  - Uses MonitorService for stable monitor identification
- **Platform service factory**
  - Chooses WindowsDesktopWindowManager / LinuxDesktopWindowManager (though this app is primarily Windows-focused)

### 3.2 Windows-specific native behavior (core requirement)
- **Live thumbnail rendering** uses:
  - `DwmRegisterThumbnail(destination, source)`
  - updates via `DwmUpdateThumbnailProperties`
  - opacity set by DWM thumbnail properties
- **Overlay click-through** uses:
  - `GetWindowLong/SetWindowLong` with WS_EX flags:
    - WS_EX_LAYERED
    - WS_EX_TRANSPARENT
    - WS_EX_TOOLWINDOW
- **Window activation** uses:
  - `SetForegroundWindow`, `SetFocus`
  - checks minimized state via window style and restores with `ShowWindowAsync` or `SetWindowPlacement`

## 4. Constants and Behavior Rules (must match)
### 4.1 Thumbnail layout bounds & defaults
- Min size: width `192`, height `108`
- Max size: width `960`, height `540`
- Default position: X `100`, Y `100`
- Default thumbnail: width `400`, height `300`
- Default opacity: `0.75`
- Focus border default:
  - Color `#0078D4`
  - Thickness `3`

### 4.2 Position validity thresholds (used during saves and clamping)
- Low threshold: `-10_000`
- High threshold: `31_000`

### 4.3 Title detection
- Base EVE title (no character): `EVE`
- Character title prefix: `EVE - `
- WindowHelper filtering:
  - If title is exactly `EVE`: exclude from thumbnail operations
  - Otherwise include

### 4.4 Timers / debounce
- ThumbnailWindowService monitoring scan: **2000ms**
- ThumbnailWindowService focus check: **100ms**
- ThumbnailWindow position tracker: **500ms**
- ThumbnailWindow overlay sync delay: **100ms**
- ThumbnailSettingsViewModel default config debounce: **300ms**
- GridLayoutViewModel refresh interval: **1000ms**
- GridLayout apply pauses monitoring, then clamps boundaries
- MumbleLinksWindow position save debounce: **500ms**
- DrawerIndicatorWindow hover delay: **300ms**
- Update check: starts after **2000ms**

## 5. SQLite Data Model (exact tables + keys)
Database file: `settings.db` stored in the application base directory (in YAEP).

### 5.1 Schema created on startup
```sql
CREATE TABLE IF NOT EXISTS Profile (
  Id INTEGER PRIMARY KEY AUTOINCREMENT,
  Name TEXT NOT NULL UNIQUE,
  DeletedAt TEXT NULL,
  IsActive INTEGER NOT NULL DEFAULT 0
);
-- columns added/migrated:
-- DeletedAt TEXT NULL
-- IsActive INTEGER NOT NULL DEFAULT 0
-- SwitchHotkey TEXT NOT NULL DEFAULT ''

CREATE TABLE IF NOT EXISTS ProcessesToPreview (
  ProfileId INTEGER NOT NULL,
  ProcessName TEXT NOT NULL,
  PRIMARY KEY (ProfileId, ProcessName),
  FOREIGN KEY (ProfileId) REFERENCES Profile(Id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ThumbnailDefaultConfig (
  ProfileId INTEGER NOT NULL PRIMARY KEY,
  Width INTEGER NOT NULL DEFAULT <default>,
  Height INTEGER NOT NULL DEFAULT <default>,
  X INTEGER NOT NULL DEFAULT <default>,
  Y INTEGER NOT NULL DEFAULT <default>,
  Opacity REAL NOT NULL DEFAULT <default>,
  FocusBorderColor TEXT NOT NULL DEFAULT '<default>',
  FocusBorderThickness INTEGER NOT NULL DEFAULT <default>,
  ShowTitleOverlay INTEGER NOT NULL DEFAULT 1,
  FOREIGN KEY (ProfileId) REFERENCES Profile(Id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ThumbnailSettings (
  ProfileId INTEGER NOT NULL,
  WindowTitle TEXT NOT NULL,
  Width INTEGER NOT NULL,
  Height INTEGER NOT NULL,
  X INTEGER NOT NULL,
  Y INTEGER NOT NULL,
  Opacity REAL NOT NULL,
  FocusBorderColor TEXT NOT NULL DEFAULT '<default>',
  FocusBorderThickness INTEGER NOT NULL DEFAULT <default>,
  ShowTitleOverlay INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (ProfileId, WindowTitle),
  FOREIGN KEY (ProfileId) REFERENCES Profile(Id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS AppSettings (
  Key TEXT NOT NULL PRIMARY KEY,
  Value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS MumbleServerGroups (
  Id INTEGER PRIMARY KEY AUTOINCREMENT,
  Name TEXT NOT NULL,
  DisplayOrder INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS MumbleLinks (
  Id INTEGER PRIMARY KEY AUTOINCREMENT,
  Name TEXT NOT NULL,
  Url TEXT NOT NULL,
  DisplayOrder INTEGER NOT NULL DEFAULT 0,
  IsSelected INTEGER NOT NULL DEFAULT 0
);
-- columns added/migrated:
-- ServerGroupId INTEGER NULL
-- Hotkey TEXT NOT NULL DEFAULT ''

CREATE TABLE IF NOT EXISTS MumbleLinkGroups (
  LinkId INTEGER NOT NULL,
  GroupId INTEGER NOT NULL,
  PRIMARY KEY (LinkId, GroupId),
  FOREIGN KEY (LinkId) REFERENCES MumbleLinks(Id) ON DELETE CASCADE,
  FOREIGN KEY (GroupId) REFERENCES MumbleServerGroups(Id) ON DELETE CASCADE
);
-- one-time migration:
-- INSERT OR IGNORE INTO MumbleLinkGroups (LinkId, GroupId)
--   SELECT Id, ServerGroupId FROM MumbleLinks WHERE ServerGroupId IS NOT NULL

CREATE TABLE IF NOT EXISTS MumbleLinksOverlaySettings (
  Id INTEGER PRIMARY KEY AUTOINCREMENT,
  AlwaysOnTop INTEGER NOT NULL DEFAULT 1,
  X INTEGER NOT NULL DEFAULT 100,
  Y INTEGER NOT NULL DEFAULT 100,
  Width INTEGER NOT NULL DEFAULT 300,
  Height INTEGER NOT NULL DEFAULT 400
);

CREATE TABLE IF NOT EXISTS ClientGroups (
  Id INTEGER PRIMARY KEY AUTOINCREMENT,
  ProfileId INTEGER NOT NULL,
  Name TEXT NOT NULL,
  DisplayOrder INTEGER NOT NULL DEFAULT 0,
  CycleForwardHotkey TEXT NOT NULL DEFAULT '',
  CycleBackwardHotkey TEXT NOT NULL DEFAULT '',
  FOREIGN KEY (ProfileId) REFERENCES Profile(Id) ON DELETE CASCADE,
  UNIQUE(ProfileId, Name)
);

CREATE TABLE IF NOT EXISTS ClientGroupMembers (
  GroupId INTEGER NOT NULL,
  WindowTitle TEXT NOT NULL,
  DisplayOrder INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (GroupId, WindowTitle),
  FOREIGN KEY (GroupId) REFERENCES ClientGroups(Id) ON DELETE CASCADE
);
```

### 5.2 AppSettings keys used by YAEP
- `Theme` = `Light` or `Dark`
- `ThemeColor` = `SukiColor` enum string
- `EnableThumbnailDragging` = `true/false`
- `StartHidden` = `true/false` (stored, not found in runtime usage elsewhere in repo)
- `IgnoredKeys` = comma-separated key names (stored, not found used elsewhere in repo)

Drawer settings are stored in AppSettings with these keys:
- `DrawerScreenIndex`
- `DrawerHardwareId`
- `DrawerSide` (`Left`/`Right`)
- `DrawerWidth`
- `DrawerHeight`
- `DrawerIsVisible`
- `DrawerIsEnabled`
- `DrawerSelectedMumbleServerGroupId` (nullable; empty string when unset)

### 5.3 Default bootstrap data
If no profile exists:
- Create Profile named `Default`
- Set it active
- Create ThumbnailDefaultConfig for that profile with hardcoded defaults
- Add process name `exefile` to ProcessesToPreview
- Create a ClientGroup named `Default`

## 6. Data Shapes (JSON payloads you’ll want from Tauri commands)
These are the model fields used in UI logic:

- `Profile`
  - `Id: number`, `Name: string`, `DeletedAt: string|null`, `IsActive: boolean`, `SwitchHotkey: string`, `IsDeleted: boolean (derived)`
- `ThumbnailConfig`
  - `Width, Height, X, Y: number`, `Opacity: number`, `FocusBorderColor: string`, `FocusBorderThickness: number`, `ShowTitleOverlay: boolean`
- `ThumbnailSetting`
  - `WindowTitle: string`, `Config: ThumbnailConfig`
- `ClientGroup`
  - `Id, ProfileId: number`, `Name: string`, `DisplayOrder: number`, `CycleForwardHotkey: string`, `CycleBackwardHotkey: string`
- `ClientGroupMember`
  - `GroupId: number`, `WindowTitle: string`, `DisplayOrder: number`
- `ClientGroupWithMembers`
  - `Group: ClientGroup`, `Members: ClientGroupMember[]`
- `MumbleServerGroup`
  - `Id: number`, `Name: string`, `DisplayOrder: number`
- `MumbleLink`
  - `Id, DisplayOrder: number`, `Name: string`, `Url: string`, `IsSelected: boolean`
  - `ServerGroupId: number|null` (legacy/display)
  - `ServerGroupName: string` (optional from join)
  - `Hotkey: string`
  - Methods: `OpenLink()` (validates URL; opens via system handler)
- `MumbleLinksOverlaySettings`
  - `AlwaysOnTop: boolean`, `X,Y,Width,Height: number`
- `DrawerSettings`
  - `ScreenIndex: number`, `HardwareId: string`, `Side: "Left"|"Right"`, `Width,Height: number`,
  - `IsVisible: boolean`, `IsEnabled: boolean`, `SelectedMumbleServerGroupId: number|null`
- `MonitorInfo` (used for monitor picker)
  - `Screen` (native Screen), but in a Tauri reimplementation you’ll likely store:
  - `Name, Bounds, WorkingArea, IsPrimary, HardwareId, DisplayNumber`

## 7. Hotkey Format and Handling (must match)
### 7.1 Hotkey parsing format
Hotkeys in DB are strings like:
- `Ctrl+Alt+F13`
- `Win+NumPad1`
- `Space`
- `A`
- `0`
- plus optional `Shift`

Supported:
- Modifiers: `Ctrl/Control`, `Alt`, `Shift`, `Win/Windows`
- Keys:
  - `F1`..`F24`
  - `NumPad0`..`NumPad9`
  - special names: `Space`, `Enter`, `Tab`, `Escape|Esc`, `Backspace|Back`, `Delete|Del`, `Insert|Ins`, `Home`, `End`, `PageUp|PgUp`, `PageDown|PgDn`, `Up`, `Down`, `Left`, `Right`
  - single alnum: `A-Z`, `0-9`

### 7.2 Hotkey actions
- Profile hotkey (`Profile.SwitchHotkey`)
  - Calls `SetCurrentProfile(profileId)` and re-registers hotkeys
- Client group forward/back hotkeys
  - Determines target next window title among active thumbnails
  - Activates it via Win32
- Mumble link hotkey
  - Opens `MumbleLink.Url`

### 7.3 Keyboard hook suppression (important QoL)
A low-level keyboard hook tracks "currently pressed" keys and suppresses non-modifier key input when the user is in the middle of pressing keys for a registered hotkey (to avoid the destination application receiving those keystrokes).

## 8. Thumbnail Pipeline (process scanning -> DWM thumbnail -> overlay)
### 8.1 Scanning loop
Every 2000ms:
- Read active profile
- Load process names from `ProcessesToPreview`
- For each configured process name:
  - get processes by name (normalized by removing `.exe`)
  - for each process:
    - check main window handle
    - decide whether to create or update thumbnail window

### 8.2 Title rules
- Skip if window title is base `EVE` (exact match)
- Otherwise create thumbnail window with title-based settings key:
  - `ThumbnailSettings(ProfileId, WindowTitle)`

### 8.3 DWM thumbnail registration
Inside thumbnail window:
- `ThumbnailControl` hosts a live DWM thumbnail:
  - finds parent native handle for destination
  - calls `DwmRegisterThumbnail(destinationHandle, processMainWindowHandle)`
  - resizes the DWM thumbnail to fill the control bounds
  - sets opacity using DWM thumbnail properties

### 8.4 Overlay border/title
Separate always-on-top overlay window:
- click-through enabled via WS_EX flags
- sync position/size with thumbnail window
- border shows only when `IsFocused`
- title overlay shows when `ShowTitleOverlay` is enabled

### 8.5 Resize/drag persistence
- Thumbnail window maintains last known valid position
- Saves to DB after:
  - right drag end
  - debounced position tracking
  - Ctrl+wheel resize changes
- Positions are clamped to working area bounds; if no screens can be determined, it falls back to threshold-based validity checks.

## 9. Grid Layout Algorithm (the "bulk move/resize" engine)
Inputs:
- `gridCellWidth`, `gridCellHeight` OR `gridCellRatio` (aspect ratio)
- `gridStartX`, `gridStartY`
- `gridColumns`
- `selectedMonitor` (working area offset)
- optional `selectedGroup`
- optional `OnlyAffectActiveThumbnails`

Steps:
1. Load thumbnail settings (`ThumbnailSettings`) for profile, filtered:
   - WindowHelper.ShouldIncludeWindowTitle
   - group membership if selected
   - optionally only active thumbnails
2. Determine ordering:
   - fetch group/member ordering dictionary from DB:
     - group display order + member display order
   - sort thumbnail settings by:
     - known group ordering, else fallback `(int.MaxValue, X)`
3. Preview grid positions:
   - for each ordered thumbnail:
     - compute relative grid cell position: `col * cellWidth`, `row * cellHeight`
     - add monitor working-area offset
4. Apply grid:
   - pause thumbnail monitoring
   - for each grid item:
     - save new settings (Width/Height/X/Y) into DB with preserved:
       - opacity
       - focus border color/thickness
       - `ShowTitleOverlay`
     - if title is currently active, update the live thumbnail immediately
   - clamp thumbnails into valid bounds and save corrected X/Y if clamped

## 10. Mumble Links Overlay (selection + drag + always-on-top)
### 10.1 Database-driven selection
- Selected links are `MumbleLinks.IsSelected = 1`
- `MumbleLinksWindow` appears only when selected links count > 0
- Always-on-top stored in `MumbleLinksOverlaySettings.AlwaysOnTop`

### 10.2 Window content and behavior
- Overlay lists selected links as buttons; clicking a button opens the link URL
- Combo box lists unselected links:
  - selecting one triggers `OpenLink()` and clears the selection UI state

### 10.3 Drag + persistence
- Right mouse drag changes window position
- On drag end:
  - saves AlwaysOnTop, X,Y,Width,Height
- While dragging, SaveSettings is suppressed using internal flags

### 10.4 Height calculation
- computed from link count:
  - headerHeight `35`
  - buttonHeight `48` * linkCount
  - bottomSectionHeight `50`
  - borderAndPadding `10`
- clamped to min `150`, max `800`

## 11. Drawer Overlay (slide-out UI anchored to monitor)
### 11.1 Persisted settings
Drawer enabled/visible/size/position are stored in DB via AppSettings keys (see section 5.2).

### 11.2 Monitor targeting via hardware ID
MonitorService:
- enumerates monitors
- builds a cache of monitor hardware IDs (device instance path when available)
- used to select correct monitor even when index shifts

DrawerWindowService:
- recalculates drawer height to target screen working-area height
- updates hardware ID if missing

### 11.3 Slide-in/out mechanics
- DrawerWindow animates by progressively changing Width with Timer + ease-out cubic
- DrawerIndicatorWindow:
  - positioned at screen edge
  - hover triggers "show drawer" after 300ms

### 11.4 Interaction rules
- DrawerWindow pointer exit slides out unless:
  - ComboBox dropdown is open

## 12. Eve Online Profile Copy Tools (local filesystem + ESI)
EveOnlineProfileService:
- finds EVE installation path via Windows registry keys (32/64-bit + user key)
- derives local EVE directory from `%LOCALAPPDATA%\CCP\EVE`
- lists server folders
- lists profiles by scanning `settings_*` folders
- reads core character/user files:
  - core_char_* and core_user_*
- copies profiles:
  - recursively duplicates directory content to `settings_<newName>`
- copies character/user files:
  - overwrites matching files for all known entries in the target profile
- fetches character names from ESI:
  - `https://esi.evetech.net/latest/characters/{characterId}/?datasource=tranquility`
- blocks operations if EVE process is running (`Process.GetProcessesByName("exefile")`)

## 13. Rebuilding in Tauri + SvelteKit: Implementation Requirements

### 13.1 Core constraint: "web UI cannot implement DWM live thumbnails"
In Tauri:
- Put **all Windows native window + DWM thumbnail + hotkey handling** in the Rust side.
- The SvelteKit frontend should only:
  - edit data (profiles, settings, hotkeys, group membership, etc.)
  - call backend commands
  - subscribe to backend events (thumbnail added/removed, maybe focus changes, list updates)
- Rust creates/manages native windows:
  - Main app window
  - Thumbnail windows (transparent/topmost)
  - Click-through overlay windows per thumbnail
  - Mumble overlay window
  - Drawer window + drawer indicator window

### 13.2 Suggested Rust module breakdown
1. `db` module
   - SQLite schema + CRUD exactly as YAEP
   - emits events:
     - active profile changed
2. `hotkeys` module
   - global hotkeys registration (RegisterHotKey)
   - keyboard hook suppression (WH_KEYBOARD_LL)
   - hotkey actions:
     - switch profile
     - cycle group (requires querying active thumbnails)
     - open mumble link
   - hotkey capture mode API for UI editing pages
3. `thumbnail_service` module
   - process scanning loop (every 2000ms)
   - manage thumbnail window lifecycles keyed by PID
   - on title change: update overlay label + reload thumbnail settings for that window title key
   - focus tracking loop (every 100ms) + QoL behavior
   - group-drag operation support:
     - backend stores relative positions for current drag group
     - frontend triggers drag updates; backend moves windows
4. `dwm` module
   - wrappers around:
     - `DwmRegisterThumbnail`
     - `DwmUpdateThumbnailProperties`
     - `DwmUnregisterThumbnail`
5. `windows` module
   - per-window implementations:
     - Thumbnail window
       - hosts DWM thumbnail child area
       - handles right-drag + ctrl+wheel resize
     - Overlay window (click-through)
     - MumbleLinksWindow
     - DrawerWindow / DrawerIndicatorWindow
6. `eve_profile_tools` module (optional in initial MVP)
   - local filesystem copy logic + ESI fetch

### 13.3 Frontend <-> backend contract (commands/events you likely need)
Commands (examples):
- `db.getProfiles()`
- `db.setCurrentProfile(profileId)`
- `db.getThumbnailDefaultConfig(profileId)`
- `db.setThumbnailDefaultConfig(profileId, config)`
- `db.getThumbnailSettings(profileId)`
- `db.saveThumbnailSettings(profileId, windowTitle, config)`
- `db.getProcessesToPreview(profileId)` / add/remove
- `db.getClientGroups(profileId)` / add/update/delete/order/members
- `db.getMumbleLinks()` / update selection/order/hotkey
- `db.getMumbleServerGroups()` / order/create/delete
- `db.getMumbleLinksOverlaySettings()` / save
- `db.getDrawerSettings()` / save
- `hotkeys.captureStart(type, targetId)` and `hotkeys.captureStop()`
- `grid.applyLayout(layoutPayload)` (or do "save updates only" and let live refresh update)

Events (examples):
- `thumbnailAdded(windowTitle, pid)`
- `thumbnailRemoved(windowTitle, pid)`
- `profileChanged(newProfileId)`
- `focusChanged(windowTitle|null)` (or only send "focused thumbnail changed")
- `drawerVisibilityChanged` (optional)

### 13.4 Feature parity checklist for MVP vs "full"
If you need a phased plan:
- MVP must include: DWM thumbnail windows + overlay border/title + focus tracking + profile/group hotkeys + SQLite + thumbnail settings editor.
- Drawer + Eve profile tools can be added after the thumbnail core is stable.

## 14. Notes / Known Implementation Quirks to Preserve
- Thumbnail title inclusion excludes exact `EVE` only.
- Focus border QoL: border isn’t cleared when foreground is a non-thumbnail app.
- Grid apply:
  - pauses monitoring
  - updates DB first, then updates live thumbnails only for active ones
  - clamps afterwards and saves corrected X/Y.
- App settings `StartHidden` is stored but not found used in other runtime logic in this repo (reimplementation should decide whether to implement or follow current behavior).
- `IgnoredKeys` is stored but not found used in repo runtime logic (if replicating fully, include UI for capture suppression behavior only if you identify how YAEP intended to use it).
