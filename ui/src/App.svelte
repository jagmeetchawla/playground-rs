<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke, Channel } from '@tauri-apps/api/core'
  import Sidebar from './lib/Sidebar.svelte'
  import TabBar from './lib/TabBar.svelte'
  import Editor from './lib/Editor.svelte'
  import Output from './lib/Output.svelte'

  type Status = 'idle' | 'checking' | 'compiling' | 'running' | 'error'
  type OutputLine = { stream: 'stdout' | 'stderr' | 'info'; line: string }

  // ── Playground list ──────────────────────────────────────────────────────────
  let playgrounds: string[] = $state([])

  // ── Tab state ────────────────────────────────────────────────────────────────
  // openTabs  = ordered list of open tab names
  // activeTab = which tab is currently visible in the editor
  // tabCode   = cached source code per tab (so switching is instant)
  // dirtyTabs = tabs with unsaved changes
  let openTabs:  string[]               = $state([])
  let activeTab: string | null          = $state(null)
  let tabCode:   Record<string, string> = $state({})
  let dirtyTabs: string[]               = $state([])

  // The code shown in the Editor is always the active tab's code
  let currentCode = $derived(activeTab ? (tabCode[activeTab] ?? '') : '')

  // ── Per-tab output & status ───────────────────────────────────────────────────
  // Each tab keeps its own console history and run status so switching tabs
  // never loses output from a previous run.
  let tabOutput: Record<string, OutputLine[]> = $state({})
  let tabStatus: Record<string, Status>       = $state({})

  // Derived views for whatever tab is currently visible
  let currentOutput = $derived(activeTab ? (tabOutput[activeTab] ?? []) : [])
  let currentStatus = $derived(activeTab ? (tabStatus[activeTab] ?? 'idle') : 'idle')
  let isRunning     = $derived(currentStatus === 'running' || currentStatus === 'compiling')

  // ── Lifecycle ────────────────────────────────────────────────────────────────
  onMount(async () => {
    playgrounds = await invoke<string[]>('list_playgrounds')
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  })

  // ── Keyboard shortcuts ───────────────────────────────────────────────────────
  function handleKey(e: KeyboardEvent) {
    if (e.metaKey && e.key === 'r') { e.preventDefault(); run() }
    if (e.metaKey && e.key === '.') { e.preventDefault(); stop() }
    if (e.metaKey && e.key === 's') { e.preventDefault(); save() }
    if (e.metaKey && e.key === 'n') { e.preventDefault(); requestNewPlayground() }
    if (e.metaKey && e.key === 'w') { e.preventDefault(); closeTab(activeTab) }
  }

  // ── Tab management ───────────────────────────────────────────────────────────

  /** Open a playground in a tab (or switch to it if already open). */
  async function openTab(name: string) {
    if (!openTabs.includes(name)) {
      // Load code from disk on first open
      const code = await invoke<string>('load_playground', { name })
      tabCode = { ...tabCode, [name]: code }
      openTabs = [...openTabs, name]
      // New tabs start with empty output and idle status — no reset needed,
      // missing key already means [] / 'idle' via the derived defaults above
    }
    activeTab = name
    // Do NOT clear output — each tab keeps its own history across switches
  }

  /** Close a tab. Prompts to save if dirty. */
  function closeTab(name: string | null) {
    if (!name) return
    if (dirtyTabs.includes(name)) {
      if (!confirm(`"${name}" has unsaved changes. Close anyway?`)) return
      dirtyTabs = dirtyTabs.filter(n => n !== name)
    }
    const idx = openTabs.indexOf(name)
    openTabs = openTabs.filter(n => n !== name)

    // Drop all per-tab state to keep memory lean
    const { [name]: _code,   ...restCode   } = tabCode
    const { [name]: _out,    ...restOut    } = tabOutput
    const { [name]: _status, ...restStatus } = tabStatus
    tabCode   = restCode
    tabOutput = restOut
    tabStatus = restStatus

    // Focus adjacent tab
    if (activeTab === name) {
      activeTab = openTabs[idx] ?? openTabs[idx - 1] ?? null
    }
  }

  // ── File operations ──────────────────────────────────────────────────────────

  async function save() {
    if (!activeTab) return
    await invoke('save_playground', { name: activeTab, content: tabCode[activeTab] })
    dirtyTabs = dirtyTabs.filter(n => n !== activeTab)
  }

  function onCodeChange(newCode: string) {
    if (!activeTab) return
    tabCode = { ...tabCode, [activeTab]: newCode }
    if (!dirtyTabs.includes(activeTab)) {
      dirtyTabs = [...dirtyTabs, activeTab]
    }
  }

  // ── Run / Stop ───────────────────────────────────────────────────────────────

  /** Append one line to a tab's output (helper keeps call sites tidy). */
  function pushLine(name: string, line: OutputLine) {
    tabOutput = { ...tabOutput, [name]: [...(tabOutput[name] ?? []), line] }
  }

  /** Update a tab's run status. */
  function setStatus(name: string, s: Status) {
    tabStatus = { ...tabStatus, [name]: s }
  }

  async function run() {
    if (!activeTab || currentStatus !== 'idle') return
    const name = activeTab   // capture — activeTab could change while awaiting
    await save()

    // ── Separator between runs ────────────────────────────────────────────────
    // If this tab already has output from a previous run, add a visual divider
    // so the history stays readable.
    if ((tabOutput[name] ?? []).length > 0) {
      pushLine(name, { stream: 'info', line: '──────────────────────────' })
    }
    pushLine(name, { stream: 'info', line: `▶  cargo run --bin ${name}` })
    setStatus(name, 'compiling')

    const channel = new Channel()
    channel.onmessage = (msg: any) => {
      if (msg.stream === 'complete') {
        setStatus(name, msg.code === 0 ? 'idle' : 'error')
      } else {
        pushLine(name, { stream: msg.stream, line: msg.line })
        if (msg.stream === 'stdout') setStatus(name, 'running')
      }
    }

    try {
      await invoke('run_playground', { name, onOutput: channel })
    } catch (e) {
      pushLine(name, { stream: 'stderr', line: String(e) })
      setStatus(name, 'error')
    }
  }

  function stop() {
    if (!activeTab) return
    // TODO: kill process — implement in v1.0 iteration 2
    pushLine(activeTab, { stream: 'info', line: '— stopped —' })
    setStatus(activeTab, 'idle')
  }

  // ── Playground CRUD ──────────────────────────────────────────────────────────

  async function requestNewPlayground() {
    const name = window.prompt('Playground name (lowercase letters, digits, underscores):')
    if (!name) return
    await invoke('new_playground', { name })
    playgrounds = await invoke<string[]>('list_playgrounds')
    await openTab(name)
  }

  async function onRename(e: CustomEvent<{ old: string; new: string }>) {
    const { old: oldName, new: newName } = e.detail
    await invoke('rename_playground', { oldName, newName })
    playgrounds = await invoke<string[]>('list_playgrounds')

    if (openTabs.includes(oldName)) {
      // Migrate code
      const { [oldName]: code,   ...restCode   } = tabCode
      const { [oldName]: out,    ...restOut    } = tabOutput
      const { [oldName]: status, ...restStatus } = tabStatus

      tabCode   = { ...restCode,   [newName]: code }
      tabOutput = { ...restOut,    [newName]: out ?? [] }
      tabStatus = { ...restStatus, [newName]: status ?? 'idle' }

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
    // closeTab handles cleaning up tabCode / tabOutput / tabStatus
    closeTab(name)
    dirtyTabs = dirtyTabs.filter(n => n !== name)
  }

  async function onDuplicate(e: CustomEvent<string>) {
    const newName = await invoke<string>('duplicate_playground', { name: e.detail })
    playgrounds = await invoke<string[]>('list_playgrounds')
    await openTab(newName)
  }

  // ── Status label ─────────────────────────────────────────────────────────────
  const statusLabel: Record<Status, string> = {
    idle: '', checking: 'Checking…', compiling: 'Compiling…',
    running: 'Running', error: 'Build failed',
  }
</script>

<div class="app">
  <!-- ── Toolbar ──────────────────────────────────────────────────────────────── -->
  <header class="toolbar">
    <div class="toolbar-left">
      <span class="app-icon">
        <!-- Small Rust crab-ish icon placeholder — just an RS badge -->
        <span class="app-badge">RS</span>
      </span>
      <span class="app-name">Rust Playground</span>
    </div>

    <div class="toolbar-center">
      {#if activeTab}
        <span class="toolbar-filename">{activeTab}.rs</span>
        {#if dirtyTabs.includes(activeTab)}
          <span class="toolbar-dirty" title="Unsaved changes">●</span>
        {/if}
      {/if}
    </div>

    <div class="toolbar-right">
      {#if currentStatus !== 'idle'}
        <span
          class="status-label"
          class:error={currentStatus === 'error'}
          class:running={isRunning}
        >
          {statusLabel[currentStatus]}
        </span>
      {/if}

      {#if isRunning}
        <button class="btn btn-stop" onclick={stop}>
          <svg width="10" height="10" viewBox="0 0 10 10"><rect width="10" height="10" rx="2" fill="currentColor"/></svg>
          Stop
        </button>
      {:else}
        <button class="btn btn-run" onclick={run} disabled={!activeTab}>
          <svg width="10" height="12" viewBox="0 0 10 12"><polygon points="0,0 10,6 0,12" fill="currentColor"/></svg>
          Run
        </button>
      {/if}
    </div>
  </header>

  <!-- ── Main layout ───────────────────────────────────────────────────────────── -->
  <div class="main">
    <!-- Sidebar -->
    <Sidebar
      {playgrounds}
      selected={activeTab}
      {dirtyTabs}
      on:select={(e) => openTab(e.detail)}
      on:new={requestNewPlayground}
      on:rename={onRename}
      on:delete={onDelete}
      on:duplicate={onDuplicate}
    />

    <!-- Editor area = TabBar + Editor stacked -->
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
          <Editor code={currentCode} on:change={(e) => onCodeChange(e.detail)} />
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

    <!-- Output panel — shows the active tab's console history -->
    <Output
      output={currentOutput}
      status={currentStatus}
      on:clear={() => { if (activeTab) tabOutput = { ...tabOutput, [activeTab]: [] } }}
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
    position: relative;   /* needed for absolute-positioned center column */
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
    gap: 5px;
    flex-shrink: 0;
    /* Center-align the file name */
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

  .toolbar-filename {
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
  }

  .toolbar-dirty {
    color: var(--text-tertiary);
    font-size: 8px;
  }

  .status-label {
    font-size: 11px;
    color: var(--text-tertiary);
    letter-spacing: 0.02em;
  }
  .status-label.running { color: var(--green); }
  .status-label.error   { color: var(--red); }

  /* ── Run / Stop buttons ── */
  .btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 14px;
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--radius-sm);
    transition: background 0.12s;
  }

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

  /* ── Main 3-panel layout ── */
  .main {
    display: flex;
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  /* ── Editor area = TabBar + editor stacked ── */
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
