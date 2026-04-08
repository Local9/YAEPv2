# Planetary Interaction template JSON — field reference

This document describes the `.json` colony layout files in this folder: root keys, nested objects, and how they relate to EVE Online Planetary Interaction (PI) mechanics.

---

## Command Center levels

`CmdCtrLv` is the in-game **Command Center Upgrades** skill level applied to the template (0–5). It determines the colony’s **CPU** and **power grid** budget for all structures (including links).

| Level | CPU  | Power grid |
|------:|-----:|-----------:|
| 0     | 1675 | 6000       |
| 1     | 7057 | 9000       |
| 2     | 12136| 12000      |
| 3     | 17215| 15000      |
| 4     | 21315| 17000      |
| 5     | 25415| 19000      |

Use this table to check whether a template’s total structure CPU/power (plus links) fits the budget for the stated `CmdCtrLv`.

---

## Structure types — base costs and grid usage

These are typical PI building classes. **`T` values in `P[].T` are EVE Online type IDs** (not listed here); use the Static Data Export (SDE) or in-game info to resolve names and exact stats.

| Type              | CPU | Power grid | Cost (ISK) |
|-------------------|----:|-----------:|-----------:|
| Extractor         | 400 | 2600       | 45000      |
| Launch Pad        | 3600| 700        | 900000     |
| Storage           | 500 | 700        | 250000     |
| Advanced Factory  | 500 | 700        | 250000     |
| Factory           | 200 | 800        | 75000      |

Links consume additional CPU and power based on **level** and **length**; `L[].Lv` is the per-link upgrade level.

---

## Root-level properties

| Key       | Type   | Description |
|-----------|--------|-------------|
| `CmdCtrLv`| number | Command Center Upgrades skill level (0–5). See [Command Center levels](#command-center-levels). |
| `Cmt`     | string | Optional human-readable comment. May use HTML entities (e.g. `&gt;` for `>`). |
| `Diam`    | number | Planet **diameter** in meters (same scale as in-game planet info). Used for layout/radius context. |
| `L`       | array  | **Link** definitions between structures. See [Links (`L`)](#links-l). |
| `P`       | array  | **Placement** of each structure: position and type. See [Placements (`P`)](#placements-p). |
| `Pln`     | number | **Planet type** identifier (game-specific enum; e.g. barren, temperate). Not a unique planet in New Eden—category of planet. |
| `R`       | array  | **Routes** (material flow between structures). See [Routes (`R`)](#routes-r). |

---

## Links (`L`)

Each element connects two structures in the PI network.

| Field | Type   | Description |
|-------|--------|-------------|
| `D`   | number | **Direction** index for the link on the hex-style PI map (tool-specific encoding; values are integers such as 1–11 in samples). Identifies *which side* of a node the link uses. |
| `Lv`  | number | **Link level** (0–5 in game), affecting link CPU/power cost and capacity. |
| `S`   | number | **Structure ID** (local index within this file). Matches `P[].S` and IDs used in `R[].P` paths. |

---

## Placements (`P`)

Each element is one built structure (or pad) on the planet.

| Field | Type            | Description |
|-------|-----------------|-------------|
| `H`   | number          | **Height** / vertical or routing layer flag used by the authoring tool (often `0`; non-zero where the tool distinguishes levels). |
| `La`  | number          | **Latitude** in the template’s planetary coordinate system (radians or normalized; consistent within one file). |
| `Lo`  | number          | **Longitude** in the same system as `La`. |
| `S`   | number \| null  | **Structure ID** (local index). Some entries use `null` where the exporter does not assign an ID (e.g. certain pads or the command center, depending on tool). |
| `T`   | number          | **EVE type ID** of the structure (Extractor, Factory, Launch Pad, Storage, etc.). Resolve via SDE or [EVE University Wiki](https://wiki.eveuniversity.org/) item databases. |

---

## Routes (`R`)

Each element describes **movement of materials** along a path of structures.

| Field | Type   | Description |
|-------|--------|-------------|
| `P`   | array of number | **Path**: ordered list of **local structure IDs** (`S`) the flow follows (source toward destination). The same ID may appear **more than once** on purpose (e.g. re-entering a hub). Paths do **not** need to list every physical hop—**intermediate structures may be skipped** when the route is still unambiguous for the tool. Length ≥ 2 for a simple A→B hop. |
| `Q`   | number | **Quantity** per transfer (meaning depends on tool: often units per program step or per cycle for that route). |
| `T`   | number | **Type ID** of the **commodity** or **routed item** for that row (e.g. P0/P1 product type). Same EVE type-ID space as items. |

Example: `P: [8, 10, 8, 9]`, `Q: 20`, `T: 2393` means material type `2393` moves along `8 → 10 → 8 → 9` with quantity `20`; the repeated `8` is an **intentional** hop (not an export glitch). Shorter paths (e.g. `[5, 8]`) omit intermediates where allowed.

---

## Cross-references inside one file

1. **`P[].S`** — primary key for structures when `S` is not `null`.  
2. **`L[].S`** — one endpoint of a link; the other endpoint is implied by pairing or adjacency rules in the exporter (depends on tool).  
3. **`R[].P`** — only references IDs that should exist in `P[].S` for routing to be meaningful.

If you change structure IDs in an editor, keep **`L`**, **`P`**, and **`R`** consistent.

---

## Quick checklist when reading a template

- [ ] `CmdCtrLv` vs. sum of structure + link CPU/power (using your tables and in-game link math).  
- [ ] `Diam` vs. planet size you intend to use.  
- [ ] `Pln` vs. planet class you are building on.  
- [ ] Resolve all `P[].T` and `R[].T` via type IDs for correct buildings and products.  
- [ ] `Cmt` for the author’s recipe note (e.g. P0→P1 chain).

---

## TODO — information to capture

Track gaps so the reference can be completed as sources or the export tool are documented.

- [ ] **`Pln`** — Full mapping of numeric values to planet types (barren, temperate, storm, etc.) and any tool-specific IDs.
- [ ] **`L[].D`** — Exact direction encoding (e.g. 12 clock positions, hex edge indices) and how it pairs with planet layout.
- [ ] **`L` topology** — How the second endpoint of each link is derived (order of `L` entries, pairing rules, adjacency from placements).
- [ ] **Link CPU / power** — Formula or table for CPU and power per link length and `Lv` (to validate templates against `CmdCtrLv`).
- [ ] **`P[].H`** — Precise meaning (routing layer, z-order, tool flag) and when it is non-zero.
- [ ] **`P[].La` / `P[].Lo`** — Units (radians, degrees, normalized 0–1), origin, and relation to in-game PI view.
- [ ] **`P[].S` = `null`** — Which structure types omit `S` and how routes still reference them (if at all).
- [ ] **`R[].Q`** — Semantics: quantity per hour, per extractor cycle, per factory cycle, or exporter-specific unit.
- [ ] **Type IDs** — Optional local table mapping frequently used `P[].T` / `R[].T` values to item names for this template set.
- [ ] **Source tool** — Name and version of the application that writes these JSON files (for versioned schema quirks).

---

*Generated for the PlanetaryInteractionTemplates project.*
