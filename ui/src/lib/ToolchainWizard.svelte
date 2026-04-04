<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { open as shellOpen } from '@tauri-apps/plugin-shell'
  import { allLanguages, getLang, type ProjectType } from './languages'
  import type { Settings } from './SettingsModal.svelte'
  import type { EditionConfig } from './editions'

  let {
    onclose,
    onapply,
    enabledLanguages = ['rust'],
    settings,
    projectSources = {},
    onthemechange,
    mode = 'wizard',
    edition,
  }: {
    onclose: (result: { enabledLanguages: string[]; booksToLoad: string[]; booksToRemove: string[]; settings?: Settings }) => void
    onapply?: (result: { enabledLanguages: string[]; booksToLoad: string[]; booksToRemove: string[]; settings: Settings }) => void
    enabledLanguages: string[]
    settings: Settings
    projectSources?: Record<string, string>
    onthemechange?: (theme: string) => void
    mode?: 'wizard' | 'settings'
    edition: EditionConfig
  } = $props()

  type ToolchainStatus = {
    wizard_completed: boolean
    all_good: boolean
    rustup: { installed: boolean; version: string | null }
    cargo: { installed: boolean; path: string; version: string | null }
    rustc: { installed: boolean; version: string | null }
    active_toolchain: string | null
    installed_toolchains: string[]
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
    allLanguages()
      .filter(l => l.book && loadedSourceTags.has(l.book.sourceTag))
      .map(l => l.type)
  )

  // Edition-aware labels for single-language editions
  const editionLang = edition.isSingleLanguage ? getLang(edition.languages![0] as ProjectType) : null
  const editionBookName = editionLang?.book?.commandLabel ?? null
  const toolchainLabel = edition.isSingleLanguage ? 'Toolchain' : 'Toolchains'
  const booksLabel = editionBookName ?? 'Books'

  // Edition-aware steps/tabs: single-language editions skip language picker
  const allWizardSteps = edition.isSingleLanguage
    ? [toolchainLabel, 'Appearance', ...(editionBookName ? [booksLabel] : []), 'Finish']
    : ['Languages', 'Toolchains', 'Appearance', 'Books', 'Finish']
  const allWizardPanels = edition.isSingleLanguage
    ? (['toolchains', 'appearance', ...(editionBookName ? ['books' as const] : []), 'finish'] as const)
    : (['languages', 'toolchains', 'appearance', 'books', 'finish'] as const)
  const wizardSteps = allWizardSteps
  const totalSteps = wizardSteps.length

  const allSettingsTabs: { id: typeof activeTab; label: string }[] = [
    ...(!edition.isSingleLanguage ? [{ id: 'languages' as const, label: 'Languages' }] : []),
    { id: 'toolchains', label: toolchainLabel },
    { id: 'appearance', label: 'Appearance' },
    ...(!edition.isSingleLanguage || editionBookName ? [{ id: 'books' as const, label: booksLabel }] : []),
  ]
  const settingsTabs = allSettingsTabs

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

  // Auto-run toolchain check when entering toolchains panel
  $effect(() => {
    if (currentPanel === 'toolchains' && !status && !checking) {
      runCheck()
    }
  })

  // ── Language toggle ───────────────────────────────────────────────────��
  function toggleLang(type: string) {
    if (selectedLangs.includes(type)) {
      selectedLangs = selectedLangs.filter(l => l !== type)
      booksChecked = booksChecked.filter(b => b !== type)
    } else {
      selectedLangs = [...selectedLangs, type]
    }
  }
  let hasSelection = $derived(selectedLangs.length > 0)
  let selectionLabel = $derived(
    hasSelection
      ? 'Selected: ' + selectedLangs.map(t => getLang(t as ProjectType).label).join(', ')
      : 'You must select at least one language'
  )

  // ── Toolchain check ───────────────────────────────────────────────────
  async function runCheck() {
    checking = true
    try {
      status = await invoke<ToolchainStatus>('check_toolchain')
    } catch (_) { /* ignore */ }
    finally { checking = false }
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
  function bookDiff() {
    const wasLoaded = allLanguages()
      .filter(l => l.book && loadedSourceTags.has(l.book.sourceTag))
      .map(l => l.type)
    const toLoad = booksChecked.filter(b => !wasLoaded.includes(b))
    const toRemove = wasLoaded.filter(b => !booksChecked.includes(b))
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
    'Menlo', 'Monaco', 'SF Mono', 'Courier New',
    'JetBrains Mono', 'Fira Code', 'Source Code Pro', 'IBM Plex Mono',
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
        <button
          class="step-dot"
          class:active={step === i + 1}
          class:done={step > i + 1}
          onclick={() => { if (i + 1 < step) step = (i + 1) as any }}
          disabled={i + 1 > step}
        >
          <span class="dot">{step > i + 1 ? '✓' : i + 1}</span>
          <span class="step-label">{label}</span>
        </button>
        {#if i < wizardSteps.length - 1}
          <div class="step-line" class:done={step > i + 1}></div>
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
              <span class="lang-badge {lang.badgeClass}">{lang.badge}</span>
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
        <p class="step-desc">{edition.isSingleLanguage ? `Checking if ${editionLang!.label.toLowerCase()} is installed.` : 'Checking if the required compilers are installed.'}</p>

        {#if checking}
          <div class="checking">
            <div class="spinner"></div>
            <span>Detecting toolchains…</span>
          </div>
        {:else if status}
          <div class="toolchain-list">
            {#each allLanguages().filter(l => selectedLangs.includes(l.type)) as lang}
              {@const ok = toolchainOk(lang.type)}
              <div class="toolchain-section">
                <div class="tc-header">
                  <span class="lang-badge {lang.badgeClass}" style="font-size: 7px">{lang.badge}</span>
                  <span class="tc-name">{lang.label}</span>
                  <span class="tc-status" class:ok={ok} class:missing={!ok}>
                    {ok ? '● Ready' : '○ Not found'}
                  </span>
                </div>

                {#if lang.type === 'rust'}
                  <div class="detail-grid">
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
                  </div>
                  {#if !status.all_good}
                    <div class="install-section">
                      <p class="install-text">Install Rust:</p>
                      <code class="install-cmd">curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh</code>
                      <p class="install-hint">Or visit <button class="link-btn" onclick={() => shellOpen('https://rustup.rs')}>rustup.rs</button></p>
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
      </div>

    <!-- ═══════════ Panel: Books ═══════════ -->
    {:else if currentPanel === 'books'}
      <div class="step-content">
        <h2 class="step-heading">{editionBookName ?? 'Example Books'}</h2>
        <p class="step-desc">{mode === 'wizard'
          ? (edition.isSingleLanguage ? `Load ${editionBookName} examples to learn from.` : 'Load book examples to learn from. Each book creates read-only reference projects.')
          : (edition.isSingleLanguage ? `Manage ${editionBookName} examples.` : 'Manage loaded book examples.')}</p>

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
              {selectedLangs.map(t => getLang(t as ProjectType).label).join(', ')}
            </span>
          </div>
          {:else}
          <div class="summary-row">
            <span class="summary-label">Language</span>
            <span class="summary-value">{editionLang!.label}</span>
          </div>
          {/if}
          <div class="summary-row">
            <span class="summary-label">Theme</span>
            <span class="summary-value">{({ system: 'System', auto: 'Auto (match language)', dark: 'Dark', light: 'Light', rust: 'Rust', seagreen: 'Clang', zig: 'Zig', swift: 'Swift' } as Record<string, string>)[draftSettings.theme] ?? draftSettings.theme}</span>
          </div>
          {#if booksChecked.length > 0}
            <div class="summary-row">
              <span class="summary-label">{edition.isSingleLanguage ? 'Book' : 'Books to load'}</span>
              <span class="summary-value">
                {booksChecked.map(t => getLang(t as ProjectType).book?.commandLabel).filter(Boolean).join(', ')}
              </span>
            </div>
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
  .step-line {
    flex: 1; height: 1.5px;
    background: var(--border);
    margin: 0 4px;
    margin-bottom: 16px;
    transition: background 0.2s;
  }
  .step-line.done { background: var(--green); }

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
  .tc-status.missing { color: var(--red, #d42020); }
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
</style>
