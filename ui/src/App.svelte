<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke, Channel } from '@tauri-apps/api/core'
  import Sidebar from './lib/Sidebar.svelte'
  import TabBar from './lib/TabBar.svelte'
  import Editor from './lib/Editor.svelte'
  import Output from './lib/Output.svelte'
  import type { RunBlock, OutputLine } from './lib/Output.svelte'

  // ── Constants ────────────────────────────────────────────────────────────────
  const CARGO_TAB = 'Cargo.toml'

  // ── Playground list ──────────────────────────────────────────────────────────
  let playgrounds: string[] = $state([])

  // ── Tab state ────────────────────────────────────────────────────────────────
  let openTabs:  string[]               = $state([])
  let activeTab: string | null          = $state(null)
  let tabCode:   Record<string, string> = $state({})
  let dirtyTabs: string[]               = $state([])

  let currentCode     = $derived(activeTab ? (tabCode[activeTab] ?? '') : '')
  let editorLanguage  = $derived(activeTab === CARGO_TAB ? 'ini' : 'rust')

  // ── Per-tab run blocks & status ──────────────────────────────────────────────
  let tabRuns:     Record<string, RunBlock[]> = $state({})
  let tabRunCount: Record<string, number>     = $state({})

  let currentRuns   = $derived(activeTab ? (tabRuns[activeTab] ?? []) : [])
  let lastRun       = $derived(currentRuns.at(-1))
  let currentStatus = $derived(
    lastRun?.status === 'compiling' ? 'compiling' :
    lastRun?.status === 'running'   ? 'running'   :
    lastRun?.status === 'error'     ? 'error'      :
    'idle'
  )
  let isRunning = $derived(currentStatus === 'compiling' || currentStatus === 'running')

  // ── Toolchain + Cargo.toml (sidebar) ─────────────────────────────────────────
  let cargoToml:     string                              = $state('')
  let toolchainInfo: { path: string; version: string }  = $state({ path: '', version: '' })

  // ── New playground binding ────────────────────────────────────────────────────
  let creatingNew: boolean = $state(false)

  // ── Lifecycle ────────────────────────────────────────────────────────────────
  onMount(async () => {
    playgrounds    = await invoke<string[]>('list_playgrounds')
    cargoToml      = await invoke<string>('get_cargo_toml').catch(() => '')
    toolchainInfo  = await invoke<{ path: string; version: string }>('get_toolchain_info')
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  })

  // ── Keyboard shortcuts ───────────────────────────────────────────────────────
  function handleKey(e: KeyboardEvent) {
    if (e.metaKey && e.key === 'n') { e.preventDefault(); requestNewPlayground() }
    if (e.metaKey && e.key === 'r') { e.preventDefault(); run() }
    if (e.metaKey && e.key === 's') { e.preventDefault(); save() }
    if (e.metaKey && e.key === '.') { e.preventDefault(); stop() }
    if (e.metaKey && e.key === 'w') { e.preventDefault(); closeTab(activeTab) }
  }

  // ── Tab management ───────────────────────────────────────────────────────────

  async function openTab(name: string) {
    if (!openTabs.includes(name)) {
      const code = name === CARGO_TAB
        ? await invoke<string>('get_cargo_toml')
        : await invoke<string>('load_playground', { name })
      tabCode  = { ...tabCode,  [name]: code }
      openTabs = [...openTabs, name]
    }
    activeTab = name
  }

  function closeTab(name: string | null) {
    if (!name) return
    // window.confirm() is not rendered in Tauri's WKWebView — always returns false.
    // Close immediately; the source file stays on disk, only in-editor edits are lost.
    dirtyTabs = dirtyTabs.filter(n => n !== name)
    const idx = openTabs.indexOf(name)
    openTabs = openTabs.filter(n => n !== name)

    const { [name]: _c, ...restCode  } = tabCode
    const { [name]: _r, ...restRuns  } = tabRuns
    const { [name]: _n, ...restCount } = tabRunCount
    tabCode     = restCode
    tabRuns     = restRuns
    tabRunCount = restCount

    if (activeTab === name) {
      activeTab = openTabs[idx] ?? openTabs[idx - 1] ?? null
    }
  }

  // ── File operations ──────────────────────────────────────────────────────────

  async function save() {
    if (!activeTab) return
    if (activeTab === CARGO_TAB) {
      await invoke('save_cargo_toml', { content: tabCode[CARGO_TAB] })
      cargoToml = tabCode[CARGO_TAB]
    } else {
      await invoke('save_playground', { name: activeTab, content: tabCode[activeTab] })
    }
    dirtyTabs = dirtyTabs.filter(n => n !== activeTab)
  }

  function onCodeChange(newCode: string) {
    if (!activeTab) return
    tabCode = { ...tabCode, [activeTab]: newCode }
    if (!dirtyTabs.includes(activeTab)) {
      dirtyTabs = [...dirtyTabs, activeTab]
    }
  }

  // ── RunBlock helpers ─────────────────────────────────────────────────────────

  function updateLastRun(name: string, updater: (r: RunBlock) => RunBlock) {
    const runs = tabRuns[name]
    if (!runs?.length) return
    const updated = [...runs]
    updated[updated.length - 1] = updater(updated[updated.length - 1])
    tabRuns = { ...tabRuns, [name]: updated }
  }

  // ── Run / Stop ───────────────────────────────────────────────────────────────

  async function run() {
    if (!activeTab || isRunning || activeTab === CARGO_TAB) return
    const name = activeTab
    await save()

    // Collapse all previous runs for this tab
    const existing = tabRuns[name] ?? []
    const collapsed = existing.map(r => ({ ...r, collapsed: true }))

    const runNum = (tabRunCount[name] ?? 0) + 1
    tabRunCount = { ...tabRunCount, [name]: runNum }

    const now = new Date()
    const startedAt = now.toLocaleTimeString('en-US', {
      hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false
    })

    const newBlock: RunBlock = {
      runNum,
      command: `cargo run --bin ${name}`,
      startedAt,
      status: 'compiling',
      exitCode: null,
      compilerLines: [],
      programLines: [],
      collapsed: false,
      programStarted: false,
    }

    tabRuns = { ...tabRuns, [name]: [...collapsed, newBlock] }

    const channel = new Channel()
    channel.onmessage = (msg: any) => {
      if (msg.stream === 'complete') {
        updateLastRun(name, r => ({
          ...r,
          status: msg.code === 0 ? 'success' : 'error',
          exitCode: msg.code,
        }))
      } else if (msg.stream === 'stdout') {
        updateLastRun(name, r => ({
          ...r,
          programStarted: true,
          status: 'running',
          programLines: [...r.programLines, { stream: 'stdout', line: msg.line }],
        }))
      } else if (msg.stream === 'stderr') {
        updateLastRun(name, r => ({
          ...r,
          ...(r.programStarted
            ? { programLines: [...r.programLines, { stream: 'stderr', line: msg.line }] }
            : { compilerLines: [...r.compilerLines, { stream: 'stderr', line: msg.line }] }),
        }))
      }
    }

    try {
      await invoke('run_playground', { name, onOutput: channel })
    } catch (e) {
      updateLastRun(name, r => ({
        ...r,
        status: 'error',
        compilerLines: [...r.compilerLines, { stream: 'stderr', line: String(e) }],
      }))
    }
  }

  function stop() {
    if (!activeTab) return
    updateLastRun(activeTab, r => ({ ...r, status: 'error', exitCode: -1 }))
  }

  // ── Playground CRUD ──────────────────────────────────────────────────────────

  function requestNewPlayground() {
    creatingNew = true
  }

  async function onNewPlayground(e: CustomEvent<string>) {
    const name = e.detail
    try {
      await invoke('new_playground', { name })
      playgrounds = await invoke<string[]>('list_playgrounds')
      await openTab(name)
    } catch (err) {
      console.error('Failed to create playground:', err)
    }
  }

  async function onRename(e: CustomEvent<{ old: string; new: string }>) {
    const { old: oldName, new: newName } = e.detail
    await invoke('rename_playground', { oldName, newName })
    playgrounds = await invoke<string[]>('list_playgrounds')

    if (openTabs.includes(oldName)) {
      const { [oldName]: code,  ...restCode  } = tabCode
      const { [oldName]: runs,  ...restRuns  } = tabRuns
      const { [oldName]: count, ...restCount } = tabRunCount

      tabCode     = { ...restCode,  [newName]: code }
      tabRuns     = { ...restRuns,  [newName]: runs  ?? [] }
      tabRunCount = { ...restCount, [newName]: count ?? 0  }

      openTabs  = openTabs.map(n => n === oldName ? newName : n)
      if (activeTab === oldName) activeTab = newName
      if (dirtyTabs.includes(oldName)) {
        dirtyTabs = [...dirtyTabs.filter(n => n !== oldName), newName]
      }
    }
  }

  async function onDelete(e: CustomEvent<string>) {
    const name = e.detail
    if (!confirm(`Delete playground "${name}"? This cannot be undone.`)) return
    await invoke('delete_playground', { name })
    playgrounds = await invoke<string[]>('list_playgrounds')
    closeTab(name)
    dirtyTabs = dirtyTabs.filter(n => n !== name)
  }

  async function onDuplicate(e: CustomEvent<string>) {
    const newName = await invoke<string>('duplicate_playground', { name: e.detail })
    playgrounds = await invoke<string[]>('list_playgrounds')
    await openTab(newName)
  }

  async function onEditCargo() {
    cargoToml = await invoke<string>('get_cargo_toml').catch(() => cargoToml)
    await openTab(CARGO_TAB)
  }

  function onToggle(e: CustomEvent<number>) {
    if (!activeTab) return
    const runNum = e.detail
    const runs = tabRuns[activeTab] ?? []
    tabRuns = {
      ...tabRuns,
      [activeTab]: runs.map(r => r.runNum === runNum ? { ...r, collapsed: !r.collapsed } : r),
    }
  }

  function onClear() {
    if (!activeTab) return
    tabRuns     = { ...tabRuns,     [activeTab]: [] }
    tabRunCount = { ...tabRunCount, [activeTab]: 0  }
  }

  // ── Toolchain label ───────────────────────────────────────────────────────────
  // Show just the version string, e.g. "cargo 1.78.0" → "1.78.0"
  let toolchainLabel = $derived(
    toolchainInfo.version
      ? (toolchainInfo.version.match(/\d+\.\d+\.\d+/)?.[0] ?? toolchainInfo.version)
      : '…'
  )
</script>

<div class="app">
  <!-- ── Toolbar ──────────────────────────────────────────────────────────────── -->
  <header class="toolbar">
    <div class="toolbar-left">
      <span class="app-badge">RS</span>
      <span class="app-name">Rust Playground</span>
    </div>

    <div class="toolbar-center">
      <!-- Toolchain info pill — display only -->
      <span class="toolchain-pill" title={toolchainInfo.path}>
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
          <circle cx="5" cy="5" r="4" stroke="currentColor" stroke-width="1.2"/>
          <path d="M5 3v2.5l1.5 1" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
        </svg>
        cargo {toolchainLabel}
      </span>
    </div>

    <div class="toolbar-right">
      {#if currentStatus !== 'idle' && currentStatus !== 'error'}
        <span class="status-label" class:running={isRunning}>
          {currentStatus === 'compiling' ? 'Compiling…' : 'Running…'}
        </span>
      {/if}
      {#if currentStatus === 'error'}
        <span class="status-label error">Build failed</span>
      {/if}

      <!-- Save button — lives on the right alongside Run/Stop -->
      {#if activeTab}
        <button
          class="btn btn-save"
          onclick={save}
          disabled={!dirtyTabs.includes(activeTab)}
          title="Save (⌘S)"
        >
          <svg width="11" height="11" viewBox="0 0 12 12" fill="none">
            <path d="M2 1h7l2 2v8H2V1z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" fill="none"/>
            <rect x="4" y="7" width="4" height="4" rx="0.5" stroke="currentColor" stroke-width="1.3" fill="none"/>
            <rect x="3.5" y="1" width="5" height="3" rx="0.5" stroke="currentColor" stroke-width="1.3" fill="none"/>
          </svg>
          Save
        </button>
      {/if}

      {#if isRunning}
        <button class="btn btn-stop" onclick={stop}>
          <svg width="10" height="10" viewBox="0 0 10 10"><rect width="10" height="10" rx="2" fill="currentColor"/></svg>
          Stop
        </button>
      {:else}
        <button class="btn btn-run" onclick={run} disabled={!activeTab || activeTab === CARGO_TAB}>
          <svg width="10" height="12" viewBox="0 0 10 12"><polygon points="0,0 10,6 0,12" fill="currentColor"/></svg>
          Run
        </button>
      {/if}
    </div>
  </header>

  <!-- ── Main layout ───────────────────────────────────────────────────────────── -->
  <div class="main">
    <Sidebar
      {playgrounds}
      selected={activeTab}
      {dirtyTabs}
      bind:creatingNew
      {cargoToml}
      on:select={(e) => openTab(e.detail)}
      on:new={onNewPlayground}
      on:rename={onRename}
      on:delete={onDelete}
      on:duplicate={onDuplicate}
      on:editcargo={onEditCargo}
    />

    <div class="editor-area">
      <TabBar
        tabs={openTabs}
        active={activeTab}
        {dirtyTabs}
        on:activate={(e) => openTab(e.detail)}
        on:close={(e) => closeTab(e.detail)}
      />

      <div class="editor-wrap">
        {#if activeTab}
          <Editor
            code={currentCode}
            language={editorLanguage}
            on:change={(e) => onCodeChange(e.detail)}
          />
        {:else}
          <div class="empty-state">
            <div class="empty-icon">
              <svg width="48" height="48" viewBox="0 0 48 48" fill="none" opacity="0.2">
                <rect x="8" y="4" width="28" height="36" rx="4" stroke="currentColor" stroke-width="2"/>
                <path d="M14 14h16M14 20h12M14 26h8" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                <polygon points="32,30 44,38 32,46" fill="currentColor"/>
              </svg>
            </div>
            <p class="empty-title">No playground open</p>
            <p class="empty-hint">
              Select one from the sidebar or
              <button class="link-btn" onclick={requestNewPlayground}>create a new one</button>
            </p>
            <div class="shortcut-grid">
              <span class="shortcut-key">⌘N</span><span class="shortcut-desc">New playground</span>
              <span class="shortcut-key">⌘R</span><span class="shortcut-desc">Run</span>
              <span class="shortcut-key">⌘S</span><span class="shortcut-desc">Save</span>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <Output
      runs={currentRuns}
      status={currentStatus}
      on:toggle={onToggle}
      on:clear={onClear}
    />
  </div>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  /* ── Toolbar ── */
  .toolbar {
    display: flex;
    align-items: center;
    height: var(--toolbar-height);
    padding: 0 12px;
    background: var(--bg-sidebar);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    position: relative;
    gap: 12px;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .toolbar-center {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    justify-content: flex-end;
    min-width: 0;
  }

  .app-badge {
    font-size: 8px;
    font-weight: 800;
    background: var(--rust-orange);
    color: #fff;
    border-radius: 3px;
    padding: 2px 4px;
    line-height: 1.3;
    letter-spacing: 0.03em;
  }

  .app-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .status-label {
    font-size: 11px;
    color: var(--text-tertiary);
    letter-spacing: 0.02em;
  }
  .status-label.running { color: var(--green); }
  .status-label.error   { color: var(--red); }

  /* ── Toolbar buttons ── */
  .btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 12px;
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--radius-sm);
    transition: background 0.12s, opacity 0.12s;
  }

  .btn-save {
    background: rgba(255,255,255,0.08);
    color: var(--text);
    border: 1px solid var(--border-strong);
  }
  .btn-save:hover:not(:disabled) { background: rgba(255,255,255,0.14); }
  .btn-save:disabled { opacity: 0.3; cursor: not-allowed; }

  .btn-run {
    background: var(--accent);
    color: #fff;
  }
  .btn-run:hover:not(:disabled) { background: var(--accent-hover); }
  .btn-run:disabled { opacity: 0.3; cursor: not-allowed; }

  .btn-stop {
    background: #3a3a3c;
    color: var(--text-secondary);
  }
  .btn-stop:hover { background: var(--bg-elevated); }

  /* ── Toolchain pill ── */
  .toolchain-pill {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-tertiary);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 3px 8px;
    cursor: default;
    white-space: nowrap;
  }

  /* ── Main 3-panel layout ── */
  .main {
    display: flex;
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  /* ── Editor area ── */
  .editor-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  .editor-wrap {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  /* ── Empty state ── */
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--text-tertiary);
    padding: 40px;
  }

  .empty-icon { margin-bottom: 6px; }

  .empty-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .empty-hint {
    font-size: 13px;
    color: var(--text-tertiary);
    text-align: center;
  }

  .link-btn {
    background: none;
    color: var(--accent);
    text-decoration: underline;
    padding: 0;
    font-size: 13px;
    display: inline;
  }

  .shortcut-grid {
    display: grid;
    grid-template-columns: auto auto;
    gap: 4px 16px;
    margin-top: 16px;
    align-items: center;
  }

  .shortcut-key {
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-xs);
    padding: 2px 7px;
    color: var(--text-secondary);
    text-align: center;
    justify-self: end;
  }

  .shortcut-desc {
    font-size: 12px;
    color: var(--text-tertiary);
  }
</style>
