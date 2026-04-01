<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke, Channel } from '@tauri-apps/api/core'
  import { listen } from '@tauri-apps/api/event'
  import Sidebar from './lib/Sidebar.svelte'
  import TabBar from './lib/TabBar.svelte'
  import Editor from './lib/Editor.svelte'
  import Output from './lib/Output.svelte'
  import ProjectSwitcher from './lib/ProjectSwitcher.svelte'
  import type { RunBlock } from './lib/Output.svelte'

  // ── Constants ────────────────────────────────────────────────────────────────
  const CARGO_TAB = 'Cargo.toml'

  // ── Tab metadata ─────────────────────────────────────────────────────────────
  type TabMeta =
    | { type: 'playground' }
    | { type: 'cargo' }
    | { type: 'content'; filename: string }

  // ── Projects ─────────────────────────────────────────────────────────────────
  let projects:            string[]                                    = $state([])
  let activeProject:       string                                      = $state('')
  let switcherPendingMode: 'new' | 'rename' | 'delete-confirm' | null = $state(null)

  // ── Playground list ──────────────────────────────────────────────────────────
  let playgrounds: string[] = $state([])

  // ── Tab state ────────────────────────────────────────────────────────────────
  let openTabs:  string[]                = $state([])
  let activeTab: string | null           = $state(null)
  let tabCode:   Record<string, string>  = $state({})
  let dirtyTabs: string[]                = $state([])
  let tabMeta:   Record<string, TabMeta> = $state({})

  let currentCode    = $derived(activeTab ? (tabCode[activeTab] ?? '') : '')
  let currentTabMeta = $derived(activeTab ? (tabMeta[activeTab] ?? { type: 'playground' } as TabMeta) : { type: 'playground' } as TabMeta)
  let editorLanguage = $derived(languageForTab(activeTab, currentTabMeta))

  let tabLabels = $derived(
    Object.fromEntries(openTabs.map(id => {
      const meta = tabMeta[id]
      if (meta?.type === 'content') return [id, meta.filename]
      if (id === CARGO_TAB) return [id, 'Cargo.toml']
      return [id, id]
    }))
  )
  let tabTypes = $derived(
    Object.fromEntries(openTabs.map(id => {
      const meta = tabMeta[id]
      if (meta?.type === 'content') return [id, 'content' as const]
      if (id === CARGO_TAB) return [id, 'cargo' as const]
      return [id, 'rs' as const]
    }))
  )

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

  // ── Toolchain + Cargo.toml ────────────────────────────────────────────────────
  let cargoToml:     string                             = $state('')
  let toolchainInfo: { path: string; version: string } = $state({ path: '', version: '' })
  let toolchainLabel = $derived(
    toolchainInfo.version
      ? (toolchainInfo.version.match(/\d+\.\d+\.\d+/)?.[0] ?? toolchainInfo.version)
      : '…'
  )

  // ── New playground binding ────────────────────────────────────────────────────
  let creatingNew: boolean = $state(false)

  // ── Layout & panel sizing ─────────────────────────────────────────────────────
  let sidebarVisible                   = $state(true)
  let layoutMode: 'bottom' | 'right'   = $state('bottom')
  let sidebarW                         = $state(220)
  let outputH                          = $state(240)   // bottom layout
  let outputW                          = $state(300)   // right layout

  // Drag state — pointer-captured resize handles
  let dragging: 'sidebar' | 'output' | null = null
  let dragStartX = 0, dragStartY = 0, dragStartVal = 0

  function startSidebarResize(e: PointerEvent) {
    dragging = 'sidebar'
    dragStartX = e.clientX
    dragStartVal = sidebarW
    ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  }

  function startOutputResize(e: PointerEvent) {
    dragging = 'output'
    dragStartX = e.clientX
    dragStartY = e.clientY
    dragStartVal = layoutMode === 'bottom' ? outputH : outputW
    ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  }

  function onDragMove(e: PointerEvent) {
    if (dragging === 'sidebar') {
      sidebarW = Math.max(160, Math.min(380, dragStartVal + (e.clientX - dragStartX)))
    } else if (dragging === 'output') {
      if (layoutMode === 'bottom') {
        // Dragging upward increases output height
        outputH = Math.max(80, Math.min(600, dragStartVal - (e.clientY - dragStartY)))
      } else {
        // Dragging leftward increases output width
        outputW = Math.max(200, Math.min(600, dragStartVal - (e.clientX - dragStartX)))
      }
    }
  }

  function stopDrag() { dragging = null }

  // ── Lifecycle ────────────────────────────────────────────────────────────────
  onMount(async () => {
    ;[activeProject, projects] = await Promise.all([
      invoke<string>('get_active_project'),
      invoke<string[]>('list_projects'),
    ])
    await loadProjectData()
    toolchainInfo = await invoke<{ path: string; version: string }>('get_toolchain_info')

    const unlisteners = await Promise.all([
      listen('menu:save',      () => save()),
      listen('menu:run',       () => run()),
      listen('menu:stop',      () => stop()),
      listen('menu:new',       () => requestNewPlayground()),
      listen('menu:close-tab', () => closeTab(activeTab)),
      listen('menu:new-project',    () => { switcherPendingMode = 'new' }),
      listen('menu:rename-project', () => { switcherPendingMode = 'rename' }),
      listen('menu:delete-project', () => { switcherPendingMode = 'delete-confirm' }),
      listen<string>('menu:switch-project', (e) => switchProject(e.payload)),
    ])

    window.addEventListener('keydown', handleKey)
    return () => {
      window.removeEventListener('keydown', handleKey)
      unlisteners.forEach(u => u())
    }
  })

  // ── Helpers ──────────────────────────────────────────────────────────────────

  async function loadProjectData() {
    playgrounds = await invoke<string[]>('list_playgrounds')
    cargoToml   = await invoke<string>('get_cargo_toml').catch(() => '')
  }

  function syncMenuProjects() {
    invoke('rebuild_projects_menu', { projects, active: activeProject }).catch(console.error)
  }

  async function switchProject(name: string) {
    openTabs  = []
    activeTab = null
    tabCode   = {}
    tabMeta   = {}
    tabRuns   = {}
    tabRunCount = {}
    dirtyTabs = []
    await invoke('switch_project', { name })
    activeProject = name
    await loadProjectData()
    syncMenuProjects()
  }

  function contentTabId(filename: string) { return `content:${filename}` }

  function languageForTab(id: string | null, meta: TabMeta): string {
    if (!id) return 'rust'
    if (id === CARGO_TAB) return 'ini'
    if (meta.type === 'content') return languageFromFilename(meta.filename)
    return 'rust'
  }

  function languageFromFilename(filename: string): string {
    const ext = filename.split('.').pop()?.toLowerCase() ?? ''
    const map: Record<string, string> = {
      rs: 'rust', json: 'json', toml: 'ini', md: 'markdown',
      yaml: 'yaml', yml: 'yaml', html: 'html', xml: 'xml',
      js: 'javascript', ts: 'typescript', css: 'css',
      sh: 'shell', bash: 'shell',
    }
    return map[ext] ?? 'plaintext'
  }

  // ── Keyboard shortcuts ───────────────────────────────────────────────────────
  function handleKey(e: KeyboardEvent) {
    if (e.metaKey && e.key === 'n') { e.preventDefault(); requestNewPlayground() }
    if (e.metaKey && e.key === 'r') { e.preventDefault(); run() }
    if (e.metaKey && e.key === 's') { e.preventDefault(); save() }
    if (e.metaKey && e.key === '.') { e.preventDefault(); stop() }
    if (e.metaKey && e.key === 'w') { e.preventDefault(); closeTab(activeTab) }
    if (e.metaKey && e.shiftKey && e.code === 'KeyL') { e.preventDefault(); sidebarVisible = !sidebarVisible }
  }

  // ── Tab management ───────────────────────────────────────────────────────────

  async function openTab(name: string, meta: TabMeta = { type: 'playground' }) {
    if (!openTabs.includes(name)) {
      let code: string
      if (meta.type === 'content') {
        code = await invoke<string>('read_content_file', { filename: meta.filename })
      } else if (name === CARGO_TAB) {
        code = await invoke<string>('get_cargo_toml')
      } else {
        code = await invoke<string>('load_playground', { name })
      }
      tabCode  = { ...tabCode,  [name]: code }
      tabMeta  = { ...tabMeta,  [name]: meta }
      openTabs = [...openTabs, name]
    }
    activeTab = name
  }

  function closeTab(name: string | null) {
    if (!name) return
    dirtyTabs = dirtyTabs.filter(n => n !== name)
    const idx = openTabs.indexOf(name)
    openTabs  = openTabs.filter(n => n !== name)

    const { [name]: _c, ...restCode  } = tabCode
    const { [name]: _m, ...restMeta  } = tabMeta
    const { [name]: _r, ...restRuns  } = tabRuns
    const { [name]: _n, ...restCount } = tabRunCount
    tabCode     = restCode
    tabMeta     = restMeta
    tabRuns     = restRuns
    tabRunCount = restCount

    if (activeTab === name) {
      activeTab = openTabs[idx] ?? openTabs[idx - 1] ?? null
    }
  }

  // ── File operations ──────────────────────────────────────────────────────────

  async function save() {
    if (!activeTab) return
    const meta = tabMeta[activeTab] ?? { type: 'playground' }

    if (meta.type === 'content') {
      await invoke('save_content_file', { filename: meta.filename, content: tabCode[activeTab] })
    } else if (meta.type === 'cargo') {
      await invoke('save_cargo_toml', { content: tabCode[activeTab] })
      cargoToml = tabCode[activeTab]
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
    if (!activeTab || isRunning) return
    const meta = tabMeta[activeTab] ?? { type: 'playground' }
    if (meta.type !== 'playground') return
    const name = activeTab
    await save()

    const existing = tabRuns[name] ?? []
    const collapsed = existing.map(r => ({ ...r, collapsed: true }))

    const runNum = (tabRunCount[name] ?? 0) + 1
    tabRunCount = { ...tabRunCount, [name]: runNum }

    const now = new Date()
    const startedAt = now.toLocaleTimeString('en-US', {
      hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false
    })

    const newBlock: RunBlock = {
      runNum, command: `cargo run --bin ${name}`, startedAt,
      status: 'compiling', exitCode: null,
      compilerLines: [], programLines: [],
      collapsed: false, programStarted: false,
    }
    tabRuns = { ...tabRuns, [name]: [...collapsed, newBlock] }

    const channel = new Channel()
    channel.onmessage = (msg: any) => {
      if (msg.stream === 'complete') {
        updateLastRun(name, r => ({
          ...r, status: msg.code === 0 ? 'success' : 'error', exitCode: msg.code,
        }))
      } else if (msg.stream === 'stdout') {
        updateLastRun(name, r => ({
          ...r, programStarted: true, status: 'running',
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
        ...r, status: 'error',
        compilerLines: [...r.compilerLines, { stream: 'stderr', line: String(e) }],
      }))
    }
  }

  // Stop: actually kill the running process via the backend.
  // The channel's "complete" message handles the RunBlock status update.
  function stop() {
    invoke('kill_playground').catch(console.error)
  }

  // ── Playground CRUD ──────────────────────────────────────────────────────────

  function requestNewPlayground() { creatingNew = true }

  async function onNewPlayground(e: CustomEvent<string>) {
    const name = e.detail
    try {
      await invoke('new_playground', { name })
      playgrounds = await invoke<string[]>('list_playgrounds')
      await openTab(name, { type: 'playground' })
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
      const { [oldName]: meta,  ...restMeta  } = tabMeta
      const { [oldName]: runs,  ...restRuns  } = tabRuns
      const { [oldName]: count, ...restCount } = tabRunCount

      tabCode     = { ...restCode,  [newName]: code }
      tabMeta     = { ...restMeta,  [newName]: meta ?? { type: 'playground' } }
      tabRuns     = { ...restRuns,  [newName]: runs  ?? [] }
      tabRunCount = { ...restCount, [newName]: count ?? 0  }
      openTabs    = openTabs.map(n => n === oldName ? newName : n)
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
    await openTab(newName, { type: 'playground' })
  }

  async function onEditCargo() {
    cargoToml = await invoke<string>('get_cargo_toml').catch(() => cargoToml)
    await openTab(CARGO_TAB, { type: 'cargo' })
  }

  // ── Project management ───────────────────────────────────────────────────────

  async function onNewProject(name: string) {
    await invoke('new_project', { name })
    projects = await invoke<string[]>('list_projects')
    await switchProject(name)
  }

  async function onRenameProject(oldName: string, newName: string) {
    await invoke('rename_project', { oldName, newName })
    projects = await invoke<string[]>('list_projects')
    activeProject = newName
    syncMenuProjects()
  }

  async function onDeleteProject(name: string) {
    const remaining = projects.filter(p => p !== name)
    let switchTo = remaining[0]
    if (!switchTo) {
      await invoke('new_project', { name: 'default' })
      switchTo = 'default'
    }
    await switchProject(switchTo)
    await invoke('delete_project', { name })
    projects = await invoke<string[]>('list_projects')
    syncMenuProjects()
  }

  // ── Content file tab opening ─────────────────────────────────────────────────

  async function onOpenContentFile(e: CustomEvent<{ filename: string }>) {
    const { filename } = e.detail
    await openTab(contentTabId(filename), { type: 'content', filename })
  }

  // ── Console events ───────────────────────────────────────────────────────────

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

  // ── Derived ──────────────────────────────────────────────────────────────────
  let runDisabled = $derived(
    !activeTab || isRunning || (tabMeta[activeTab ?? '']?.type ?? 'playground') !== 'playground'
  )
</script>

<div class="app">
  <!-- ── Toolbar ────────────────────────────────────────────────────────────── -->
  <header class="toolbar">
    <div class="toolbar-left">
      <!-- Hide / show sidebar -->
      <button
        class="toolbar-pill"
        class:pill-active={sidebarVisible}
        onclick={() => sidebarVisible = !sidebarVisible}
        title="{sidebarVisible ? 'Hide' : 'Show'} Sidebar (⌘⇧L)"
        aria-label="{sidebarVisible ? 'Hide' : 'Show'} sidebar"
      >
        <svg width="16" height="13" viewBox="0 0 16 13" fill="none">
          <rect x="0.5" y="0.5" width="4.5" height="12" rx="1.5"
                fill="currentColor" opacity={sidebarVisible ? 1 : 0.35}/>
          <rect x="6.5" y="0.5" width="9" height="12" rx="1.5"
                fill="currentColor" opacity="0.35"/>
        </svg>
      </button>

      <span class="app-badge">RS</span>

      <ProjectSwitcher
        {projects}
        active={activeProject}
        onswitch={switchProject}
        onnew={onNewProject}
        onrename={onRenameProject}
        ondelete={onDeleteProject}
        bind:pendingMode={switcherPendingMode}
      />
    </div>

    <div class="toolbar-center">
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
        <button class="btn btn-run" onclick={run} disabled={runDisabled}>
          <svg width="10" height="12" viewBox="0 0 10 12"><polygon points="0,0 10,6 0,12" fill="currentColor"/></svg>
          Run
        </button>
      {/if}

      <!-- Layout switch — icon shows what you'll switch TO -->
      <button
        class="toolbar-pill"
        onclick={() => layoutMode = layoutMode === 'bottom' ? 'right' : 'bottom'}
        title="Switch to {layoutMode === 'bottom' ? 'side-by-side' : 'stacked'} layout"
        aria-label="Switch layout"
      >
        {#if layoutMode === 'bottom'}
          <!-- Currently stacked → icon previews side-by-side -->
          <svg width="16" height="13" viewBox="0 0 16 13" fill="none">
            <rect x="0.5" y="0.5" width="7"   height="12" rx="1.5" fill="currentColor" opacity="0.8"/>
            <rect x="9"   y="0.5" width="6.5" height="12" rx="1.5" fill="currentColor" opacity="0.4"/>
          </svg>
        {:else}
          <!-- Currently side-by-side → icon previews stacked -->
          <svg width="16" height="13" viewBox="0 0 16 13" fill="none">
            <rect x="0.5" y="0.5" width="15" height="6.5" rx="1.5" fill="currentColor" opacity="0.8"/>
            <rect x="0.5" y="8.5" width="15" height="4"   rx="1.5" fill="currentColor" opacity="0.4"/>
          </svg>
        {/if}
      </button>
    </div>
  </header>

  <!-- ── Main layout ──────────────────────────────────────────────────────────── -->
  <div class="main">

    <!-- Sidebar + vertical resize handle -->
    {#if sidebarVisible}
      <div class="sidebar-wrap" style="width:{sidebarW}px">
        <Sidebar
          {playgrounds}
          selected={activeTab && tabMeta[activeTab]?.type === 'playground' ? activeTab : null}
          {dirtyTabs}
          bind:creatingNew
          {cargoToml}
          on:select={(e) => openTab(e.detail, { type: 'playground' })}
          on:new={onNewPlayground}
          on:rename={onRename}
          on:delete={onDelete}
          on:duplicate={onDuplicate}
          on:editcargo={onEditCargo}
          on:opencontentfile={onOpenContentFile}
        />
      </div>
      <div
        class="drag-handle drag-col"
        onpointerdown={startSidebarResize}
        onpointermove={onDragMove}
        onpointerup={stopDrag}
        role="separator"
        aria-label="Resize sidebar"
      ></div>
    {/if}

    <!-- Editor + Output (layout switches between row and column) -->
    <div class="center-wrap" class:layout-bottom={layoutMode === 'bottom'}>
      <div class="editor-area">
        <TabBar
          tabs={openTabs}
          active={activeTab}
          {dirtyTabs}
          {tabLabels}
          {tabTypes}
          on:activate={(e) => openTab(e.detail, tabMeta[e.detail] ?? { type: 'playground' })}
          on:close={(e) => closeTab(e.detail)}
        />
        <div class="editor-wrap">
          {#if activeTab}
            <Editor
              code={currentCode}
              language={editorLanguage}
              onSave={save}
              onRun={run}
              onNew={requestNewPlayground}
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

      <!-- Resize handle between editor and output (col in right mode, row in bottom) -->
      <div
        class="drag-handle"
        class:drag-col={layoutMode === 'right'}
        class:drag-row={layoutMode === 'bottom'}
        onpointerdown={startOutputResize}
        onpointermove={onDragMove}
        onpointerup={stopDrag}
        role="separator"
        aria-label="Resize output panel"
      ></div>

      <div
        class="output-wrap"
        style="{layoutMode === 'bottom' ? `height:${outputH}px` : `width:${outputW}px`}"
      >
        <Output
          runs={currentRuns}
          status={currentStatus}
          on:toggle={onToggle}
          on:clear={onClear}
        />
      </div>
    </div>

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
    padding: 0 8px 0 6px;
    background: var(--bg-sidebar);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    position: relative;
    gap: 8px;
  }

  .toolbar-left {
    display: flex; align-items: center; gap: 6px;
    flex: 1; min-width: 0;
  }

  .toolbar-center {
    display: flex; align-items: center; gap: 8px;
    flex-shrink: 0;
    position: absolute; left: 50%; transform: translateX(-50%);
  }

  .toolbar-right {
    display: flex; align-items: center; gap: 8px;
    flex: 1; justify-content: flex-end; min-width: 0;
  }

  /* ── Pill buttons — sidebar toggle + layout switch (Safari-style) ── */
  .toolbar-pill {
    display: inline-flex; align-items: center; justify-content: center;
    height: 26px;
    padding: 0 9px;
    border-radius: 8px;
    background: rgba(255,255,255,0.07);
    border: 1px solid rgba(255,255,255,0.11);
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: background 0.12s, border-color 0.12s, color 0.12s;
  }
  .toolbar-pill:hover {
    background: rgba(255,255,255,0.13);
    border-color: rgba(255,255,255,0.18);
    color: var(--text);
  }
  /* "active" state — sidebar is visible, pill looks pressed-in */
  .toolbar-pill.pill-active {
    background: rgba(255,255,255,0.12);
    border-color: rgba(255,255,255,0.18);
    color: var(--text);
  }

  /* ── App badge ── */
  .app-badge {
    font-size: 8px; font-weight: 800;
    background: var(--rust-orange); color: #fff;
    border-radius: 3px; padding: 2px 4px;
    line-height: 1.3; letter-spacing: 0.03em;
    flex-shrink: 0;
  }

  .status-label { font-size: 11px; color: var(--text-tertiary); letter-spacing: 0.02em; }
  .status-label.running { color: var(--green); }
  .status-label.error   { color: var(--red); }

  .btn {
    display: flex; align-items: center; gap: 5px;
    padding: 5px 12px; font-size: 12px; font-weight: 600;
    border-radius: var(--radius-sm);
    transition: background 0.12s, opacity 0.12s;
    flex-shrink: 0;
  }

  .btn-save {
    background: rgba(255,255,255,0.08); color: var(--text);
    border: 1px solid var(--border-strong);
  }
  .btn-save:hover:not(:disabled) { background: rgba(255,255,255,0.14); }
  .btn-save:disabled { opacity: 0.3; cursor: not-allowed; }

  .btn-run  { background: var(--accent); color: #fff; }
  .btn-run:hover:not(:disabled) { background: var(--accent-hover); }
  .btn-run:disabled { opacity: 0.3; cursor: not-allowed; }

  .btn-stop { background: #3a3a3c; color: var(--text-secondary); }
  .btn-stop:hover { background: var(--bg-elevated); }

  .toolchain-pill {
    display: flex; align-items: center; gap: 5px;
    font-size: 11px; font-family: var(--font-mono);
    color: var(--text-tertiary);
    background: var(--bg-elevated); border: 1px solid var(--border);
    border-radius: var(--radius-sm); padding: 3px 8px;
    cursor: default; white-space: nowrap;
  }

  /* ── Main layout ── */
  .main {
    display: flex;
    flex: 1;
    overflow: hidden;
    flex-direction: row;
  }

  /* ── Sidebar ── */
  .sidebar-wrap {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Drag handles ── */
  .drag-handle {
    flex-shrink: 0;
    background: transparent;
    transition: background 0.15s;
    position: relative;
    z-index: 10;
  }
  .drag-handle:hover,
  .drag-handle:active { background: rgba(255,255,255,0.1); }

  .drag-col { width: 4px; cursor: col-resize; border-right: 1px solid var(--border); }
  .drag-row { height: 4px; cursor: row-resize; border-bottom: 1px solid var(--border); }

  /* ── Center wrap (editor + output) ── */
  .center-wrap {
    flex: 1;
    display: flex;
    flex-direction: row;   /* right layout: side-by-side */
    overflow: hidden;
    min-width: 0;
  }
  .center-wrap.layout-bottom {
    flex-direction: column; /* bottom layout: stacked */
  }

  /* ── Editor area ── */
  .editor-area {
    flex: 1; display: flex; flex-direction: column;
    overflow: hidden; min-width: 0;
  }

  .editor-wrap {
    flex: 1; display: flex; overflow: hidden;
  }

  /* ── Output wrap ── */
  .output-wrap {
    flex-shrink: 0;
    display: flex;
    overflow: hidden;
  }
  /* Border direction depends on layout */
  .center-wrap:not(.layout-bottom) .output-wrap {
    border-left: 1px solid var(--border);
  }
  .center-wrap.layout-bottom .output-wrap {
    border-top: 1px solid var(--border);
  }

  /* ── Empty state ── */
  .empty-state {
    flex: 1; display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    gap: 10px; color: var(--text-tertiary); padding: 40px;
  }

  .empty-icon { margin-bottom: 6px; }

  .empty-title {
    font-size: 15px; font-weight: 600; color: var(--text-secondary);
  }

  .empty-hint {
    font-size: 13px; color: var(--text-tertiary); text-align: center;
  }

  .link-btn {
    background: none; color: var(--accent);
    text-decoration: underline; padding: 0;
    font-size: 13px; display: inline;
  }

  .shortcut-grid {
    display: grid; grid-template-columns: auto auto;
    gap: 4px 16px; margin-top: 16px; align-items: center;
  }

  .shortcut-key {
    font-family: var(--font-mono); font-size: 11px;
    background: var(--bg-elevated); border: 1px solid var(--border-strong);
    border-radius: var(--radius-xs); padding: 2px 7px;
    color: var(--text-secondary); text-align: center; justify-self: end;
  }

  .shortcut-desc { font-size: 12px; color: var(--text-tertiary); }
</style>
