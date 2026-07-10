<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import LanguageLogo from './LanguageLogo.svelte'
  import type { ProjectType } from './languages'

  // ── Types ────────────────────────────────────────────────────────────────
  type ToolchainInfo = {
    name: string
    short_name: string
    version: string | null
    is_rustup_default: boolean
    is_active: boolean
  }

  // ── Props ────────────────────────────────────────────────────────────────
  let {
    projectType,
    projectPath,
    missingProjectPin,
    projectPinnedName,
    pillStatus,
    pillIcon,
    pillText,
    onOpenFixWizard,
    onOpenInstallDialog,
    onToolchainSwitched,
    onRemovePin,
  }: {
    projectType: ProjectType
    /** Absolute path to the active project directory, or null if none loaded. */
    projectPath: string | null
    /** v0.4+: name of the toolchain the active project pins in
        rust-toolchain.toml, IF that toolchain isn't installed. When set, the
        dropdown surfaces a prominent "install and use it" prompt, and the
        pill tooltip explains the silent fallback. */
    missingProjectPin: string | null
    /** v0.4+: name of the toolchain the active project pins (whether it's
        installed or not). Used to render 📌 next to the row that matches
        this name, and to conditionally show the "Remove pin" action. */
    projectPinnedName: string | null
    pillStatus: 'not-enabled' | 'missing' | 'partial' | 'ok'
    pillIcon: string
    pillText: string
    /** Opens the ToolchainFixWizard/settings modal. */
    onOpenFixWizard: () => void
    /** Opens the InstallToolchainDialog. Optional preferredVersion pre-fills
        the dialog's version input (used by the "install newer stable?" hint). */
    onOpenInstallDialog: (preferredVersion?: string) => void
    /** Called after a successful toolchain switch so the parent can refresh
        toolchainInfo (which drives the pill display). */
    onToolchainSwitched: () => Promise<void>
    /** v0.4+: called when the user clicks "Remove pin" — parent should delete
        rust-toolchain.toml and refresh toolchainInfo. */
    onRemovePin: () => Promise<void>
  } = $props()

  // ── State ────────────────────────────────────────────────────────────────
  let open = $state(false)
  let toolchains: ToolchainInfo[] = $state([])
  let latestKnownStable = $state('')
  let loading = $state(false)
  let error = $state('')

  // ── Actions ──────────────────────────────────────────────────────────────
  async function toggle() {
    if (open) { close(); return }
    open = true
    // Only load Rust toolchains lazily when the picker is opened while the
    // active project is Rust and rustup is at least partially working. For
    // non-Rust or broken toolchains, the popover shows a "manage in settings"
    // fallback instead — cheaper and clearer than showing an error banner.
    if (projectType === 'rust' && (pillStatus === 'ok' || pillStatus === 'partial')) {
      await loadToolchains()
    }
  }

  function close() {
    open = false
    error = ''
  }

  async function loadToolchains() {
    loading = true
    error = ''
    try {
      // Run in parallel — get_latest_known_stable is a trivial constant read;
      // list_rust_toolchains may take 50-300ms depending on installed count
      // (subprocess call per channel-named toolchain to resolve its version).
      const [list, latest] = await Promise.all([
        invoke<ToolchainInfo[]>('list_rust_toolchains'),
        invoke<string>('get_latest_known_stable'),
      ])
      toolchains = list
      latestKnownStable = latest
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function selectToolchain(t: ToolchainInfo) {
    if (t.is_active) { close(); return }
    try {
      // 1. Persist as the app's session-level active toolchain (config.json).
      //    Used as default for new projects and as fallback when a project's
      //    pin isn't installed.
      await invoke('set_active_toolchain', {
        toolchain: t.name,
        applyToActiveProject: !!projectPath,
      })
      // 2. If a project is loaded, also write its rust-toolchain.toml so
      //    subsequent runs use the newly-selected toolchain immediately.
      //    We pass short_name (channel or bare version) rather than the full
      //    name — matches what real Rust developers write in their pins.
      if (projectPath) {
        await invoke('set_project_toolchain', { projectPath, name: t.short_name })
      }
      close()
      await onToolchainSwitched()
    } catch (e) {
      error = String(e)
    }
  }

  // Compare (major, minor, patch) tuples — returns positive if `a > b`, 0 if
  // equal, negative if `a < b`. Returns null on any parse failure (caller
  // should treat that as "don't compare").
  function compareVersions(a: string, b: string): number | null {
    const ap = a.split('.').map(Number)
    const bp = b.split('.').map(Number)
    if (ap.length !== 3 || bp.length !== 3) return null
    if (ap.some(isNaN) || bp.some(isNaN)) return null
    for (let i = 0; i < 3; i++) {
      if (ap[i] !== bp[i]) return ap[i] - bp[i]
    }
    return 0
  }

  // Hint rule (per spec — see specs/specifications.md):
  // Show iff max(t.version across ALL installed toolchains) < LATEST_KNOWN_STABLE.
  // "Any channel counts" — a user on nightly 1.99 is not asleep at the wheel,
  // and shouldn't get nagged about installing latest stable.
  let showUpgradeHint = $derived.by(() => {
    if (!latestKnownStable || toolchains.length === 0) return false
    for (const t of toolchains) {
      if (!t.version) continue
      const cmp = compareVersions(t.version, latestKnownStable)
      if (cmp === null) continue
      if (cmp >= 0) return false // user has current-or-newer somewhere
    }
    return true // no installed toolchain is at or above latest known stable
  })

  // For toolchain rows: show the version in a subtler position when the
  // short_name is a channel ("stable", "nightly", "beta") — so "stable
  // (1.90.0)". When the short_name IS the version (pinned toolchain like
  // "1.85.0"), don't repeat it.
  function versionLabel(t: ToolchainInfo): string | null {
    if (!t.version) return null
    if (t.version === t.short_name) return null
    return t.version
  }

  // Does the given toolchain match the current project's rust-toolchain.toml
  // pin? Used to render 📌 next to that row in the dropdown.
  function isPinnedForProject(t: ToolchainInfo): boolean {
    if (!projectPinnedName) return false
    return t.short_name === projectPinnedName || t.name === projectPinnedName
  }
</script>

<div class="toolchain-picker">
  <button
    class="toolchain-info"
    class:pill-red={pillStatus === 'not-enabled' || pillStatus === 'missing'}
    class:pill-yellow={pillStatus === 'partial' || !!missingProjectPin}
    class:open
    title={projectType === 'rust'
      ? (missingProjectPin
          ? `Project pins ${missingProjectPin}, but it's not installed — using fallback. Click to install.`
          : 'Manage Rust toolchains…')
      : (pillStatus === 'not-enabled' ? 'Language support not enabled in Settings' : '')}
    onclick={toggle}
  >
    <LanguageLogo type={projectType === 'rust' ? 'cargo' : projectType} size={16} />
    <span class="pill-dot">{pillIcon}</span>
    <span class="toolchain-text">{pillText}</span>
    <svg class="pill-caret" width="9" height="6" viewBox="0 0 10 6">
      <path d="M1 1l4 4 4-4" stroke="currentColor" stroke-width="1.5"
            stroke-linecap="round" fill="none"/>
    </svg>
  </button>

  {#if open}
    <div class="backdrop" onclick={close} aria-hidden="true"></div>
    <div class="popover" role="menu">

      {#if projectType !== 'rust'}
        <div class="empty-msg">
          The toolchain picker is Rust-only. Manage other languages in Settings.
        </div>
        <button class="menu-item" onclick={() => { close(); onOpenFixWizard() }}>
          Manage in Settings…
        </button>

      {:else if pillStatus === 'not-enabled' || pillStatus === 'missing'}
        <div class="empty-msg">
          {pillStatus === 'not-enabled'
            ? 'Rust support is not enabled.'
            : 'Rust toolchain is not installed.'}
          Set it up in Settings first.
        </div>
        <button class="menu-item" onclick={() => { close(); onOpenFixWizard() }}>
          Manage in Settings…
        </button>

      {:else if loading}
        <div class="loading">Loading toolchains…</div>

      {:else if error}
        <div class="error" role="alert">{error}</div>
        <button class="menu-item" onclick={() => { close(); onOpenFixWizard() }}>
          Manage in Settings…
        </button>

      {:else}
        {#if missingProjectPin}
          <!-- Highest-priority hint: project pins a toolchain that isn't
               installed. One click installs and switches to it. -->
          <button
            class="menu-item hint hint-warning"
            onclick={() => { close(); onOpenInstallDialog(missingProjectPin) }}
          >
            <span class="hint-title">This project pins {missingProjectPin} — not installed</span>
            <span class="hint-sub">Install and use it? (Currently using fallback.)</span>
          </button>
          <div class="separator"></div>
        {:else if showUpgradeHint}
          <button
            class="menu-item hint"
            onclick={() => { close(); onOpenInstallDialog(latestKnownStable) }}
          >
            <span class="hint-title">Latest stable is {latestKnownStable}</span>
            <span class="hint-sub">Install newer stable?</span>
          </button>
          <div class="separator"></div>
        {/if}

        <div class="section-label">Installed toolchains</div>
        <ul class="toolchain-list">
          {#each toolchains as t (t.name)}
            <li>
              <button
                class="menu-item toolchain-row"
                class:active={t.is_active}
                onclick={() => selectToolchain(t)}
                title={t.name}
              >
                <span class="checkmark">{t.is_active ? '✓' : ''}</span>
                <span class="tc-name">
                  {t.short_name}{#if isPinnedForProject(t)}<span class="pin-marker" title="Pinned to this project via rust-toolchain.toml">📌</span>{/if}
                </span>
                {#if versionLabel(t)}
                  <span class="tc-version">{versionLabel(t)}</span>
                {:else}
                  <span class="tc-version"></span>
                {/if}
                {#if t.is_rustup_default}
                  <span class="tag default">default</span>
                {:else}
                  <span class="tag-spacer"></span>
                {/if}
              </button>
            </li>
          {/each}
        </ul>

        <div class="separator"></div>
        {#if projectPinnedName && projectPath}
          <button
            class="menu-item subtle"
            onclick={async () => { close(); await onRemovePin() }}
            title="Delete rust-toolchain.toml from this project"
          >
            Remove pin ({projectPinnedName})
          </button>
        {/if}
        <button class="menu-item" onclick={() => { close(); onOpenInstallDialog() }}>
          Install Toolchain…
        </button>
        <button class="menu-item subtle" onclick={() => { close(); onOpenFixWizard() }}>
          Manage in Settings…
        </button>
      {/if}

    </div>
  {/if}
</div>

<style>
  .toolchain-picker { position: relative; }

  /* Pill styles — copied from App.svelte's .toolchain-info to keep behavior
     identical when the picker is closed. Component ownership: this file now
     owns the pill, so if we ever refactor the shared styles into a common
     stylesheet, both places move together. */
  .toolchain-info {
    display: flex; align-items: center; gap: 5px;
    font-size: 11px; font-family: var(--font-mono);
    color: var(--text-secondary);
    white-space: nowrap;
    background: none; border: 1px solid transparent;
    padding: 3px 8px; border-radius: 12px;
    cursor: pointer;
    transition: background 0.12s, border-color 0.12s;
  }
  .toolchain-info:hover,
  .toolchain-info.open {
    background: rgba(255,255,255,0.05);
    border-color: var(--border);
  }
  .toolchain-text { line-height: 1; }
  .pill-dot { font-size: 8px; line-height: 1; color: var(--green); margin-bottom: 1px; }
  .toolchain-info.pill-red { color: #f44; }
  .toolchain-info.pill-red .toolchain-text,
  .toolchain-info.pill-red .pill-dot { color: #f44; }
  .toolchain-info.pill-yellow { color: #e8a820; }
  .toolchain-info.pill-yellow .toolchain-text,
  .toolchain-info.pill-yellow .pill-dot { color: #e8a820; }
  .pill-caret { opacity: 0.5; margin-left: 2px; transition: transform 0.12s; }
  .toolchain-info.open .pill-caret { transform: rotate(180deg); }

  /* Backdrop for click-outside-to-close */
  .backdrop {
    position: fixed; inset: 0;
    z-index: 100;
    background: transparent;
  }

  .popover {
    position: absolute; top: calc(100% + 4px); left: 0;
    z-index: 101;
    min-width: 300px; max-width: 380px;
    background: var(--bg-card, #1a1a1a);
    border: 1px solid var(--border, #333);
    border-radius: 8px;
    padding: 4px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.35);
    font-size: 13px;
    color: var(--text);
  }

  .menu-item {
    display: flex; align-items: center; gap: 8px;
    width: 100%; padding: 8px 12px;
    background: transparent; border: none;
    color: var(--text); text-align: left; cursor: pointer;
    border-radius: 4px; font-size: 13px;
    font-family: inherit;
  }
  .menu-item:hover { background: rgba(255,255,255,0.06); }
  .menu-item.subtle { color: var(--text-secondary); font-size: 12px; }

  .menu-item.hint {
    flex-direction: column; align-items: flex-start; gap: 2px;
    padding: 10px 12px;
  }
  .menu-item.hint:hover { background: rgba(206, 66, 43, 0.08); }
  .hint-title { font-weight: 600; color: var(--accent, #ce422b); }
  .hint-sub { font-size: 11px; color: var(--text-secondary); }

  /* Warning variant for the "project pins X but not installed" prompt —
     distinct color from the upgrade hint so a user seeing both across
     sessions can tell them apart. Yellow signals action-required rather
     than just recommendation. */
  .menu-item.hint-warning:hover { background: rgba(232, 168, 32, 0.10); }
  .menu-item.hint-warning .hint-title { color: #e8a820; }

  .separator { height: 1px; background: var(--border, #333); margin: 4px 0; }

  .section-label {
    padding: 8px 12px 4px;
    font-size: 10px; font-weight: 600;
    color: var(--text-tertiary, #888);
    text-transform: uppercase; letter-spacing: 0.5px;
  }

  .toolchain-list { list-style: none; padding: 0; margin: 0; }

  .toolchain-row {
    display: grid;
    grid-template-columns: 14px 1fr auto 60px;
    gap: 8px; align-items: center;
    padding: 6px 12px;
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .checkmark { color: var(--accent, #ce422b); font-weight: 700; text-align: center; }
  .tc-name { color: var(--text); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .toolchain-row.active .tc-name { font-weight: 600; }
  .tc-version { color: var(--text-secondary); font-size: 11px; }
  .tag {
    font-size: 9px; font-family: var(--font-sans, sans-serif); text-transform: uppercase;
    padding: 2px 6px; border-radius: 3px;
    color: var(--text-tertiary, #888); background: rgba(255,255,255,0.06);
    letter-spacing: 0.4px;
    justify-self: end;
  }
  .tag-spacer { display: block; }

  /* Small pushpin next to whichever row matches the project's
     rust-toolchain.toml pin. font-size stays tight so it reads as a badge
     rather than a large emoji. */
  .pin-marker {
    margin-left: 6px;
    font-size: 10px;
    filter: saturate(0.8);
    display: inline-block;
  }

  .empty-msg, .loading, .error {
    padding: 12px;
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.4;
  }
  .error { color: #f44; }
</style>
