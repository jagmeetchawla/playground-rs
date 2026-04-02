<script lang="ts">
  import { onMount, tick } from 'svelte'
  import { invoke, Channel } from '@tauri-apps/api/core'
  import { listen } from '@tauri-apps/api/event'
  import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
  import Sidebar from './lib/Sidebar.svelte'
  import TabBar from './lib/TabBar.svelte'
  import Editor from './lib/Editor.svelte'
  import Output from './lib/Output.svelte'
  import ProjectSwitcher from './lib/ProjectSwitcher.svelte'
  import HelpModal from './lib/HelpModal.svelte'
  import AboutModal from './lib/AboutModal.svelte'
  import SettingsModal from './lib/SettingsModal.svelte'
  import NewPlaygroundModal from './lib/NewPlaygroundModal.svelte'
  import ToolchainWizard from './lib/ToolchainWizard.svelte'
  import type { Settings } from './lib/SettingsModal.svelte'
  import type { Template } from './lib/templates'
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

  // ── Modal state ───────────────────────────────────────────────────────────────
  let showHelp:        boolean       = $state(false)
  let showAbout:       boolean       = $state(false)
  let showSettings:    boolean       = $state(false)
  let showWizard:      boolean       = $state(false)

  // ── Settings ──────────────────────────────────────────────────────────────────
  let settings: Settings = $state({
    font_size: 13,
    font_family: 'Menlo',
    tab_size: 4,
    cargo_path: '',
  })
  let deletePending:   string | null = $state(null)
  let showAddDep:      boolean       = $state(false)
  let depName:         string        = $state('')
  let depVersion:      string        = $state('')
  let depError:        string | null = $state(null)
  let toastMessage:    string | null = $state(null)
  let _toastTimer:     ReturnType<typeof setTimeout> | null = null

  function showToast(msg: string, durationMs = 4000) {
    toastMessage = msg
    if (_toastTimer) clearTimeout(_toastTimer)
    _toastTimer = setTimeout(() => { toastMessage = null }, durationMs)
  }

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

  function stopDrag() {
    dragging = null
    saveWindowState()
  }

  // ── Window state persistence ──────────────────────────────────────────────────

  let _restoring = true   // suppresses saves during the onMount restore phase

  async function saveWindowState() {
    if (_restoring) return
    try {
      await invoke('save_window_state', {
        state: {
          sidebar_visible: sidebarVisible,
          layout:          layoutMode,
          sidebar_w:       Math.round(sidebarW),
          output_h:        Math.round(outputH),
          output_w:        Math.round(outputW),
          open_tabs:       openTabs.map(id => {
            const meta = tabMeta[id]
            if (meta?.type === 'content') return { id, tab_type: 'content', filename: meta.filename }
            if (id === CARGO_TAB)         return { id, tab_type: 'cargo',   filename: null }
            return                               { id, tab_type: 'playground', filename: null }
          }),
          active_tab:    activeTab,
          window_width:  Math.round(window.innerWidth),
          window_height: Math.round(window.innerHeight),
        }
      })
    } catch(e) {
      console.error('saveWindowState failed:', e)
    }
  }

  let _resizeTimer: ReturnType<typeof setTimeout> | null = null
  function onWindowResize() {
    if (_resizeTimer) clearTimeout(_resizeTimer)
    _resizeTimer = setTimeout(() => saveWindowState(), 1000)
  }

  // ── Lifecycle ────────────────────────────────────────────────────────────────
  onMount(async () => {
    let unlisteners: (() => void)[] = []
    try {
      ;[activeProject, projects] = await Promise.all([
        invoke<string>('get_active_project'),
        invoke<string[]>('list_projects'),
      ])
      await loadProjectData()
      toolchainInfo = await invoke<{ path: string; version: string }>('get_toolchain_info')
      settings = await invoke<Settings>('get_settings')

      // Show wizard on first launch or if toolchain is missing
      const tc = await invoke<{ wizard_completed: boolean; all_good: boolean }>('check_toolchain')
      if (!tc.wizard_completed || !tc.all_good) {
        showWizard = true
      }

      // ── Restore window state ────────────────────────────────────────────────
      const ws = await invoke<any>('get_window_state')

      sidebarVisible = ws.sidebar_visible ?? true
      layoutMode     = ws.layout          ?? 'bottom'
      sidebarW       = ws.sidebar_w       ?? 220
      outputH        = ws.output_h        ?? 240
      outputW        = ws.output_w        ?? 300

      if (ws.window_width && ws.window_height) {
        // saved as logical content-area pixels — restore as logical size
        await getCurrentWindow().setSize(new LogicalSize(ws.window_width, ws.window_height))
          .catch(e => console.warn('setSize failed:', e))
      }

      if (Array.isArray(ws.open_tabs)) {
        for (const t of ws.open_tabs) {
          try {
            if (t.tab_type === 'playground' && playgrounds.includes(t.id)) {
              await openTab(t.id, { type: 'playground' })
            } else if (t.tab_type === 'cargo') {
              await openTab(CARGO_TAB, { type: 'cargo' })
            } else if (t.tab_type === 'content' && t.filename) {
              await openTab(t.id, { type: 'content', filename: t.filename })
            }
          } catch { /* file was deleted — skip silently */ }
        }
        if (ws.active_tab && openTabs.includes(ws.active_tab)) {
          activeTab = ws.active_tab
        }
      }

      unlisteners = await Promise.all([
        listen('menu:save',      () => save()),
        listen('menu:run',       () => run()),
        listen('menu:stop',      () => stop()),
        listen('menu:new',       () => requestNewPlayground()),
        listen('menu:close-tab', () => closeTab(activeTab)),
        listen('menu:new-project',    () => { switcherPendingMode = 'new' }),
        listen('menu:rename-project', () => { switcherPendingMode = 'rename' }),
        listen('menu:delete-project', () => { switcherPendingMode = 'delete-confirm' }),
        listen<string>('menu:switch-project', (e) => switchProject(e.payload)),
        listen('menu:settings',          () => { showSettings = true }),
        listen('menu:help',              () => { showHelp  = true }),
        listen('menu:about',             () => { showAbout = true }),
        listen('menu:rust-book',         () => seedRustBook()),
        listen('menu:delete-playground', () => { if (activeTab && tabMeta[activeTab]?.type === 'playground') deletePending = activeTab }),
      ])
    } catch (e) {
      console.error('onMount error:', e)
    } finally {
      // Always show the window — even if something above failed
      await tick()
      await getCurrentWindow().show()
      _restoring = false   // open saves from this point on
    }

    window.addEventListener('keydown', handleKey)
    window.addEventListener('resize', onWindowResize)
    return () => {
      window.removeEventListener('keydown', handleKey)
      window.removeEventListener('resize', onWindowResize)
      unlisteners.forEach(u => u())
    }
  })

  // Save layout prefs whenever sidebar or layout mode changes
  $effect(() => {
    void sidebarVisible; void layoutMode
    saveWindowState()
  })

  // Rebuild the native menu whenever projects or playground count changes
  $effect(() => {
    invoke('rebuild_menu', {
      projects,
      active: activeProject,
      playgroundCount: playgrounds.length,
    }).catch(console.error)
  })

  // ── Helpers ──────────────────────────────────────────────────────────────────

  async function loadProjectData() {
    playgrounds = await invoke<string[]>('list_playgrounds')
    cargoToml   = await invoke<string>('get_cargo_toml').catch(() => '')
  }

  function syncMenuProjects() {
    invoke('rebuild_menu', { projects, active: activeProject, playgroundCount: playgrounds.length }).catch(console.error)
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

  async function seedRustBook() {
    try {
      const created = await invoke<string[]>('seed_rust_book')
      projects = await invoke<string[]>('list_projects')
      if (created.length > 0) {
        // Switch to the first chapter so the user lands somewhere useful
        await switchProject(created[0])
        showToast(`✅ Loaded ${created.length} Rust Book chapter${created.length > 1 ? 's' : ''}. Starting with ${created[0]}.`)
      } else {
        showToast('ℹ️ Rust Book chapters are already loaded.')
      }
    } catch (e) {
      console.error('seedRustBook failed:', e)
      showToast('❌ Failed to load Rust Book examples.')
    }
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
    saveWindowState()
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
    saveWindowState()
  }

  // ── File operations ──────────────────────────────────────────────────────────

  async function save() {
    if (!activeTab) return
    const meta = tabMeta[activeTab] ?? { type: 'playground' }

    if (meta.type === 'content') {
      await invoke('save_content_file', { filename: meta.filename, content: tabCode[activeTab] })
    } else if (meta.type === 'cargo') {
      try {
        await invoke('save_cargo_toml', { content: tabCode[activeTab] })
        cargoToml = tabCode[activeTab]
      } catch (err) {
        showToast(String(err), 6000)
        return
      }
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
        const ts = new Date().toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit', fractionalSecondDigits: 3 } as any)
        updateLastRun(name, r => ({
          ...r, programStarted: true, status: 'running',
          programLines: [...r.programLines, { stream: 'stdout', line: msg.line, ts }],
        }))
      } else if (msg.stream === 'stderr') {
        const ts = new Date().toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit', fractionalSecondDigits: 3 } as any)
        updateLastRun(name, r => ({
          ...r,
          ...(r.programStarted
            ? { programLines: [...r.programLines, { stream: 'stderr', line: msg.line, ts }] }
            : { compilerLines: [...r.compilerLines, { stream: 'stderr', line: msg.line, ts }] }),
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

  async function onNewPlayground(name: string, template: Template) {
    try {
      await invoke('new_playground', { name, content: template.code })

      // Auto-add dependencies if the template requires them
      if (template.deps?.length) {
        let cargoContent = await invoke<string>('get_cargo_toml')
        for (const dep of template.deps) {
          try {
            cargoContent = await invoke<string>('add_dependency', {
              content: cargoContent,
              name: dep.name,
              version: dep.version,
            })
          } catch {
            // Dep may already exist — that's fine
          }
        }
        // Update the editor if Cargo.toml tab is open
        tabCode = { ...tabCode, [CARGO_TAB]: cargoContent }
        cargoToml = cargoContent
      }

      playgrounds = await invoke<string[]>('list_playgrounds')
      creatingNew = false
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

  function onDelete(e: CustomEvent<string>) {
    deletePending = e.detail
  }

  async function confirmDelete() {
    const name = deletePending!
    deletePending = null
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

  async function onStdin(e: CustomEvent<string>) {
    if (!activeTab) return
    const line = e.detail
    // Echo the input in the console as a stdin line
    const ts = new Date().toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit', fractionalSecondDigits: 3 } as any)
    updateLastRun(activeTab, r => ({
      ...r,
      programLines: [...r.programLines, { stream: 'stdin' as const, line, ts }],
    }))
    // Send to the running process
    try {
      await invoke('send_stdin', { line })
    } catch (err) {
      console.error('send_stdin failed:', err)
    }
  }

  // ── Derived ──────────────────────────────────────────────────────────────────
  let runDisabled = $derived(
    !activeTab || isRunning || (tabMeta[activeTab ?? '']?.type ?? 'playground') !== 'playground'
  )
  let isCargoTab = $derived(activeTab === CARGO_TAB)

  // ── Dependency management ───────────────────────────────────────────────────

  async function addDep() {
    const name = depName.trim()
    const version = depVersion.trim() || '*'
    if (!name) { depError = 'Crate name is required'; return }
    depError = null
    try {
      const content = tabCode[CARGO_TAB] ?? cargoToml
      const updated = await invoke<string>('add_dependency', { content, name, version })
      tabCode = { ...tabCode, [CARGO_TAB]: updated }
      cargoToml = updated
      dirtyTabs = dirtyTabs.filter(n => n !== CARGO_TAB)
      depName = ''; depVersion = ''; showAddDep = false
    } catch (err) {
      depError = String(err)
    }
  }

  async function removeDep(name: string) {
    try {
      const content = tabCode[CARGO_TAB] ?? cargoToml
      const updated = await invoke<string>('remove_dependency', { content, name })
      tabCode = { ...tabCode, [CARGO_TAB]: updated }
      cargoToml = updated
      dirtyTabs = dirtyTabs.filter(n => n !== CARGO_TAB)
    } catch (err) {
      console.error('remove_dependency failed:', err)
    }
  }
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
      <button
        class="settings-btn"
        title="Settings (⌘,)"
        onclick={() => showSettings = true}
      >&#9881;&#65038;</button>
      <button class="toolchain-pill" title="{toolchainInfo.path} — Click for toolchain details" onclick={() => showWizard = true}>
        <!-- Isometric crate — crates.io style -->
        <svg width="14" height="12" viewBox="0 0 14 12" fill="none">
          <path d="M7 1L13 4L7 7L1 4Z"
                stroke="currentColor" stroke-width="1.15" stroke-linejoin="round"/>
          <path d="M1 4L1 8.5L7 11.5L7 7Z"
                stroke="currentColor" stroke-width="1.15" stroke-linejoin="round"/>
          <path d="M13 4L13 8.5L7 11.5L7 7Z"
                stroke="currentColor" stroke-width="1.15" stroke-linejoin="round"/>
        </svg>
        cargo {toolchainLabel}
      </button>
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
          {cargoToml}
          onNewPlayground={requestNewPlayground}
          on:select={(e) => openTab(e.detail, { type: 'playground' })}
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
        {#if isCargoTab}
          <div class="dep-toolbar">
            {#if showAddDep}
              <div class="dep-form">
                <input
                  class="dep-input"
                  bind:value={depName}
                  placeholder="crate name"
                  spellcheck="false"
                  onkeydown={(e) => { if (e.key === 'Enter') addDep(); if (e.key === 'Escape') { showAddDep = false; depError = null } }}
                />
                <input
                  class="dep-input dep-version"
                  bind:value={depVersion}
                  placeholder="version (e.g. 1.0)"
                  spellcheck="false"
                  onkeydown={(e) => { if (e.key === 'Enter') addDep(); if (e.key === 'Escape') { showAddDep = false; depError = null } }}
                />
                <button class="dep-btn dep-btn-add" onclick={addDep}>Add</button>
                <button class="dep-btn dep-btn-cancel" onclick={() => { showAddDep = false; depError = null }}>Cancel</button>
              </div>
              {#if depError}
                <div class="dep-error">{depError}</div>
              {/if}
            {:else}
              <button class="dep-btn dep-btn-add" onclick={() => { showAddDep = true; depName = ''; depVersion = ''; depError = null }}>+ Add Dependency</button>
            {/if}
          </div>
        {/if}

        <div class="editor-wrap">
          {#if activeTab}
            <Editor
              code={currentCode}
              language={editorLanguage}
              fontSize={settings.font_size}
              fontFamily={settings.font_family}
              tabSize={settings.tab_size}
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
          on:stdin={onStdin}
        />
      </div>
    </div>

  </div>
</div>

{#if showHelp}
  <HelpModal onclose={() => showHelp = false} />
{/if}

{#if showAbout}
  <AboutModal onclose={() => showAbout = false} />
{/if}

{#if showSettings}
  <SettingsModal
    bind:settings
    onclose={() => showSettings = false}
    onsave={(s) => { settings = s }}
  />
{/if}

{#if showWizard}
  <ToolchainWizard onclose={() => { showWizard = false }} />
{/if}

{#if creatingNew}
  <NewPlaygroundModal
    existingNames={playgrounds}
    onclose={() => creatingNew = false}
    oncreate={onNewPlayground}
  />
{/if}

{#if toastMessage}
  <div class="toast" role="status" aria-live="polite">{toastMessage}</div>
{/if}

{#if deletePending}
  <div class="confirm-backdrop" onclick={() => deletePending = null} aria-hidden="true"></div>
  <div class="confirm-dialog" role="alertdialog" aria-modal="true">
    <p class="confirm-msg">Delete <strong>{deletePending}</strong>?<br><span class="confirm-sub">This cannot be undone.</span></p>
    <div class="confirm-actions">
      <button class="confirm-cancel" onclick={() => deletePending = null}>Cancel</button>
      <button class="confirm-delete" onclick={confirmDelete}>Delete</button>
    </div>
  </div>
{/if}

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
    height: 28px;
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


  .status-label { font-size: 11px; color: var(--text-tertiary); letter-spacing: 0.02em; }
  .status-label.running { color: var(--green); }
  .status-label.error   { color: var(--red); }

  .btn {
    display: flex; align-items: center; gap: 5px;
    height: 28px; box-sizing: border-box;
    padding: 0 12px; font-size: 12px; font-weight: 600;
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

  .settings-btn {
    display: flex; align-items: center; justify-content: center;
    width: 28px; height: 28px;
    font-size: 21px; line-height: 1;
    color: var(--text-tertiary);
    background: rgba(255,255,255,0.07); border: 1px solid rgba(255,255,255,0.11);
    border-radius: 8px;
    cursor: pointer; transition: color 0.12s, background 0.12s, border-color 0.12s;
  }
  .settings-btn:hover {
    color: var(--text); background: rgba(255,255,255,0.13);
    border-color: rgba(255,255,255,0.18);
  }

  .toolchain-pill {
    display: flex; align-items: center; gap: 5px;
    height: 28px; box-sizing: border-box;
    font-size: 11px; font-family: var(--font-mono);
    color: var(--text-tertiary);
    background: rgba(255,255,255,0.07); border: 1px solid rgba(255,255,255,0.11);
    border-radius: 8px; padding: 0 8px;
    cursor: pointer; white-space: nowrap;
    transition: background 0.12s, border-color 0.12s, color 0.12s;
  }

  .toolchain-pill:hover {
    background: rgba(255,255,255,0.13);
    border-color: rgba(255,255,255,0.18);
    color: var(--text);
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

  /* ── Dependency toolbar ── */
  .dep-toolbar {
    display: flex; flex-wrap: wrap; align-items: center; gap: 6px;
    padding: 6px 10px;
    background: var(--bg-sidebar);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .dep-form {
    display: flex; align-items: center; gap: 6px; flex-wrap: wrap;
  }
  .dep-input {
    font-family: var(--font-mono); font-size: 12px;
    background: rgba(0,0,0,0.25); color: var(--text);
    border: 1px solid var(--border); border-radius: var(--radius-xs);
    padding: 4px 8px; outline: none;
  }
  .dep-input:focus { border-color: var(--accent); }
  .dep-input::placeholder { color: var(--text-tertiary); opacity: 0.5; }
  .dep-version { width: 120px; }
  .dep-btn {
    font-size: 11px; font-weight: 600;
    padding: 4px 10px; border-radius: var(--radius-xs);
    cursor: pointer; transition: background 0.1s, border-color 0.1s;
  }
  .dep-btn-add {
    color: #fff; background: var(--accent); border: 1px solid var(--accent);
  }
  .dep-btn-add:hover { filter: brightness(1.15); }
  .dep-btn-cancel {
    color: var(--text-secondary);
    background: rgba(255,255,255,0.06); border: 1px solid var(--border);
  }
  .dep-btn-cancel:hover { background: rgba(255,255,255,0.1); border-color: var(--border-strong); }
  .dep-error {
    font-size: 11px; color: var(--red); padding: 2px 0;
    width: 100%;
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

  /* ── Toast notification ── */
  .toast {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 400;
    background: rgba(30, 32, 40, 0.94);
    color: #e8e8e8;
    border: 1px solid rgba(255,255,255,0.12);
    border-radius: 8px;
    padding: 10px 18px;
    font-size: 13px;
    line-height: 1.4;
    backdrop-filter: blur(8px);
    box-shadow: 0 4px 20px rgba(0,0,0,0.45);
    pointer-events: none;
    white-space: nowrap;
    animation: toast-in 0.18s ease;
  }
  @keyframes toast-in {
    from { opacity: 0; transform: translateX(-50%) translateY(8px); }
    to   { opacity: 1; transform: translateX(-50%) translateY(0);   }
  }

  /* ── Delete confirm dialog ── */
  .confirm-backdrop {
    position: fixed; inset: 0; z-index: 299;
    background: rgba(0,0,0,0.45); backdrop-filter: blur(2px);
  }
  .confirm-dialog {
    position: fixed;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    z-index: 300;
    width: 300px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    box-shadow: 0 16px 48px rgba(0,0,0,0.6);
    padding: 20px 20px 16px;
    display: flex; flex-direction: column; gap: 16px;
  }
  .confirm-msg {
    font-size: 13px; color: var(--text); line-height: 1.55; margin: 0;
  }
  .confirm-msg strong { color: var(--text); }
  .confirm-sub { font-size: 12px; color: var(--text-tertiary); }
  .confirm-actions {
    display: flex; justify-content: flex-end; gap: 8px;
  }
  .confirm-cancel {
    font-size: 12px; padding: 5px 12px; border-radius: 6px;
    background: rgba(255,255,255,0.07); border: 1px solid var(--border);
    color: var(--text-secondary);
  }
  .confirm-cancel:hover { background: rgba(255,255,255,0.11); }
  .confirm-delete {
    font-size: 12px; padding: 5px 12px; border-radius: 6px;
    background: rgba(220, 60, 60, 0.25); border: 1px solid rgba(220,60,60,0.4);
    color: #ff7070;
  }
  .confirm-delete:hover { background: rgba(220, 60, 60, 0.38); }
</style>
