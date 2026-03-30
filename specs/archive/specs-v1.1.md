# SPECS ARCHIVE — v1.1 Security hardening + UI redesign

```
Status
- Version:  v1.1
- Archived: 2026-03-30
- Era:      Work completed in session immediately after v1.0 Tauri GUI shipped
- Commits:  31d42ae (security hardening)
            72997f9 (entitlements + App Support storage + README)
            cddffcb (pixel-perfect UI redesign, tabs, per-tab console, Clear)
- Context:  User reviewed the working v1.0 app, asked to harden security, then
            shared Swift Playgrounds screenshots and asked for a pixel-perfect
            UI match. Both workstreams completed in the same session.
```

---

## What prompted this work

**Security:** After the v1.0 app was running, the user noticed a reference to localhost
in the Tauri dev output and asked whether the JS↔Rust bridge could be hijacked to
inject arbitrary Rust code. This triggered a full security audit and two hardening commits.

**UI:** User shared two screenshots of Apple Swift Playgrounds and said
*"can we make this app almost pixel perfect"* followed by *"And tabs for playgrounds."*
This triggered a full visual redesign targeting macOS dark system colours, Apple-style
sidebar selection, file tabs above the editor, and a custom Monaco syntax theme.

---

## SECURITY HARDENING (commits 31d42ae + 72997f9)

### Path traversal prevention

Two independent layers added to every Tauri command that accepts a playground name:

**Layer 1 — Name whitelist validation (`validate_name()`)**
- Accepts only `[a-z][a-z0-9_]*`, max 64 characters
- Rejects: path separators (`/`, `..`), Unicode, shell metacharacters, uppercase,
  leading digits, empty strings
- Called before any filesystem operation or process spawn

**Layer 2 — Canonicalized path check (`safe_playground_path()`)**
- Canonicalizes `bin_dir` (resolves symlinks, normalises `.` and `..`)
- Constructs the candidate path, canonicalizes it, and verifies its parent
  matches `bin_dir` exactly
- Catches symlink-based escapes that pass the name whitelist

Both layers applied to: `load`, `save`, `new`, `rename`, `delete`, `duplicate`, `run`.

### Entitlements (`src-tauri/entitlements.plist`)

Explicit macOS entitlements for a non-sandboxed developer tool:

| Entitlement | Value | Reason |
|---|---|---|
| `com.apple.security.app-sandbox` | `false` | Same policy as Xcode, VS Code, Terminal |
| `cs.disable-library-validation` | `true` | rustc loads unsigned proc macro dylibs |
| `cs.allow-unsigned-executable-memory` | `true` | LLVM JIT inside rustc |
| `cs.allow-dyld-environment-variables` | `true` | cargo toolchain internals |
| `network.client` | `true` | cargo needs crates.io access |

Referenced from `tauri.conf.json` → `bundle.macOS.entitlements`.

### Production storage — App Support

Production builds no longer write to the app bundle (which is read-only when
distributed as a signed `.app`). Storage moved to:

```
~/Library/Application Support/com.playground-rs.app/workspace/
├── Cargo.toml        ← seeded on first launch
└── src/bin/          ← user playgrounds
    └── hello.rs      ← seeded on first launch
```

- `ensure_workspace()` runs in the Tauri `.setup()` hook
- Creates directory, `Cargo.toml`, and `hello.rs` on first launch if absent
- Dev mode unchanged — still uses project `src/bin/` so `cargo tauri dev` works
- `workspace_path()` command added so the frontend can display the storage location

### README warning

Big red `> [!WARNING]` block added to the top of `README.md`:
- NOT sandboxed — stated explicitly
- Full system access — stated explicitly
- No binary distributed — compile from source only
- "You are responsible for code you run"

Full documentation rewrite follows: GUI usage, CLI usage, keyboard shortcuts,
security model, build instructions.

---

## UI REDESIGN (commit cddffcb)

### Motivation

The v1.0 UI used basic dark colours and had no file tabs. The user wanted it to look
and feel like Swift Playgrounds — Apple's macOS/iPad native coding environment.
Key requests: file tabs, Apple system blue pill for sidebar selection, dark macOS
system colours, native-feeling toolbar.

### macOS colour system (`ui/src/app.css`)

Full replacement of the colour palette with exact macOS dark system values:

```css
--bg:              #1c1c1e   /* systemBackground */
--bg-sidebar:      #2c2c2e   /* secondarySystemBackground */
--bg-elevated:     #3a3a3c   /* tertiarySystemBackground */
--bg-tab:          #1c1c1e
--bg-tab-active:   #2c2c2e
--bg-hover:        rgba(255,255,255,0.07)
--accent:          #0a84ff   /* Apple system blue */
--accent-hover:    #409cff
--red:             #ff453a   /* Apple system red */
--amber:           #ffd60a   /* Apple system yellow */
--green:           #30d158   /* Apple system green */
--rust-orange:     #ce422b   /* Rust brand orange — used for RS badges */
--text:            #ffffff
--text-secondary:  rgba(235,235,245,0.80)
--text-tertiary:   rgba(235,235,245,0.50)
--border:          rgba(255,255,255,0.10)
--border-strong:   rgba(255,255,255,0.15)
```

Also added CSS custom properties for border radii (`--radius`, `--radius-sm`,
`--radius-xs`) and font stacks (`--font-mono`).

### New component: `TabBar.svelte`

File tabs above the editor, below the toolbar.

- Each tab shows: **RS badge** (Rust orange pill) + playground name + **× close button**
- Active tab: slightly lighter background + **accent-blue 2px underline** at bottom
- Dirty (unsaved) tabs: `●` appended to name
- Close button: hidden until tab is hovered or active; appears with opacity transition
- Horizontal scroll when many tabs open (scrollbar hidden)
- Empty state: *"Open a playground from the sidebar"*
- Uses `div[role="tab"]` + nested `<button>` for close (avoids invalid button-in-button)

### Redesigned: `Sidebar.svelte`

- **Search bar** — filters playground list as you type; clear × button when non-empty
- **RS file badge** — small Rust-orange pill before each playground name
- **Blue pill selection** — active item gets full `--accent` background (Apple style);
  file badge switches to translucent white on active row
- New button uses SVG `+` icon instead of text character
- Context menu: rounded corners, `role="menu"`, hover turns accent blue / red for danger
- `dirtyTabs` prop replaces `dirty: boolean` — sidebar now knows dirty state for all
  open tabs, not just the current one

### Redesigned: `Output.svelte`

- Header renamed from **OUTPUT** to **Console** (matches Swift Playgrounds panel title)
- **Error badge**: red pill with stderr line count, only shown when errors exist
- **CSS spinner**: shown while `status === 'compiling' | 'running'`
- **Clear button**: appears in header whenever output is non-empty; hidden when empty
- Info lines prefixed with accent-blue `›` character
- Empty state: play-button SVG icon + *"Run a playground to see output"*
- Receives `status` prop from parent (for spinner + error badge)

### Redesigned: `Editor.svelte` — custom Monaco theme

Replaced `vs-dark` (which uses `#1e1e1e`) with a custom theme `playground-dark`
matching our `#1c1c1e` background. Xcode Dark-inspired syntax colours:

| Token | Colour | Name |
|---|---|---|
| Keywords (`fn`, `let`, `mut`, …) | `#fc5fa3` | Pink |
| Types, type parameters | `#5dd8ff` | Cyan |
| String literals | `#fc6a5d` | Salmon |
| Number literals | `#d9c97c` | Gold |
| Comments | `#636770` italic | Gray |
| Operators | `#cdd6f4` | Soft white |
| Attributes, macros | `#a8c7fa` | Light blue |

Additional theme overrides:
- Editor background: `#1c1c1e` (matches `--bg`)
- Line highlight: `#2c2c2e`
- Selection: `#0a84ff40` (accent blue at 25% opacity)
- Cursor: `#0a84ff`
- Scrollbar: 6px, no shadow, semi-transparent
- `cursorSmoothCaretAnimation: 'on'`, `smoothScrolling: true`

### Rewritten: `App.svelte`

**Tab state management** — four new state records keyed by playground name:

```typescript
let openTabs:  string[]               // ordered list of open tab names
let activeTab: string | null          // which tab is in the editor
let tabCode:   Record<string, string> // cached source per tab
let dirtyTabs: string[]               // tabs with unsaved changes
```

`tabCode` means switching tabs is instant — no re-load from disk.
Closing a tab prompts if dirty (`confirm()`); cleans up all state maps on close.
`onRename` migrates all four maps to the new key.

**Per-tab output and status:**

```typescript
let tabOutput: Record<string, OutputLine[]>  // console history per tab
let tabStatus: Record<string, Status>        // run status per tab
```

- Switching tabs shows that tab's own output history and status
- Running one playground does not affect any other tab's output or status
- `currentOutput` and `currentStatus` derived from `activeTab` — rest of template
  is unaware of the map structure

**Append mode between runs:**
- Previous output is NOT cleared when a playground is run again
- A `──────────────────────────` divider line is inserted before each new run
- Run header: `▶  cargo run --bin <name>` (was `cargo run --bin <name>`)
- Output accumulates as a full run history for that tab

**Clear button:**
- `on:clear` handler in App.svelte zeroes `tabOutput[activeTab]` only
- Other tabs' output unaffected
- Clear button in Output.svelte only visible when `output.length > 0`

**Toolbar redesign:**
- Three-column layout: app name (left), current filename centred (absolute position),
  status + buttons (right)
- **Run button**: accent blue (`#0a84ff`) with SVG play triangle
- **Stop button**: dark gray with SVG stop square
- Never both visible — Run shows when idle/error, Stop shows when compiling/running
- Filename in centre shows `<name>.rs` + dirty dot

**Empty state:**
- Shown when no tabs are open
- SVG document icon, *"No playground open"*, link to create new
- Shortcut grid: ⌘N, ⌘R, ⌘S displayed as keyboard badge + description

---

## Files changed summary

| File | Change |
|---|---|
| `src-tauri/src/lib.rs` | Path traversal prevention, App Support storage, `ensure_workspace()`, `workspace_path()` |
| `src-tauri/entitlements.plist` | New file — explicit macOS entitlements |
| `src-tauri/tauri.conf.json` | Reference entitlements, macOS minimum version |
| `README.md` | Full rewrite with big red warning block |
| `ui/src/app.css` | Full colour system replacement with macOS dark values |
| `ui/src/App.svelte` | Tab state, per-tab output/status, toolbar redesign, empty state |
| `ui/src/lib/TabBar.svelte` | **New file** — file tabs component |
| `ui/src/lib/Sidebar.svelte` | Search bar, blue pill selection, RS badges, SVG + button |
| `ui/src/lib/Editor.svelte` | Custom Monaco theme `playground-dark` |
| `ui/src/lib/Output.svelte` | Console header, error badge, spinner, Clear button, empty state |

---

## Decisions made

- **Non-sandboxed is intentional and documented** — same posture as Xcode, VS Code,
  Terminal. The README warning makes this explicit. No binary distributed.
- **Two-layer path security** — name whitelist alone is not sufficient (symlinks);
  canonicalized path check is the belt to the whitelist's suspenders.
- **App Support over bundle** — app bundles are read-only when signed and distributed;
  App Support is the correct macOS-idiomatic location for user data.
- **Per-tab output** — each playground is a separate program with its own run history;
  mixing them in one panel would be confusing. History persists across tab switches.
- **Append, not replace** — clearing previous output before a new run loses useful
  history (e.g. "did I fix that error?"). Separator dividers keep history readable.
- **`#1c1c1e` not `#1e1e1e`** — macOS `systemBackground` dark is `#1c1c1e`.
  VS Code's `vs-dark` uses `#1e1e1e`. Small difference, noticeable when the editor
  background doesn't match the window chrome.
- **Xcode Dark syntax colours** — chosen over VS Code dark colours because the app
  is a macOS tool; Xcode colour conventions feel native to the platform.
