<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { open as shellOpen } from '@tauri-apps/plugin-shell'
  import { allLanguages, getLang, type ProjectType } from './languages'
  import LanguageLogo from './LanguageLogo.svelte'
  import type { Settings } from './SettingsModal.svelte'
  import type { EditionConfig } from './editions'

  let {
    onclose,
    onapply,
    onrepair,
    enabledLanguages = ['rust'],
    settings,
    projectSources = {},
    onthemechange,
    mode = 'wizard',
    edition,
    refreshKey = 0,
  }: {
    onclose: (result: { enabledLanguages: string[]; booksToLoad: string[]; booksToRemove: string[]; settings?: Settings }) => void
    onapply?: (result: { enabledLanguages: string[]; booksToLoad: string[]; booksToRemove: string[]; settings: Settings }) => void
    onrepair?: () => void
    enabledLanguages: string[]
    settings: Settings
    projectSources?: Record<string, string>
    onthemechange?: (theme: string) => void
    mode?: 'wizard' | 'settings'
    edition: EditionConfig
    /** Bumped externally after an in-app toolchain fix to force a re-check. */
    refreshKey?: number
  } = $props()

  type RustState = 'clt_missing' | 'not_installed' | 'no_default' | 'missing_components' | 'healthy'
  type ToolchainStatus = {
    wizard_completed: boolean
    all_good: boolean
    rust_state: RustState
    missing_components: string[]
    xcode_clt: { installed: boolean; path: string | null }
    rustup: { installed: boolean; version: string | null }
    cargo: { installed: boolean; path: string; version: string | null }
    rustc: { installed: boolean; version: string | null }
    active_toolchain: string | null
    components: { rustfmt: boolean; clippy: boolean }
    clang: { installed: boolean; path: string; version: string | null }
    zig: { installed: boolean; path: string; version: string | null; version_ok: boolean }
    swiftc: { installed: boolean; path: string; version: string | null }
  }

  // ── State ──────────────────────────────────────────────────────────────
  let step = $state(1)
  let activeTab: 'languages' | 'toolchains' | 'appearance' | 'books' = $state('languages')
  let selectedLangs: string[] = $state([...new Set(enabledLanguages)])
  let status = $state<ToolchainStatus | null>(null)
  let checking = $state(false)
  let draftSettings: Settings = $state({ ...settings })
  const originalTheme = settings.theme
  let applied = $state(false)
  const loadedSourceTags = new Set(Object.values(projectSources))
  let booksChecked: string[] = $state(
    mode === 'wizard'
      ? allLanguages().filter(l => l.book && enabledLanguages.includes(l.type)).map(l => l.type)
      : allLanguages().filter(l => l.book && loadedSourceTags.has(l.book.sourceTag)).map(l => l.type)
  )

  // Edition-aware: single-language edition's language config (for toolchain labels)
  const editionLang = edition.isSingleLanguage ? getLang(edition.languages![0] as ProjectType) : null

  // Derive labels from enabled languages and their book counts
  let enabledBooks = $derived(allLanguages().filter(l => selectedLangs.includes(l.type) && l.book))
  let bookCount = $derived(enabledBooks.length)
  let singleBookName = $derived(bookCount === 1 ? enabledBooks[0].book!.commandLabel : null)
  const toolchainLabel = edition.isSingleLanguage ? 'Toolchain' : 'Toolchains'
  let booksLabel = $derived(bookCount === 1 ? 'Book' : 'Books')

  // Edition-aware steps/tabs: single-language editions skip language picker
  let wizardSteps = $derived(edition.isSingleLanguage
    ? [toolchainLabel, 'Appearance', ...(bookCount > 0 ? [booksLabel] : []), 'Finish']
    : ['Languages', 'Toolchains', 'Appearance', ...(bookCount > 0 ? ['Books'] : []), 'Finish'])
  let allWizardPanels = $derived(edition.isSingleLanguage
    ? (['toolchains', 'appearance', ...(bookCount > 0 ? ['books' as const] : []), 'finish'] as const)
    : (['languages', 'toolchains', 'appearance', ...(bookCount > 0 ? ['books' as const] : []), 'finish'] as const))
  let totalSteps = $derived(wizardSteps.length)

  let settingsTabs = $derived([
    ...(!edition.isSingleLanguage ? [{ id: 'languages' as const, label: 'Languages' }] : []),
    { id: 'toolchains' as const, label: toolchainLabel },
    { id: 'appearance' as const, label: 'Appearance' },
    ...(bookCount > 0 ? [{ id: 'books' as const, label: booksLabel }] : []),
  ])

  // Default settings tab for single-language editions
  if (edition.isSingleLanguage && activeTab === 'languages') {
    activeTab = 'toolchains'
  }

  // Current panel (unified for both modes)
  let currentPanel = $derived(
    mode === 'wizard'
      ? allWizardPanels[step - 1]
      : activeTab
  )

  // Redirect away from books tab if no books available (language deselected)
  $effect(() => {
    if (mode === 'settings' && activeTab === 'books' && bookCount === 0) {
      activeTab = 'appearance'
    }
  })

  // Auto-run toolchain check when entering toolchains panel
  $effect(() => {
    if (currentPanel === 'toolchains' && !status && !checking) {
      runCheck()
    }
  })

  // Re-run check when an in-app toolchain fix completes (refreshKey bumped).
  // Skip the initial value (0) so we don't double-check on mount.
  // Silent mode: keep the existing status card visible during the re-check.
  $effect(() => {
    if (refreshKey > 0) {
      runCheck(true)
    }
  })

  // ── Language toggle ───────────────────────────────────────────────────��
  function toggleLang(type: string) {
    if (selectedLangs.includes(type)) {
      selectedLangs = selectedLangs.filter(l => l !== type)
    } else {
      selectedLangs = [...selectedLangs, type]
      // Auto-select book when enabling a language (if it has one)
      const lang = getLang(type as ProjectType)
      if (lang.book && !booksChecked.includes(type)) {
        booksChecked = [...booksChecked, type]
      }
    }
  }
  let hasSelection = $derived(selectedLangs.length > 0)
  let selectionLabel = $derived(
    hasSelection
      ? 'Selected: ' + selectedLangs.map(t => getLang(t as ProjectType).label).join(', ')
      : 'You must select at least one language'
  )

  // Note: we deliberately do NOT block Next/Get Started on a broken toolchain.
  // The status card makes the situation obvious (red pill, headline, install
  // button), and the user has multiple in-app paths to fix it later (toolbar
  // pill click, Settings → Repair Toolchain, Help → Rust Help → Rust Toolchain).
  // Forcing them through install before they can see the rest of the app would
  // be hostile — let them browse, read help, decide if they want this tool.
  // When they actually try to ⌘R a playground, the run will fail with a clear
  // error pointing back at the toolchain modal.

  // True when Rust is selected but the toolchain is in a hard-block state
  // (no rust at all, no CLT, or no default toolchain). Used to mark the
  // toolchain wizard step red instead of green when the user proceeds past
  // it without installing, and to annotate the finish-step summary so they
  // know the toolchain still needs attention.
  // Note: missing_components is NOT considered "skipped" — cargo can still
  // compile and run, the components are just optional polish.
  let toolchainSkipped = $derived(
    selectedLangs.includes('rust') &&
    status !== null &&
    (status.rust_state === 'clt_missing' ||
     status.rust_state === 'not_installed' ||
     status.rust_state === 'no_default')
  )

  // ── Toolchain check ───────────────────────────────────────────────────
  // `silent` keeps the existing status card visible (no spinner) while a
  // background re-check runs — used after an in-app fix completes.
  async function runCheck(silent = false) {
    if (!silent) checking = true
    try {
      status = await invoke<ToolchainStatus>('check_toolchain')
    } catch (_) { /* ignore */ }
    finally { if (!silent) checking = false }
  }

  function toolchainOk(type: string): boolean | null {
    if (!status) return null
    switch (type) {
      case 'rust': return status.all_good
      case 'clang': return status.clang.installed
      case 'zig': return (status.zig?.installed && status.zig?.version_ok) ?? false
      case 'swift': return status.swiftc?.installed ?? false
      default: return false
    }
  }

  // ── Theme ──────────────────────────────────────────────────────────────
  function setTheme(theme: string) {
    draftSettings.theme = theme
    onthemechange?.(theme)
  }

  // ── Book toggle ────────────────────────────────────────────────────────
  function toggleBook(type: string) {
    if (booksChecked.includes(type)) {
      booksChecked = booksChecked.filter(b => b !== type)
    } else {
      booksChecked = [...booksChecked, type]
    }
  }

  let availableBooks = $derived(
    allLanguages().filter(l => selectedLangs.includes(l.type) && l.book)
  )

  // ── Navigation (wizard mode) ──────────────────────────────────────────
  function next() {
    if (step < totalSteps) step += 1 as any
  }
  function back() {
    if (step > 1) step -= 1 as any
  }

  // ── Book diff: what to load vs remove based on checkbox changes ──────
  // Books for deselected languages are always removed, even if still in booksChecked
  function bookDiff() {
    const wasLoaded = allLanguages()
      .filter(l => l.book && loadedSourceTags.has(l.book.sourceTag))
      .map(l => l.type)
    const effectiveChecked = booksChecked.filter(b => selectedLangs.includes(b))
    const toLoad = effectiveChecked.filter(b => !wasLoaded.includes(b))
    const toRemove = wasLoaded.filter(b => !effectiveChecked.includes(b))
    return { toLoad, toRemove }
  }

  // ── Cancel ─────────────────────────────────────────────────────────────
  function cancel() {
    if (!applied) onthemechange?.(originalTheme)
    onclose({ enabledLanguages: applied ? selectedLangs : enabledLanguages, booksToLoad: [], booksToRemove: [], settings: applied ? draftSettings : undefined })
  }

  // ── Apply (settings mode — persist without closing) ───────────────────
  function apply() {
    applied = true
    const { toLoad, toRemove } = bookDiff()
    // Update baseline so subsequent Apply calls don't re-trigger
    for (const t of toLoad) loadedSourceTags.add(getLang(t as ProjectType).book!.sourceTag)
    for (const t of toRemove) loadedSourceTags.delete(getLang(t as ProjectType).book!.sourceTag)
    onapply?.({
      enabledLanguages: selectedLangs,
      booksToLoad: toLoad,
      booksToRemove: toRemove,
      settings: draftSettings,
    })
  }

  // ── Save / Finish ─────────────────────────────────────────────────────
  async function finish() {
    if (mode === 'wizard') {
      const { toLoad, toRemove } = bookDiff()
      await invoke('complete_wizard', { enabledLanguages: selectedLangs })
      onclose({
        enabledLanguages: selectedLangs,
        booksToLoad: toLoad,
        booksToRemove: toRemove,
        settings: draftSettings,
      })
    } else {
      if (!applied) apply()
      onclose({ enabledLanguages: selectedLangs, booksToLoad: [], booksToRemove: [], settings: draftSettings })
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape' && mode === 'settings') cancel()
  }

  const fontFamilies = [
    'Menlo', 'Monaco', 'Courier New',
  ]
</script>

<svelte:window onkeydown={handleKey} />

<div class="backdrop" onclick={() => mode === 'settings' && cancel()} aria-hidden="true"></div>

<div class="modal" role="dialog" aria-modal="true" aria-label={mode === 'wizard' ? 'Welcome' : 'Settings'}>
  <div class="modal-header">
    <div class="header-left">
      <span class="rs-badge">RS</span>
      <span class="modal-title">{mode === 'wizard' ? `Welcome to ${edition.displayName}` : 'Settings'}</span>
    </div>
    {#if mode === 'settings'}
      <button class="close-btn" onclick={cancel} aria-label="Close">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
        </svg>
      </button>
    {/if}
  </div>

  <!-- Wizard: step indicator / Settings: tab bar -->
  {#if mode === 'wizard'}
    <div class="step-bar">
      {#each wizardSteps as label, i}
        <!-- Conditions inlined into directives (not via {@const}) because
             {@const} inside {#each} doesn't reliably re-evaluate when
             external $state (status, toolchainSkipped) changes after the
             iteration first renders. Inlining ensures Svelte's reactivity
             tracking sees the dependencies and updates the classes when
             status arrives from check_toolchain. -->
        <button
          class="step-dot"
          class:active={step === i + 1}
          class:done={step > i + 1 && !(allWizardPanels[i] === 'toolchains' && toolchainSkipped)}
          class:skipped={step > i + 1 && allWizardPanels[i] === 'toolchains' && toolchainSkipped}
          onclick={() => { if (i + 1 < step) step = (i + 1) as any }}
          disabled={i + 1 > step}
        >
          <span class="dot">
            {#if step > i + 1 && allWizardPanels[i] === 'toolchains' && toolchainSkipped}
              !
            {:else if step > i + 1}
              ✓
            {:else}
              {i + 1}
            {/if}
          </span>
          <span class="step-label">{label}</span>
        </button>
        {#if i < wizardSteps.length - 1}
          <div
            class="step-line"
            class:done={step > i + 1 && !(allWizardPanels[i] === 'toolchains' && toolchainSkipped)}
            class:skipped={step > i + 1 && allWizardPanels[i] === 'toolchains' && toolchainSkipped}
          ></div>
        {/if}
      {/each}
    </div>
  {:else}
    <div class="tab-bar">
      {#each settingsTabs as tab}
        <button
          class="tab" class:active={activeTab === tab.id}
          onclick={() => activeTab = tab.id}
        >{tab.label}</button>
      {/each}
    </div>
  {/if}

  <div class="modal-body">

    <!-- ═══════════ Panel: Languages ═══════════ -->
    {#if currentPanel === 'languages'}
      <div class="step-content">
        <h2 class="step-heading">Choose Your Languages</h2>
        <p class="step-desc">Select which programming languages you'd like to use.</p>

        <div class="lang-grid">
          {#each allLanguages() as lang}
            <button
              class="lang-card"
              class:selected={selectedLangs.includes(lang.type)}
              onclick={() => toggleLang(lang.type)}
            >
              <LanguageLogo type={lang.type} size={24} />
              <span class="lang-name">{lang.label}</span>
              {#if lang.experimental}
                <span class="exp-tag">exp</span>
              {/if}
              <div class="check-mark">
                {#if selectedLangs.includes(lang.type)}
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <circle cx="8" cy="8" r="7" fill="var(--accent)" stroke="var(--accent)" stroke-width="1"/>
                    <path d="M4.5 8l2.5 2.5 4.5-5" stroke="#fff" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
                  </svg>
                {:else}
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <circle cx="8" cy="8" r="7" stroke="var(--border-strong)" stroke-width="1"/>
                  </svg>
                {/if}
              </div>
            </button>
          {/each}
        </div>
        <p class="selection-label" class:error={!hasSelection}>{selectionLabel}</p>
      </div>

    <!-- ═══════════ Panel: Toolchains ═══════════ -->
    {:else if currentPanel === 'toolchains'}
      <div class="step-content">
        <h2 class="step-heading">{edition.isSingleLanguage ? `${editionLang!.label} Toolchain` : 'Toolchain Check'}</h2>
        <p class="step-desc">{edition.isSingleLanguage ? `Status of your ${editionLang!.label} toolchain.` : 'Status of the required compilers and toolchains.'}</p>

        {#if checking}
          <div class="checking">
            <div class="spinner"></div>
            <span>Detecting toolchain…</span>
          </div>
        {:else if status}
          <div class="toolchain-list">
            {#each allLanguages().filter(l => selectedLangs.includes(l.type)) as lang}
              {@const ok = toolchainOk(lang.type)}
              <div class="toolchain-section">
                <div class="tc-header">
                  <LanguageLogo type={lang.type} size={18} />
                  <span class="tc-name">{lang.label}</span>
                  {#if lang.type !== 'rust'}
                    <span class="tc-status" class:ok={ok} class:missing={!ok}>
                      {ok ? '● Ready' : '○ Not found'}
                    </span>
                  {/if}
                </div>

                {#if lang.type === 'rust'}
                  <div class="status-card" class:healthy={status.rust_state === 'healthy'} class:unhealthy={status.rust_state !== 'healthy'}>
                    {#if status.rust_state === 'healthy'}
                      <div class="status-headline ok">● Rust toolchain is healthy</div>
                      <p class="status-sub">Everything is installed and ready to use.</p>
                    {:else if status.rust_state === 'clt_missing'}
                      <div class="status-headline missing">○ Xcode Command Line Tools required</div>
                      <p class="status-sub">Rust needs Apple's Command Line Tools to compile on macOS. Install them first — this one-time setup also enables C/C++ and Swift. <strong>Apple's installer takes about 10–15 minutes</strong>, so grab a coffee.</p>
                    {:else if status.rust_state === 'not_installed'}
                      <div class="status-headline missing">○ Rust is not installed</div>
                      <p class="status-sub">Install rustup, cargo, and the stable toolchain to start writing Rust.</p>
                    {:else if status.rust_state === 'no_default'}
                      <div class="status-headline missing">○ No default toolchain</div>
                      <p class="status-sub">rustup is installed, but no default toolchain is selected. This often happens after moving <code>~/.rustup</code>.</p>
                    {:else if status.rust_state === 'missing_components'}
                      <div class="status-headline warn">◐ Missing components</div>
                      <p class="status-sub">Rust is installed, but {status.missing_components.join(' and ')} {status.missing_components.length === 1 ? 'is' : 'are'} missing.</p>
                    {/if}
                  </div>
                  <div class="detail-grid">
                    <div class="detail-row">
                      <span class="detail-icon" class:ok={status.xcode_clt.installed} class:missing={!status.xcode_clt.installed}>{status.xcode_clt.installed ? '●' : '○'}</span>
                      <span class="detail-label">Xcode CLT</span>
                      <span class="detail-value">{status.xcode_clt.installed ? (status.xcode_clt.path ?? 'installed') : 'not found'}</span>
                    </div>
                    {#if status.rust_state !== 'clt_missing'}
                      <div class="detail-row">
                        <span class="detail-icon" class:ok={status.rustup.installed} class:missing={!status.rustup.installed}>{status.rustup.installed ? '●' : '○'}</span>
                        <span class="detail-label">rustup</span>
                        <span class="detail-value">{status.rustup.installed ? (status.rustup.version ?? 'installed') : 'not found'}</span>
                      </div>
                      <div class="detail-row">
                        <span class="detail-icon" class:ok={status.cargo.installed} class:missing={!status.cargo.installed}>{status.cargo.installed ? '●' : '○'}</span>
                        <span class="detail-label">cargo</span>
                        <span class="detail-value">{status.cargo.installed ? (status.cargo.version ?? 'installed') : 'not found'}</span>
                      </div>
                      <div class="detail-row">
                        <span class="detail-icon" class:ok={status.rustc.installed} class:missing={!status.rustc.installed}>{status.rustc.installed ? '●' : '○'}</span>
                        <span class="detail-label">rustc</span>
                        <span class="detail-value">{status.rustc.installed ? (status.rustc.version ?? 'installed') : 'not found'}</span>
                      </div>
                      <div class="detail-row">
                        <span class="detail-icon" class:ok={status.components.rustfmt} class:missing={!status.components.rustfmt}>{status.components.rustfmt ? '●' : '○'}</span>
                        <span class="detail-label">rustfmt</span>
                        <span class="detail-value">{status.components.rustfmt ? 'installed' : 'not found'}</span>
                      </div>
                      <div class="detail-row">
                        <span class="detail-icon" class:ok={status.components.clippy} class:missing={!status.components.clippy}>{status.components.clippy ? '●' : '○'}</span>
                        <span class="detail-label">clippy</span>
                        <span class="detail-value">{status.components.clippy ? 'installed' : 'not found'}</span>
                      </div>
                    {/if}
                  </div>
                  {#if status.rust_state !== 'healthy'}
                    <div class="install-section">
                      <button class="btn btn-primary" onclick={() => onrepair?.()}>
                        {mode === 'wizard' ? 'Install Rust Toolchain…' : 'Repair Rust Toolchain…'}
                      </button>
                    </div>
                  {/if}

                {:else if lang.type === 'clang'}
                  <div class="detail-grid">
                    <div class="detail-row">
                      <span class="detail-icon" class:ok={status.clang.installed} class:missing={!status.clang.installed}>{status.clang.installed ? '●' : '○'}</span>
                      <span class="detail-label">clang</span>
                      <span class="detail-value">{status.clang.installed ? (status.clang.version ?? 'installed') : 'not found'}</span>
                    </div>
                  </div>
                  {#if !status.clang.installed}
                    <div class="install-section">
                      <code class="install-cmd">xcode-select --install</code>
                    </div>
                  {/if}

                {:else if lang.type === 'zig'}
                  {@const zigInstalled = status.zig?.installed ?? false}
                  {@const zigVersionOk = status.zig?.version_ok ?? false}
                  {@const zigOk = zigInstalled && zigVersionOk}
                  <div class="detail-grid">
                    <div class="detail-row">
                      <span class="detail-icon" class:ok={zigOk} class:warn={zigInstalled && !zigVersionOk} class:missing={!zigInstalled}>{zigOk ? '●' : zigInstalled ? '◐' : '○'}</span>
                      <span class="detail-label">zig</span>
                      <span class="detail-value">{zigInstalled ? (status.zig?.version ?? 'installed') : 'not found'}</span>
                    </div>
                  </div>
                  {#if zigInstalled && !zigVersionOk}
                    <div class="install-section warn-note">
                      <p class="install-hint">Zig support targets <strong>0.15.x</strong>. Your version may have breaking API changes. Consider:</p>
                      <code class="install-cmd">brew install zig</code>
                    </div>
                  {:else if !zigInstalled}
                    <div class="install-section">
                      <code class="install-cmd">brew install zig</code>
                      <p class="install-hint">Or download from <button class="link-btn" onclick={() => shellOpen('https://ziglang.org/download/')}>ziglang.org</button></p>
                    </div>
                  {/if}

                {:else if lang.type === 'swift'}
                  {@const swiftOk = status.swiftc?.installed ?? false}
                  <div class="detail-grid">
                    <div class="detail-row">
                      <span class="detail-icon" class:ok={swiftOk} class:missing={!swiftOk}>{swiftOk ? '●' : '○'}</span>
                      <span class="detail-label">swiftc</span>
                      <span class="detail-value">{swiftOk ? (status.swiftc?.version ?? 'installed') : 'not found'}</span>
                    </div>
                  </div>
                  {#if !swiftOk}
                    <div class="install-section">
                      <code class="install-cmd">xcode-select --install</code>
                    </div>
                  {/if}
                {/if}
              </div>
            {/each}
          </div>

          <!-- Cargo path (always shown) -->
          <div class="cargo-path-row">
            <label for="wiz-cargo-path">Cargo Path</label>
            <input
              id="wiz-cargo-path"
              type="text"
              bind:value={draftSettings.cargo_path}
              class="path-input"
              spellcheck="false"
            />
          </div>

          <button class="btn btn-secondary recheck-btn" onclick={runCheck}>Re-check</button>

          {#if selectedLangs.includes('rust')}
            <!-- External help links — visible in both wizard and settings mode.
                 Compact footer so users know where to ask for help with toolchain
                 issues we can't fix in-app. -->
            <div class="help-links">
              <div class="help-links-title">Need help with the Rust toolchain?</div>
              <ul class="help-links-list">
                <li><button class="link-btn" onclick={() => shellOpen('https://rustup.rs')}>rustup.rs</button> — official installer</li>
                <li><button class="link-btn" onclick={() => shellOpen('https://www.rust-lang.org/learn/get-started')}>rust-lang.org</button> — getting started guide</li>
                <li><button class="link-btn" onclick={() => shellOpen('https://users.rust-lang.org')}>users.rust-lang.org</button> — friendly Q&A forum</li>
                <li><button class="link-btn" onclick={() => shellOpen('https://www.reddit.com/r/rust')}>r/rust</button> — Reddit community</li>
                <li><button class="link-btn" onclick={() => shellOpen('https://github.com/rust-lang/rustup/issues')}>rustup issues</button> — for installer bugs</li>
                <li><button class="link-btn" onclick={() => shellOpen('https://github.com/jagmeetchawla/rustic-playground/issues')}>Report a bug</button> — for issues with this app</li>
              </ul>
            </div>
          {/if}
        {/if}
      </div>

    <!-- ═══════════ Panel: Appearance ═══════════ -->
    {:else if currentPanel === 'appearance'}
      <div class="step-content">
        <h2 class="step-heading">Appearance</h2>
        <p class="step-desc">Set your preferred theme and editor font.</p>

        <div class="appearance-grid">
          <div class="setting-row">
            <label for="wiz-theme">Theme</label>
            <select
              id="wiz-theme"
              value={draftSettings.theme}
              onchange={(e) => setTheme((e.target as HTMLSelectElement).value)}
            >
              <optgroup label="General">
                <option value="system">System</option>
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </optgroup>
              <optgroup label={edition.isSingleLanguage ? 'Language' : 'Languages'}>
                {#if !edition.isSingleLanguage}
                  <option value="auto">Auto (match language)</option>
                {/if}
                {#if selectedLangs.includes('rust')}
                  <option value="rust">Rust</option>
                {/if}
                {#if selectedLangs.includes('clang')}
                  <option value="seagreen">Clang</option>
                {/if}
                {#if selectedLangs.includes('zig')}
                  <option value="zig">Zig</option>
                {/if}
                {#if selectedLangs.includes('swift')}
                  <option value="swift">Swift</option>
                {/if}
              </optgroup>
            </select>
          </div>

          <div class="setting-row">
            <label for="wiz-font-size">Font Size</label>
            <div class="input-group">
              <input id="wiz-font-size" type="number" min="8" max="32" bind:value={draftSettings.font_size} />
              <span class="unit">px</span>
            </div>
          </div>

          <div class="setting-row">
            <label for="wiz-font-family">Font</label>
            <select id="wiz-font-family" bind:value={draftSettings.font_family}>
              {#each fontFamilies as f}
                <option value={f}>{f}</option>
              {/each}
            </select>
          </div>

          <div class="setting-row">
            <label for="wiz-tab-size">Tab Size</label>
            <select id="wiz-tab-size" bind:value={draftSettings.tab_size}>
              <option value={0}>Auto</option>
              <option value={2}>2 spaces</option>
              <option value={4}>4 spaces</option>
              <option value={8}>8 spaces</option>
            </select>
          </div>

        </div>

        <div
          class="preview-box"
          style="font-size: {draftSettings.font_size}px; font-family: '{draftSettings.font_family}', monospace;"
        >
          <span class="preview-kw">fn</span> <span class="preview-fn">main</span>() &#123;<br/>
          &nbsp;&nbsp;&nbsp;&nbsp;<span class="preview-mac">println!</span>(<span class="preview-str">"Hello, playground!"</span>);<br/>
          &#125;
        </div>

        {#if draftSettings.font_size !== 13 || draftSettings.font_family !== 'Menlo' || draftSettings.tab_size !== 0 || draftSettings.theme !== 'system'}
        <button class="btn btn-secondary btn-reset" onclick={() => {
          draftSettings.font_size = 13
          draftSettings.font_family = 'Menlo'
          draftSettings.tab_size = 0
          draftSettings.theme = 'system'
        }}>Reset Defaults</button>
        {/if}
      </div>

    <!-- ═══════════ Panel: Books ═══════════ -->
    {:else if currentPanel === 'books'}
      <div class="step-content">
        <h2 class="step-heading">{singleBookName ?? 'Example Books'}</h2>
        <p class="step-desc">{mode === 'wizard'
          ? (bookCount === 1 ? `Load ${singleBookName} examples to learn from.` : 'Load book examples to learn from. Each book creates read-only reference projects.')
          : (bookCount === 1 ? `Manage ${singleBookName} examples.` : 'Manage loaded book examples.')}</p>

        {#if availableBooks.length === 0}
          <div class="no-books">
            <p>No books are available for your selected languages.</p>
          </div>
        {:else}
          <div class="book-list">
            {#each availableBooks as lang}
              {@const book = lang.book!}
              <button
                class="book-card"
                class:selected={booksChecked.includes(lang.type)}
                onclick={() => toggleBook(lang.type)}
              >
                <span class="book-icon">📖</span>
                <div class="book-info">
                  <span class="book-name">{book.commandLabel}</span>
                  <span class="book-desc">Read-only reference projects with example code</span>
                </div>
                <div class="check-mark">
                  {#if booksChecked.includes(lang.type)}
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                      <circle cx="8" cy="8" r="7" fill="var(--accent)" stroke="var(--accent)" stroke-width="1"/>
                      <path d="M4.5 8l2.5 2.5 4.5-5" stroke="#fff" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
                    </svg>
                  {:else}
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                      <circle cx="8" cy="8" r="7" stroke="var(--border-strong)" stroke-width="1"/>
                    </svg>
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>

    <!-- ═══════════ Panel: Finish (wizard only) ═══════════ -->
    {:else if currentPanel === 'finish'}
      <div class="step-content finish-step">
        <div class="finish-icon">🚀</div>
        <h2 class="step-heading">You're All Set!</h2>

        <div class="summary">
          {#if !edition.isSingleLanguage}
          <div class="summary-row">
            <span class="summary-label">Languages</span>
            <span class="summary-value">
              {selectedLangs.map(t => getLang(t as ProjectType).label).join(', ')}{toolchainSkipped ? ' — Rust toolchain skipped' : ''}
            </span>
          </div>
          {:else}
          <div class="summary-row">
            <span class="summary-label">Language</span>
            <span class="summary-value" class:warn={toolchainSkipped}>
              {editionLang!.label}{toolchainSkipped ? ' (toolchain skipped)' : ''}
            </span>
          </div>
          {/if}
          <div class="summary-row">
            <span class="summary-label">Theme</span>
            <span class="summary-value">{({ system: 'System', auto: 'Auto (match language)', dark: 'Dark', light: 'Light', rust: 'Rust', seagreen: 'Clang', zig: 'Zig', swift: 'Swift' } as Record<string, string>)[draftSettings.theme] ?? draftSettings.theme}</span>
          </div>
          {#if booksChecked.length > 0}
            {#each booksChecked as btype, i}
              {@const bookLabel = getLang(btype as ProjectType).book?.commandLabel}
              {#if bookLabel}
              <div class="summary-row">
                <span class="summary-label">{i === 0 ? (booksChecked.length === 1 ? 'Book' : 'Books') : ''}</span>
                <span class="summary-value">{bookLabel}</span>
              </div>
              {/if}
            {/each}
          {/if}
        </div>
      </div>
    {/if}
  </div>

  <!-- Footer -->
  <div class="modal-footer">
    {#if mode === 'wizard'}
      {#if step > 1}
        <button class="btn btn-secondary" onclick={back}>Back</button>
      {:else}
        <div></div>
      {/if}
      <div class="footer-right">
        {#if step < totalSteps}
          <button class="btn btn-primary" onclick={next} disabled={!hasSelection}>Next</button>
        {:else}
          <button class="btn btn-primary btn-finish" onclick={finish} disabled={!hasSelection}>Get Started</button>
        {/if}
      </div>
    {:else}
      <div></div>
      <div class="footer-right">
        <button class="btn btn-secondary" onclick={cancel}>Cancel</button>
        <button class="btn btn-primary" onclick={apply} disabled={!hasSelection}>Apply</button>
        <button class="btn btn-secondary" onclick={finish}>Done</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0; z-index: 299;
    background: rgba(0,0,0,0.55);
    backdrop-filter: blur(2px);
  }

  .modal {
    position: fixed;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    z-index: 300;
    width: min(520px, calc(100vw - 40px));
    max-height: calc(100vh - 80px);
    display: flex; flex-direction: column;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    box-shadow: 0 24px 80px rgba(0,0,0,0.7), 0 4px 16px rgba(0,0,0,0.4);
    overflow: hidden;
  }

  /* ── Header ── */
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .header-left { display: flex; align-items: center; gap: 8px; }
  .rs-badge {
    font-size: 8px; font-weight: 800; letter-spacing: 0.04em;
    background: var(--rust-orange); color: #fff;
    border-radius: 3px; padding: 2px 4px; line-height: 1.3;
  }
  .modal-title { font-size: 14px; font-weight: 600; color: var(--text); }
  .close-btn {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%; color: var(--text-tertiary);
    transition: background 0.1s, color 0.1s;
  }
  .close-btn:hover { background: var(--bg-hover); color: var(--text); }

  /* ── Tab bar (settings mode) ── */
  .tab-bar {
    display: flex;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    background: var(--bg-elevated);
  }
  .tab {
    flex: 1;
    display: flex; align-items: center; justify-content: center;
    padding: 9px 12px;
    font-size: 12px; font-weight: 600;
    color: var(--text-tertiary);
    border: none; background: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s, background 0.15s;
  }
  .tab:hover { color: var(--text-secondary); background: rgba(255,255,255,0.03); }
  .tab.active {
    color: var(--text);
    border-bottom-color: var(--accent);
  }

  /* ── Step indicator (wizard mode) ── */
  .step-bar {
    display: flex; align-items: center; justify-content: center;
    padding: 16px 24px 12px;
    gap: 0;
    flex-shrink: 0;
  }
  .step-dot {
    display: flex; flex-direction: column; align-items: center; gap: 4px;
    background: none; border: none; cursor: pointer; padding: 0;
    min-width: 60px;
  }
  .step-dot:disabled { cursor: default; }
  .dot {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%;
    font-size: 11px; font-weight: 700;
    background: var(--bg-input);
    color: var(--text-tertiary);
    border: 1.5px solid var(--border);
    transition: all 0.2s;
  }
  .step-dot.active .dot {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
  }
  .step-dot.done .dot {
    background: var(--green);
    color: #fff;
    border-color: var(--green);
    font-size: 10px;
  }
  .step-label {
    font-size: 9px; font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .step-dot.active .step-label { color: var(--accent); }
  .step-dot.done .step-label { color: var(--green); }
  /* Skipped state — user advanced past the toolchain step without installing.
     Red dot with "!" inside, red label. Visually distinct from "done". */
  .step-dot.skipped .dot {
    background: var(--red, #d44);
    color: #fff;
    border-color: var(--red, #d44);
    font-size: 11px;
  }
  .step-dot.skipped .step-label { color: var(--red, #d44); }
  .step-line {
    flex: 1; height: 1.5px;
    background: var(--border);
    margin: 0 4px;
    margin-bottom: 16px;
    transition: background 0.2s;
  }
  .step-line.done { background: var(--green); }
  .step-line.skipped { background: var(--red, #d44); }

  /* ── Body ── */
  .modal-body {
    flex: 1; overflow-y: auto;
    padding: 16px 24px 20px;
  }

  .step-content {
    display: flex; flex-direction: column; gap: 16px;
  }
  .step-heading {
    font-size: 16px; font-weight: 600; color: var(--text);
    margin: 0;
  }
  .step-desc {
    font-size: 12px; color: var(--text-secondary);
    margin: -8px 0 0 0; line-height: 1.5;
  }

  /* ── Language cards ── */
  .lang-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .selection-label {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 10px;
  }
  .selection-label.error {
    color: var(--red, #e44);
  }
  .lang-card {
    display: flex; align-items: center; gap: 10px;
    padding: 12px 14px;
    background: var(--bg-input);
    border: 1.5px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
    position: relative;
  }
  .lang-card:hover { border-color: var(--border-strong); background: var(--bg-hover); }
  .lang-card.selected { border-color: var(--accent); background: rgba(var(--accent-rgb, 229, 115, 0), 0.06); }
  .lang-badge {
    font-size: 8px; font-weight: 800; letter-spacing: 0.04em;
    color: #fff; border-radius: 3px; padding: 2px 5px; line-height: 1.3;
  }
  .lang-badge.badge-rust { background: var(--rust-orange); }
  .lang-badge.badge-clang { background: #4a9; }
  .lang-badge.badge-zig { background: #f7a41d; font-size: 7px; }
  .lang-badge.badge-swift { background: #f05138; }
  .lang-name { font-size: 13px; font-weight: 600; color: var(--text); flex: 1; }
  .exp-tag {
    font-size: 7px; font-weight: 700; letter-spacing: 0.03em;
    text-transform: uppercase;
    background: rgba(247, 164, 29, 0.15);
    color: #f7a41d;
    border: 1px solid rgba(247, 164, 29, 0.3);
    border-radius: 3px;
    padding: 0.5px 3px;
    line-height: 1.3;
  }
  .check-mark {
    flex-shrink: 0;
  }

  /* ── Toolchain list ── */
  .toolchain-list {
    display: flex; flex-direction: column; gap: 16px;
  }
  .toolchain-section {
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px 14px;
    display: flex; flex-direction: column; gap: 8px;
  }
  .tc-header {
    display: flex; align-items: center; gap: 8px;
  }
  .tc-name { font-size: 13px; font-weight: 600; color: var(--text); flex: 1; }
  .tc-status { font-size: 11px; font-weight: 600; }
  .tc-status.ok { color: var(--green); }
  .tc-status.warn { color: #e8a820; }
  .tc-status.missing { color: var(--red, #d42020); }

  .status-card {
    padding: 10px 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: rgba(0,0,0,0.15);
    display: flex; flex-direction: column; gap: 2px;
    margin-bottom: 4px;
  }
  .status-card.healthy { border-color: rgba(46, 160, 67, 0.4); }
  .status-card.unhealthy { border-color: rgba(212, 32, 32, 0.4); }
  .status-headline { font-size: 12px; font-weight: 600; }
  .status-headline.ok { color: var(--green); }
  .status-headline.missing { color: var(--red, #d42020); }
  .status-headline.warn { color: #e8a820; }
  .status-sub {
    font-size: 11px; color: var(--text-secondary);
    margin: 2px 0 0 0; line-height: 1.4;
  }
  .status-sub code {
    font-family: var(--font-mono); font-size: 10px;
    background: rgba(0,0,0,0.25);
    padding: 1px 4px; border-radius: 3px;
  }
  .recheck-btn { align-self: flex-start; margin-top: 4px; }

  .cargo-path-row {
    display: flex; align-items: center; gap: 12px;
    margin-top: 4px;
  }
  .cargo-path-row label {
    font-size: 12px; color: var(--text-secondary); font-weight: 600;
    flex-shrink: 0;
  }

  .checking {
    display: flex; align-items: center; gap: 12px;
    padding: 20px 0;
    color: var(--text-secondary); font-size: 13px;
    justify-content: center;
  }
  .spinner {
    width: 18px; height: 18px;
    border: 2px solid rgba(255,255,255,0.15);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .detail-grid { display: flex; flex-direction: column; gap: 2px; }
  .detail-row {
    display: flex; align-items: center; gap: 10px;
    padding: 4px 8px; border-radius: 4px; font-size: 12px;
  }
  .detail-icon { font-size: 8px; width: 12px; text-align: center; }
  .detail-icon.ok { color: var(--green); }
  .detail-icon.warn { color: #e8a820; }
  .detail-icon.missing { color: var(--red, #d42020); }
  .detail-label {
    font-family: var(--font-mono); font-weight: 600;
    color: var(--text-secondary); width: 70px; flex-shrink: 0;
  }
  .detail-value {
    font-family: var(--font-mono); color: var(--text-tertiary);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }

  .install-section {
    background: rgba(0,0,0,0.15);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px 12px;
  }
  .install-section.warn-note {
    border-color: rgba(232, 168, 32, 0.3);
    background: rgba(232, 168, 32, 0.06);
  }
  .install-text { font-size: 11px; color: var(--text-secondary); margin: 0 0 6px 0; }
  .install-cmd {
    display: block;
    font-family: var(--font-mono); font-size: 11px;
    color: var(--text); background: rgba(0,0,0,0.2);
    border: 1px solid var(--border);
    border-radius: 4px; padding: 6px 8px; margin: 2px 0;
    user-select: all; word-break: break-all;
  }
  .install-hint { font-size: 10px; color: var(--text-tertiary); margin: 6px 0 0 0; }
  .link-btn {
    background: none; border: none; padding: 0;
    color: var(--accent); font-size: 10px;
    text-decoration: underline; cursor: pointer;
  }
  .link-btn:hover { filter: brightness(1.2); }

  /* ── Appearance ── */
  .appearance-grid {
    display: flex; flex-direction: column; gap: 12px;
  }
  .setting-row {
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
  }
  .setting-row label {
    font-size: 13px; color: var(--text-secondary); font-weight: 500; flex-shrink: 0;
  }
  select, input[type="number"] {
    font-family: var(--font-mono); font-size: 12px;
    background: var(--bg-input); color: var(--text);
    border: 1px solid var(--border); border-radius: var(--radius-xs);
    padding: 4px 8px; outline: none;
    min-width: 140px; cursor: pointer;
    color-scheme: var(--color-scheme, dark);
  }
  select:focus, input[type="number"]:focus { border-color: var(--accent); }
  select option { background: var(--bg-elevated); color: var(--text); }
  select optgroup { background: var(--bg-elevated); color: var(--text-secondary); font-style: normal; }
  .input-group { display: flex; align-items: center; gap: 4px; }
  .input-group input { width: 60px; text-align: right; min-width: 60px; }
  .unit { font-size: 11px; color: var(--text-tertiary); }

  .path-input {
    font-family: var(--font-mono); font-size: 11px;
    background: var(--bg-input); color: var(--text);
    border: 1px solid var(--border); border-radius: var(--radius-xs);
    padding: 4px 8px; outline: none;
    flex: 1; min-width: 0;
  }
  .path-input:focus { border-color: var(--accent); }

  .preview-box {
    background: var(--bg-sidebar);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 12px 14px;
    line-height: 1.7;
    color: var(--text);
    white-space: pre;
  }
  .preview-kw  { color: #fc5fa3; }
  .preview-fn  { color: #67b7a4; }
  .preview-mac { color: #b281eb; }
  .preview-str { color: #fc6a5d; }

  /* ── Books ── */
  .book-list {
    display: flex; flex-direction: column; gap: 10px;
  }
  .book-card {
    display: flex; align-items: center; gap: 12px;
    padding: 14px 16px;
    background: var(--bg-input);
    border: 1.5px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
    text-align: left;
  }
  .book-card:hover { border-color: var(--border-strong); background: var(--bg-hover); }
  .book-card.selected { border-color: var(--accent); }
  .book-icon { font-size: 20px; }
  .book-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .book-name { font-size: 13px; font-weight: 600; color: var(--text); }
  .book-desc { font-size: 11px; color: var(--text-tertiary); }
  .no-books {
    text-align: center; padding: 24px;
    color: var(--text-tertiary); font-size: 13px;
  }

  /* ── Finish ── */
  .finish-step { align-items: center; text-align: center; }
  .finish-icon { font-size: 40px; margin-bottom: 4px; }
  .summary {
    width: 100%;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 18px;
    display: flex; flex-direction: column; gap: 8px;
    text-align: left;
  }
  .summary-row {
    display: flex; justify-content: space-between; gap: 12px;
    font-size: 12px;
  }
  .summary-label { color: var(--text-tertiary); font-weight: 600; }
  .summary-value { color: var(--text); font-weight: 500; text-align: right; }
  /* Toolchain skipped state — red text on the language summary value to
     match the red step indicator from the wizard. Visually says "this is
     not OK, fix it later." */
  .summary-value.warn { color: var(--red, #d44); }

  /* ── Footer ── */
  .modal-footer {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }
  .footer-right { display: flex; gap: 8px; }

  .btn {
    font-size: 12px; font-weight: 600;
    padding: 6px 18px; border-radius: 6px;
    cursor: pointer; transition: background 0.1s;
  }
  .btn-secondary {
    color: var(--text-secondary);
    background: var(--bg-input); border: 1px solid var(--border);
  }
  .btn-secondary:hover { background: var(--bg-hover); border-color: var(--border-strong); }
  .btn-primary { color: #fff; background: var(--accent); border: 1px solid var(--accent); }
  .btn-primary:hover { filter: brightness(1.15); }
  .btn-finish { padding: 8px 28px; font-size: 13px; }
  .btn-reset { margin-top: 12px; font-size: 11px; padding: 4px 12px; opacity: 0.7; }

  /* External help links footer in toolchain panel — visible in both wizard
     and settings mode. Compact, unobtrusive, but discoverable. */
  .help-links {
    margin-top: 16px;
    padding: 12px 14px;
    background: rgba(0,0,0,0.15);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .help-links-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }
  .help-links-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px 16px;
  }
  .help-links-list li {
    font-size: 11px;
    color: var(--text-tertiary);
    line-height: 1.6;
  }
  .help-links .link-btn {
    background: none; border: none; padding: 0;
    color: var(--accent);
    font-size: 11px;
    font-family: inherit;
    cursor: pointer;
    text-decoration: none;
  }
  .help-links .link-btn:hover { text-decoration: underline; }
</style>
