<script lang="ts">
  import { onMount, tick } from 'svelte'
  import { invoke, Channel } from '@tauri-apps/api/core'
  import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
  import { listen } from '@tauri-apps/api/event'
  import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window'
  import Sidebar from './lib/Sidebar.svelte'
  import TabBar from './lib/TabBar.svelte'
  import Editor from './lib/Editor.svelte'
  import Output from './lib/Output.svelte'
  import ProjectSwitcher from './lib/ProjectSwitcher.svelte'
  import HelpModal from './lib/HelpModal.svelte'
  import AboutModal from './lib/AboutModal.svelte'
  // SettingsModal.svelte is now type-only; ToolchainWizard handles settings UI
  import NewPlaygroundModal from './lib/NewPlaygroundModal.svelte'
  import ToolchainWizard from './lib/ToolchainWizard.svelte'
  import CopyToProjectModal from './lib/CopyToProjectModal.svelte'
  import type { Settings } from './lib/SettingsModal.svelte'
  import type { Template } from './lib/templates'
  import type { RunBlock } from './lib/Output.svelte'
  import { getLang, allLanguages, type ProjectType } from './lib/languages'
  import { currentEdition } from './lib/editions'

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
  let projectType:         ProjectType                                 = $state('rust')
  let projectTypes:        Record<string, ProjectType>                 = $state({})
  let projectSources:      Record<string, string>                     = $state({})
  let projectReadonly:     Record<string, boolean>                    = $state({})
  let lockedPlaygrounds:   string[]                                   = $state([])
  let switcherPendingMode: 'new' | 'rename' | 'delete-confirm' | null = $state(null)
  let enabledLangs:        string[]                                  = $state(['rust'])
  let lang                 = $derived(getLang(projectType))
  let isBookProject        = $derived(!!projectSources[activeProject])
  let isPlaygroundLocked   = $derived(activeTab ? lockedPlaygrounds.includes(activeTab) : false)
  let isReadOnly           = $derived(isBookProject || isPlaygroundLocked)

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
      // For non-Rust projects, derive type from extension
      if (lang.needsExtension) {
        const ext = id.split('.').pop()?.toLowerCase() ?? ''
        const extMap: Record<string, string> = { c: 'c', cpp: 'cpp', zig: 'zig', swift: 'swift', rs: 'rs' }
        return [id, (extMap[ext] ?? 'rs') as any]
      }
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

  // Check if any playground (not just the active tab) has a running process
  let runningPlayground = $derived.by(() => {
    for (const [name, runs] of Object.entries(tabRuns)) {
      const last = runs.at(-1)
      if (last && (last.status === 'compiling' || last.status === 'running')) {
        return name
      }
    }
    return null
  })

  // ── Toolchain + Cargo.toml ────────────────────────────────────────────────────
  let cargoToml:     string                             = $state('')
  let toolchainInfo: { path: string; version: string } = $state({ path: '', version: '' })
  let clangInfo:     { path: string; version: string } = $state({ path: '', version: '' })
  let zigInfo:       { path: string; version: string; version_ok: boolean } = $state({ path: '', version: '', version_ok: false })
  let swiftInfo:     { version: string }               = $state({ version: '' })
  let activeToolchain = $derived(
    projectType === 'clang' ? clangInfo
    : projectType === 'zig' ? zigInfo
    : projectType === 'swift' ? swiftInfo as any
    : toolchainInfo
  )
  let toolchainLabel = $derived(
    activeToolchain.version
      ? (activeToolchain.version.match(/\d+\.\d+\.\d+/)?.[0] ?? activeToolchain.version)
      : '…'
  )
  let toolchainName = $derived(lang.toolchainName)
  let langEnabled = $derived(enabledLangs.includes(projectType))
  let pillStatus: 'not-enabled' | 'missing' | 'partial' | 'ok' = $derived.by(() => {
    if (!langEnabled) return 'not-enabled'
    if (!activeToolchain.version) return 'missing'
    // Partial: Rust needs cargo+rustc+rustfmt+clippy; others just need version
    if (projectType === 'rust' && toolchainLabel === '…') return 'missing'
    // Zig: warn if installed but not the supported 0.15.x version
    if (projectType === 'zig' && zigInfo && !zigInfo.version_ok) return 'partial'
    return 'ok'
  })
  let pillText = $derived.by(() => {
    if (pillStatus === 'not-enabled') return `${lang.label} support not enabled`
    if (pillStatus === 'missing') return `${toolchainName} not found`
    if (pillStatus === 'partial' && projectType === 'zig') return `${toolchainName} ${toolchainLabel} · requires 0.15.x`
    return `${toolchainName} ${toolchainLabel}${lang.experimental ? ' · experimental' : ''}`
  })

  // ── New playground binding ────────────────────────────────────────────────────
  let creatingNew: boolean = $state(false)

  // ── Modal state ───────────────────────────────────────────────────────────────
  let showHelp:        boolean       = $state(false)
  let showAbout:       boolean       = $state(false)
  let showWizard:      boolean       = $state(false)
  let wizardMode:      'wizard' | 'settings' = $state('wizard')
  let showExportMenu:  boolean       = $state(false)
  let showCopyToProject: boolean     = $state(false)
  let copyToProjectPlayground: string | null = $state(null)

  // ── Settings ──────────────────────────────────────────────────────────────────
  let settings: Settings = $state({
    font_size: 13,
    font_family: 'Menlo',
    tab_size: 0,
    cargo_path: '',
    theme: 'system',
  })

  // ── Theme resolution ──────────────────────────────────────────────────────
  let systemDark = $state(window.matchMedia('(prefers-color-scheme: dark)').matches)

  // Map project type → language theme
  const autoThemeMap: Record<string, string> = {
    rust: 'rust', clang: 'seagreen', zig: 'zig', swift: 'swift',
  }

  // Resolved theme based on setting + OS preference + project type
  let resolvedTheme = $derived(
    settings.theme === 'system' ? (systemDark ? 'dark' : 'light')
      : settings.theme === 'auto' ? (autoThemeMap[projectType] ?? 'dark')
      : settings.theme
  )

  // Monaco theme name derived from resolved theme
  let monacoTheme = $derived(
    resolvedTheme === 'rust' ? 'playground-rust'
      : resolvedTheme === 'seagreen' ? 'playground-seagreen'
      : resolvedTheme === 'zig' ? 'playground-zig'
      : resolvedTheme === 'swift' ? 'playground-swift'
      : resolvedTheme === 'light' ? 'playground-light'
      : 'playground-dark'
  )
  let deletePending:   string | null = $state(null)
  let renameTarget:    string | null = $state(null)
  let stopAndRunPending: string | null = $state(null)  // name of playground to run after stopping current
  let showAddDep:      boolean       = $state(false)
  let depName:         string        = $state('')
  let depVersion:      string        = $state('')
  let depError:        string | null = $state(null)
  let toasts: { id: number; msg: string }[] = $state([])
  let _toastId = 0

  function showToast(msg: string, durationMs = 4000) {
    const id = ++_toastId
    toasts = [...toasts, { id, msg }]
    setTimeout(() => { toasts = toasts.filter(t => t.id !== id) }, durationMs)
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
  let _lastWindowX: number | null = null
  let _lastWindowY: number | null = null

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
          window_x:      _lastWindowX,
          window_y:      _lastWindowY,
        }
      })
    } catch(e) {
      console.error('saveWindowState failed:', e)
    }
  }

  let _resizeTimer: ReturnType<typeof setTimeout> | null = null
  async function onWindowChange() {
    // Capture position via Tauri API (browser screenX/Y is unreliable in WKWebView)
    try {
      const pos = await getCurrentWindow().outerPosition()
      _lastWindowX = Math.round(pos.x)
      _lastWindowY = Math.round(pos.y)
    } catch {}
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
      enabledLangs = await invoke<string[]>('get_enabled_languages')

      // Edition builds: lock languages to edition's fixed set
      const edition = currentEdition()
      if (edition.languages !== null) {
        enabledLangs = edition.languages
        await invoke('set_enabled_languages', { languages: edition.languages })
      }

      // Show wizard on first launch or if toolchain is missing
      const tc = await invoke<any>('check_toolchain')
      if (!tc.wizard_completed) {
        wizardMode = 'wizard'
        showWizard = true
      }
      if (tc.clang) {
        clangInfo = { path: tc.clang.path ?? '', version: tc.clang.version ?? '' }
      }
      if (tc.zig) {
        zigInfo = { path: tc.zig.path ?? '', version: tc.zig.version ?? '', version_ok: tc.zig.version_ok ?? false }
      }
      if (tc.swiftc) {
        swiftInfo = { version: tc.swiftc.version ?? '' }
      }

      // ── Restore window state ────────────────────────────────────────────────
      const ws = await invoke<any>('get_window_state')

      sidebarVisible = ws.sidebar_visible ?? true
      layoutMode     = ws.layout          ?? 'bottom'
      sidebarW       = ws.sidebar_w       ?? 220
      outputH        = ws.output_h        ?? 240
      outputW        = ws.output_w        ?? 300

      if (ws.window_width && ws.window_height) {
        await getCurrentWindow().setSize(new LogicalSize(ws.window_width, ws.window_height))
          .catch(e => console.warn('setSize failed:', e))
      }
      if (ws.window_x != null && ws.window_y != null) {
        _lastWindowX = ws.window_x
        _lastWindowY = ws.window_y
        await getCurrentWindow().setPosition(new LogicalPosition(ws.window_x, ws.window_y))
          .catch(e => console.warn('setPosition failed:', e))
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
        listen('menu:revert',    () => revert()),
        listen('menu:run',       () => run()),
        listen('menu:stop',      () => stop()),
        listen('menu:new',       () => { if (!isReadOnly) requestNewPlayground() }),
        listen('menu:close-tab', () => closeTab(activeTab)),
        listen('menu:new-project',    () => { switcherPendingMode = 'new' }),
        listen('menu:duplicate-project', () => onDuplicateProject(activeProject)),
        listen('menu:rename-project', () => { switcherPendingMode = 'rename' }),
        listen('menu:delete-project', () => { switcherPendingMode = 'delete-confirm' }),
        listen<string>('menu:switch-project', (e) => switchProject(e.payload)),
        listen('menu:copy-code',         () => copyCodeToClipboard()),
        listen('menu:export-project',    () => exportProject()),
        listen('menu:settings',          () => { wizardMode = 'settings'; showWizard = true }),
        listen('menu:help',              () => { showHelp  = true }),
        listen('menu:about',             () => { showAbout = true }),
        ...allLanguages().filter(l => l.book).map(l => listen(l.book!.menuEvent, () => seedBook(l.type))),
        ...allLanguages().filter(l => l.book).map(l => listen(l.book!.removeMenuEvent, () => removeBook(l.book!.sourceTag, l.book!.commandLabel))),
        listen('menu:rename-playground', () => { if (activeTab && tabMeta[activeTab]?.type === 'playground' && !isReadOnly) renameTarget = activeTab }),
        listen('menu:delete-playground', () => { if (activeTab && tabMeta[activeTab]?.type === 'playground' && !isReadOnly) deletePending = activeTab }),
      ])
    } catch (e) {
      console.error('onMount error:', e)
    } finally {
      // Always show the window — even if something above failed
      await tick()
      await getCurrentWindow().show()
      _restoring = false   // open saves from this point on
    }

    const unlistenMove = await getCurrentWindow().onMoved(() => onWindowChange())

    window.addEventListener('keydown', handleKey)
    window.addEventListener('resize', onWindowChange)
    return () => {
      window.removeEventListener('keydown', handleKey)
      window.removeEventListener('resize', onWindowChange)
      unlistenMove()
      unlisteners.forEach(u => u())
    }
  })

  // Save layout prefs whenever sidebar or layout mode changes
  $effect(() => {
    void sidebarVisible; void layoutMode
    saveWindowState()
  })

  // Rebuild the native menu whenever projects, playground count, or active tab changes
  let hasActivePlayground = $derived(!!activeTab && currentTabMeta.type === 'playground')
  let hasActiveTab = $derived(!!activeTab)
  $effect(() => {
    invoke('rebuild_menu', {
      projects,
      active: activeProject,
      playgroundCount: playgrounds.length,
      hasActivePlayground,
      hasActiveTab,
      projectType,
      isBookProject: isReadOnly,
      projectSources,
      enabledLanguages: enabledLangs,
    }).catch(console.error)
  })

  // Apply theme class to document body whenever resolved theme changes
  $effect(() => {
    document.body.classList.remove('theme-dark', 'theme-light', 'theme-rust', 'theme-seagreen', 'theme-zig', 'theme-swift')
    document.body.classList.add(`theme-${resolvedTheme}`)
  })

  // Listen to OS appearance changes for "system" theme
  $effect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    const handler = (e: MediaQueryListEvent) => { systemDark = e.matches }
    mq.addEventListener('change', handler)
    return () => mq.removeEventListener('change', handler)
  })

  // ── Helpers ──────────────────────────────────────────────────────────────────

  async function loadProjectData() {
    projectType = await invoke<ProjectType>('get_project_type', { name: activeProject }).catch(() => 'rust' as ProjectType)
    playgrounds = await invoke<string[]>('list_playgrounds')
    lockedPlaygrounds = await invoke<string[]>('get_locked_playgrounds').catch(() => [])
    cargoToml   = projectType === 'rust'
      ? await invoke<string>('get_cargo_toml').catch(() => '')
      : ''
    await refreshProjectTypes()
  }

  async function refreshProjectTypes() {
    const [types, sources, rdonly] = await Promise.all([
      (async () => {
        const t: Record<string, ProjectType> = {}
        await Promise.all(projects.map(async (name) => {
          t[name] = await invoke<ProjectType>('get_project_type', { name }).catch(() => 'rust' as ProjectType)
        }))
        return t
      })(),
      invoke<Record<string, string>>('get_project_sources').catch(() => ({})),
      invoke<Record<string, boolean>>('get_project_readonly_map').catch(() => ({})),
    ])
    projectTypes = types
    projectSources = sources
    projectReadonly = rdonly
  }

  function syncMenuProjects() {
    invoke('rebuild_menu', { projects, active: activeProject, playgroundCount: playgrounds.length, hasActivePlayground, hasActiveTab, projectType, isBookProject: isReadOnly, projectSources, enabledLanguages: enabledLangs }).catch(console.error)
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

  async function seedBook(forType?: ProjectType) {
    const ptype = forType ?? projectType
    const bookConfig = getLang(ptype).book
    if (!bookConfig) return
    try {
      const created = await invoke<string[]>('seed_book', { projectType: ptype })
      projects = await invoke<string[]>('list_projects')
      await refreshProjectTypes()
      if (created.length > 0) {
        await switchProject(created[0])  // also rebuilds menu
        showToast(`Loaded ${created.length} ${bookConfig.toastEntity}${created.length > 1 ? 's' : ''}. Starting with ${created[0]}.`)
      } else {
        syncMenuProjects()
        showToast(bookConfig.toastAlreadyLoaded)
      }
    } catch (e) {
      console.error('seedBook failed:', e)
      showToast(`Failed to load ${bookConfig.commandLabel} examples.`)
    }
  }

  async function removeBook(sourceTag: string, label: string) {
    try {
      const removed = await invoke<string[]>('remove_book', { sourceTag })
      if (removed.length === 0) {
        showToast(`No ${label} chapters found.`)
        return
      }
      projects = await invoke<string[]>('list_projects')
      await refreshProjectTypes()
      // If the active project was removed, switch to the first remaining project
      if (removed.includes(activeProject)) {
        if (projects.length > 0) {
          await switchProject(projects[0])
        }
      } else {
        syncMenuProjects()
      }
      showToast(`Removed ${removed.length} ${label} chapter${removed.length > 1 ? 's' : ''}.`)
    } catch (e) {
      console.error('removeBook failed:', e)
      showToast(`Failed to remove ${label} examples.`)
    }
  }

  function contentTabId(filename: string) { return `content:${filename}` }

  function languageForTab(id: string | null, meta: TabMeta): string {
    if (!id) return 'rust'
    if (id === CARGO_TAB) return 'ini'
    if (meta.type === 'content') return languageFromFilename(meta.filename)
    // For file-based languages, playground names include extension
    if (lang.needsExtension) return languageFromFilename(id)
    return 'rust'
  }

  function languageFromFilename(filename: string): string {
    const ext = filename.split('.').pop()?.toLowerCase() ?? ''
    const map: Record<string, string> = {
      rs: 'rust', c: 'c', cpp: 'cpp', zig: 'zig', swift: 'swift',
      json: 'json', toml: 'ini', md: 'markdown',
      yaml: 'yaml', yml: 'yaml', html: 'html', xml: 'xml',
      js: 'javascript', ts: 'typescript', css: 'css',
      sh: 'shell', bash: 'shell',
    }
    return map[ext] ?? 'plaintext'
  }

  // ── Keyboard shortcuts ───────────────────────────────────────────────────────
  // NOTE: ⌘N, ⌘R, ⌘S, ⌘., ⌘W are handled by Tauri menu accelerators (menu.rs).
  // macOS intercepts these before WKWebView, so JS keydown never fires for them.
  // Enable/disable logic lives in build_menu() — single source of truth.
  function handleKey(e: KeyboardEvent) {
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
      // Create a .saved/ snapshot so revert has a clean baseline
      if (meta.type === 'playground') {
        invoke('snapshot_playground', { name }).catch(() => {})
      }
    }
    activeTab = name
    saveWindowState()
  }

  function closeTab(name: string | null) {
    if (!name) return
    const meta = tabMeta[name]
    // If closing a dirty playground tab, restore the saved version on disk
    if (dirtyTabs.includes(name) && meta?.type === 'playground') {
      invoke('revert_playground', { name }).catch(() => {})
    }
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

  // ── Lock toggle ─────────────────────────────────────────────────────────────

  async function toggleReadOnly() {
    if (isBookProject || !activeTab) return
    const newVal = !isPlaygroundLocked
    try {
      await invoke('set_playground_locked', { playground: activeTab, locked: newVal })
      lockedPlaygrounds = newVal
        ? [...lockedPlaygrounds, activeTab]
        : lockedPlaygrounds.filter(n => n !== activeTab)
    } catch (e) {
      console.error('toggleReadOnly failed:', e)
    }
  }

  // ── File operations ──────────────────────────────────────────────────────────

  async function save() {
    if (!activeTab || isReadOnly) return
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

  async function revert() {
    if (!activeTab || isReadOnly) return
    if (!dirtyTabs.includes(activeTab)) return
    const meta = tabMeta[activeTab] ?? { type: 'playground' }
    try {
      let code: string
      if (meta.type === 'playground') {
        code = await invoke<string>('revert_playground', { name: activeTab })
      } else if (meta.type === 'content') {
        code = await invoke<string>('read_content_file', { filename: meta.filename })
      } else if (meta.type === 'cargo') {
        code = await invoke<string>('get_cargo_toml')
      } else {
        return
      }
      tabCode = { ...tabCode, [activeTab]: code }
      dirtyTabs = dirtyTabs.filter(n => n !== activeTab)
    } catch (err) {
      console.error('Revert failed:', err)
    }
  }

  function onCodeChange(newCode: string) {
    if (!activeTab || isReadOnly) return
    tabCode = { ...tabCode, [activeTab]: newCode }
    if (!dirtyTabs.includes(activeTab)) {
      dirtyTabs = [...dirtyTabs, activeTab]
    }
    scheduleCheck(activeTab, newCode)
  }

  // ── Live error checking ────────────────────────────────────────────────────
  // 300ms idle debounce + at most one cargo check in flight at a time.
  // If the user edits while a check is running, the pending edit is queued
  // and fires immediately when the in-flight check finishes.
  let _checkTimer: ReturnType<typeof setTimeout> | null = null
  let _checkRunning = false
  let _pendingCheck: { name: string; code: string } | null = null
  let tabMarkers: Record<string, any[]> = $state({})
  let currentMarkers = $derived(activeTab ? (tabMarkers[activeTab] ?? []) : [])

  function scheduleCheck(name: string, code: string) {
    const meta = tabMeta[name]
    if (meta?.type !== 'playground') return
    if (!lang.supportsLiveCheck) return
    if (_checkTimer) clearTimeout(_checkTimer)
    if (_checkRunning) {
      // A check is already in flight — queue this edit and it will fire
      // as soon as the current check finishes (no additional delay).
      _pendingCheck = { name, code }
      return
    }
    _checkTimer = setTimeout(() => runCheck(name, code), 300)
  }

  async function runCheck(name: string, code: string) {
    _checkRunning = true
    _pendingCheck = null
    const diagnostics: any[] = []
    const channel = new Channel()
    channel.onmessage = (msg: any) => {
      if (msg.type === 'diagnostic') {
        diagnostics.push(msg)
      } else if (msg.type === 'done') {
        tabMarkers = { ...tabMarkers, [name]: diagnostics }
      }
    }
    try {
      await invoke('check_playground', { name, code, onDiagnostics: channel })
    } catch {
      // check failed — ignore silently
    } finally {
      _checkRunning = false
      // If edits arrived while this check was running, fire immediately.
      if (_pendingCheck) {
        const { name: n, code: c } = _pendingCheck
        _pendingCheck = null
        runCheck(n, c)
      }
    }
  }

  function cancelCheck() {
    if (_checkTimer) { clearTimeout(_checkTimer); _checkTimer = null }
    _pendingCheck = null
    invoke('cancel_check').catch(() => {})
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

    // If another playground is running, ask before killing it
    if (runningPlayground && runningPlayground !== activeTab) {
      stopAndRunPending = activeTab
      return
    }

    cancelCheck()
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

    // Build display command based on project type
    const command = lang.runCommandDisplay(name)

    const newBlock: RunBlock = {
      runNum, command, startedAt,
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
        // Cargo/zig print "     Running `target/...`" on stderr when the binary starts.
        // CompileThenRun languages (native, swift) use the same pattern from the backend.
        // Use this to transition from 'compiling' → 'running' so the stdin input appears.
        const isBinaryStart = /^\s*Running\s+`/.test(msg.line)
        updateLastRun(name, r => {
          const nowRunning = r.programStarted || isBinaryStart
          if (isBinaryStart) {
            return { ...r, programStarted: true, status: 'running' as const }
          }
          return {
            ...r,
            ...(nowRunning
              ? { programLines: [...r.programLines, { stream: 'stderr', line: msg.line, ts }] }
              : { compilerLines: [...r.compilerLines, { stream: 'stderr', line: msg.line, ts }] }),
          }
        })
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

  async function confirmStopAndRun() {
    const target = stopAndRunPending
    stopAndRunPending = null
    if (!target) return
    await invoke('kill_playground')
    // Brief delay to let the process group fully terminate
    await new Promise(r => setTimeout(r, 300))
    // Switch to the target tab and run it
    activeTab = target
    await tick()
    run()
  }

  // ── Export / Share ───────────────────────────────────────────────────────────

  async function exportProject() {
    const dest = await dialogOpen({ directory: true, title: 'Export project to…' })
    if (!dest) return // user cancelled
    try {
      await save()
      const path = await invoke<string>('export_project', { dest })
      showToast(`Exported to ${path}`)
    } catch (e) {
      showToast(`Export failed: ${e}`)
    }
  }

  async function copyCodeToClipboard() {
    if (!activeTab || tabMeta[activeTab]?.type !== 'playground') return
    try {
      await navigator.clipboard.writeText(currentCode)
      showToast('Copied to clipboard')
    } catch {
      showToast('Failed to copy')
    }
  }

  // ── Playground CRUD ──────────────────────────────────────────────────────────

  function requestNewPlayground() { creatingNew = true }

  async function onNewPlayground(name: string, template: Template) {
    try {
      await invoke('new_playground', { name, content: template.code || null })

      // Auto-add dependencies if the template requires them (Rust only)
      if (lang.hasCargoToml && template.deps?.length) {
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

  async function onNewProject(name: string, ptype: ProjectType = 'rust') {
    await invoke('new_project', { name, projectType: ptype })
    projects = await invoke<string[]>('list_projects')
    await switchProject(name)
  }

  async function onRenameProject(oldName: string, newName: string) {
    await invoke('rename_project', { oldName, newName })
    projects = await invoke<string[]>('list_projects')
    activeProject = newName
    await refreshProjectTypes()
    syncMenuProjects()
  }

  async function onDuplicateProject(name: string) {
    const newName = await invoke<string>('duplicate_project', { name })
    projects = await invoke<string[]>('list_projects')
    await switchProject(newName)
  }

  async function onDeleteProject(name: string) {
    const remaining = projects.filter(p => p !== name)
    let switchTo = remaining[0]
    if (!switchTo) {
      const helloNames: Record<string, string> = { rust: 'hello_rust', clang: 'hello_c', zig: 'hello_zig', swift: 'hello_swift' }
      const firstLang = enabledLangs[0] ?? 'rust'
      const fallbackName = helloNames[firstLang] ?? `hello_${firstLang}`
      await invoke('new_project', { name: fallbackName, projectType: firstLang })
      switchTo = fallbackName
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
        {projectType}
        {projectTypes}
        {projectSources}
        {projectReadonly}
        enabledLanguages={enabledLangs}
        edition={currentEdition()}
        onswitch={switchProject}
        onnew={onNewProject}
        onrename={onRenameProject}
        ondelete={onDeleteProject}
        onduplicate={onDuplicateProject}
        onloadbook={(ptype) => seedBook(ptype)}
        bind:pendingMode={switcherPendingMode}
      />
    </div>

    <div class="toolbar-center">
      <button
        class="settings-btn"
        title="Settings (⌘,)"
        onclick={() => { wizardMode = 'settings'; showWizard = true }}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M6.8 1.5h2.4l.3 1.7.8.3 1.4-1 1.7 1.7-1 1.4.3.8 1.7.3v2.4l-1.7.3-.3.8 1 1.4-1.7 1.7-1.4-1-.8.3-.3 1.7H6.8l-.3-1.7-.8-.3-1.4 1-1.7-1.7 1-1.4-.3-.8-1.7-.3V6.8l1.7-.3.3-.8-1-1.4 1.7-1.7 1.4 1 .8-.3.3-1.7z"
                stroke="currentColor" stroke-width="1.1" stroke-linejoin="round" fill="none"/>
          <circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.1" fill="none"/>
        </svg>
      </button>
      <span class="toolchain-pill" class:pill-red={pillStatus === 'not-enabled' || pillStatus === 'missing'} class:pill-yellow={pillStatus === 'partial'} title="{pillStatus === 'not-enabled' ? 'Language support not enabled in Settings' : activeToolchain.path ?? ''}">
        {#if projectType === 'clang'}
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M5 2C3.5 2 2 3 2 5L2 9C2 11 3.5 12 5 12"
                  stroke="currentColor" stroke-width="1.4" stroke-linecap="round" fill="none"/>
            <path d="M9 2C10.5 2 12 3 12 5L12 9C12 11 10.5 12 9 12"
                  stroke="currentColor" stroke-width="1.4" stroke-linecap="round" fill="none"/>
          </svg>
        {:else if projectType === 'zig'}
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M8 1L3 8h4l-1 5 5-7H7l1-5z"
                  stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" fill="none"/>
          </svg>
        {:else if projectType === 'swift'}
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M11 3C8.5 5 5 7 3 8c2-0.5 4-0.5 6 0.5-1.5 2-4 3-7 3 4 0 7.5-1.5 9-4.5 0.5-1 0.5-2.5 0-4z"
                  stroke="currentColor" stroke-width="1.1" stroke-linejoin="round" fill="none"/>
          </svg>
        {:else}
          <svg width="14" height="12" viewBox="0 0 14 12" fill="none">
            <path d="M7 1L13 4L7 7L1 4Z"
                  stroke="currentColor" stroke-width="1.15" stroke-linejoin="round"/>
            <path d="M1 4L1 8.5L7 11.5L7 7Z"
                  stroke="currentColor" stroke-width="1.15" stroke-linejoin="round"/>
            <path d="M13 4L13 8.5L7 11.5L7 7Z"
                  stroke="currentColor" stroke-width="1.15" stroke-linejoin="round"/>
          </svg>
        {/if}
        {pillText}
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

      <button
        class="lock-btn {isReadOnly ? 'lock-btn-locked' : 'lock-btn-unlocked'}"
        onclick={toggleReadOnly}
        disabled={isBookProject || !activeTab}
        title={isBookProject ? 'Book project — always read-only' : !activeTab ? 'No playground open' : isReadOnly ? 'Unlock this playground' : 'Lock this playground'}
      >
        {#if isReadOnly}
          <svg width="12" height="13" viewBox="0 0 12 13" fill="none">
            <rect x="1.5" y="6" width="9" height="6" rx="1.5" stroke="currentColor" stroke-width="1.3" fill="none"/>
            <path d="M3.5 6V4.5a2.5 2.5 0 0 1 5 0V6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" fill="none"/>
          </svg>
        {:else}
          <svg width="12" height="13" viewBox="0 0 12 13" fill="none">
            <rect x="1.5" y="6" width="9" height="6" rx="1.5" stroke="currentColor" stroke-width="1.3" fill="none"/>
            <path d="M8.5 6V4.5a2.5 2.5 0 0 0-5 0" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" fill="none"/>
          </svg>
        {/if}
      </button>

      <div class="export-wrap">
        <button class="btn btn-export" onclick={() => showExportMenu = !showExportMenu} title="Export / Share">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M6 1v7M3 3.5 6 1l3 2.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M2 7v3.5h8V7" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        {#if showExportMenu}
          <div class="export-dropdown" role="menu">
            <button class="export-item" onclick={() => { showExportMenu = false; exportProject() }}>Export Project…</button>
            {#if activeTab && tabMeta[activeTab]?.type === 'playground'}
              <button class="export-item" onclick={() => { showExportMenu = false; copyCodeToClipboard() }}>Copy Code to Clipboard</button>
              {#if isReadOnly}
                <button class="export-item" onclick={() => { showExportMenu = false; copyToProjectPlayground = activeTab; showCopyToProject = true }}>Copy to Project…</button>
              {/if}
            {/if}
          </div>
          <div class="export-backdrop" onclick={() => showExportMenu = false} aria-hidden="true"></div>
        {/if}
      </div>

      {#if activeTab}
        <button
          class="btn btn-save"
          onclick={save}
          disabled={isReadOnly || !dirtyTabs.includes(activeTab)}
          title={isReadOnly ? 'Save disabled — project is read-only' : 'Save (⌘S)'}
        >
          <svg width="11" height="11" viewBox="0 0 12 12" fill="none">
            <path d="M2 1h7l2 2v8H2V1z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" fill="none"/>
            <rect x="4" y="7" width="4" height="4" rx="0.5" stroke="currentColor" stroke-width="1.3" fill="none"/>
            <rect x="3.5" y="1" width="5" height="3" rx="0.5" stroke="currentColor" stroke-width="1.3" fill="none"/>
          </svg>
          Save
        </button>
        <button
          class="btn btn-revert"
          onclick={revert}
          disabled={isReadOnly || !dirtyTabs.includes(activeTab)}
          title="Revert to last saved version"
        >
          <svg width="11" height="11" viewBox="0 0 12 12" fill="none">
            <path d="M2 6a4.5 4.5 0 1 1 1 3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" fill="none"/>
            <path d="M2 9V6h3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
          </svg>
          Revert
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
          {projectType}
          readOnly={isReadOnly}
          onNewPlayground={requestNewPlayground}
          bind:renameTarget
          on:select={(e) => openTab(e.detail, { type: 'playground' })}
          on:rename={onRename}
          on:delete={onDelete}
          on:duplicate={onDuplicate}
          on:copyToProject={(e) => { copyToProjectPlayground = e.detail; showCopyToProject = true }}
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
              theme={monacoTheme}
              diagnostics={currentMarkers}
              readOnly={isReadOnly}
              onSave={save}
              onRun={run}
              onNew={requestNewPlayground}
              onChange={onCodeChange}
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
              {#if isReadOnly}
                <p class="empty-hint">Select a playground from the sidebar to view it</p>
              {:else}
                <p class="empty-hint">
                  Select one from the sidebar or
                  <button class="link-btn" onclick={requestNewPlayground}>create a new one</button>
                </p>
                <div class="shortcut-grid">
                  <span class="shortcut-key">⌘N</span><span class="shortcut-desc">New playground</span>
                  <span class="shortcut-key">⌘R</span><span class="shortcut-desc">Run</span>
                  <span class="shortcut-key">⌘S</span><span class="shortcut-desc">Save</span>
                </div>
              {/if}
              {#each [allLanguages().filter(l => enabledLangs.includes(l.type) && l.book && !Object.values(projectSources).includes(l.book!.sourceTag))] as unloadedBooks}
                {#if unloadedBooks.length > 0}
                  <div class="empty-books">
                    <p class="empty-books-label">Learn from examples</p>
                    <div class="empty-books-list">
                      {#each unloadedBooks as l (l.type)}
                        <button class="empty-book-btn" onclick={() => seedBook(l.type)}>
                          <span class="empty-book-icon">📖</span>
                          {l.book?.commandLabel}
                        </button>
                      {/each}
                    </div>
                  </div>
                {/if}
              {/each}
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
  <HelpModal onclose={() => showHelp = false} enabledLanguages={enabledLangs} edition={currentEdition()} />
{/if}

{#if showAbout}
  <AboutModal onclose={() => showAbout = false} edition={currentEdition()} />
{/if}

{#if showWizard}
  <ToolchainWizard
    mode={wizardMode}
    enabledLanguages={enabledLangs}
    {settings}
    {projectSources}
    edition={currentEdition()}
    onthemechange={(t) => { settings = { ...settings, theme: t } }}
    onapply={async (result) => {
      enabledLangs = result.enabledLanguages
      await invoke('set_enabled_languages', { languages: result.enabledLanguages })
      settings = result.settings
      await invoke('save_settings', { settings: result.settings })
      for (const ptype of result.booksToRemove) {
        const book = getLang(ptype as ProjectType).book
        if (book) await removeBook(book.sourceTag, book.commandLabel)
      }
      for (const ptype of result.booksToLoad) {
        await seedBook(ptype)
      }
      syncMenuProjects()
    }}
    onclose={async (result) => {
      enabledLangs = result.enabledLanguages
      await invoke('set_enabled_languages', { languages: result.enabledLanguages })
      if (result.settings) {
        settings = result.settings
        await invoke('save_settings', { settings: result.settings })
      }
      // Create hello projects for each selected language (wizard mode only)
      if (wizardMode === 'wizard') {
        const helloNames: Record<string, string> = { rust: 'hello_rust', clang: 'hello_c', zig: 'hello_zig', swift: 'hello_swift' }
        let firstProject = ''
        for (const ptype of result.enabledLanguages) {
          const name = helloNames[ptype] ?? `hello_${ptype}`
          try {
            await invoke('new_project', { name, projectType: ptype })
            if (!firstProject) firstProject = name
          } catch { /* project may already exist */ }
        }
        if (firstProject) {
          await switchProject(firstProject)
        }
      }
      for (const ptype of result.booksToRemove) {
        const book = getLang(ptype as ProjectType).book
        if (book) await removeBook(book.sourceTag, book.commandLabel)
      }
      for (const ptype of result.booksToLoad) {
        await seedBook(ptype)
      }
      showWizard = false
      syncMenuProjects()
    }}
  />
{/if}

{#if creatingNew}
  <NewPlaygroundModal
    existingNames={playgrounds}
    {projectType}
    onclose={() => creatingNew = false}
    oncreate={onNewPlayground}
  />
{/if}

{#if showCopyToProject && copyToProjectPlayground}
  <CopyToProjectModal
    playgroundName={copyToProjectPlayground}
    code={tabCode[copyToProjectPlayground] ?? ''}
    {projects}
    {projectTypes}
    {projectSources}
    {projectType}
    currentProject={activeProject}
    onclose={() => { showCopyToProject = false; copyToProjectPlayground = null }}
    oncopy={(targetProject, newName) => {
      showCopyToProject = false
      copyToProjectPlayground = null
      showToast(`Copied to ${targetProject} / ${newName}`)
    }}
  />
{/if}

{#if toasts.length > 0}
  <div class="toast-stack" role="status" aria-live="polite">
    {#each toasts as toast (toast.id)}
      <div class="toast">{toast.msg}</div>
    {/each}
  </div>
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

{#if stopAndRunPending}
  <div class="confirm-backdrop" onclick={() => stopAndRunPending = null} aria-hidden="true"></div>
  <div class="confirm-dialog" role="alertdialog" aria-modal="true">
    <p class="confirm-msg"><strong>{runningPlayground}</strong> is running.<br><span class="confirm-sub">Stop it and run <strong>{stopAndRunPending}</strong> instead?</span></p>
    <div class="confirm-actions">
      <button class="confirm-cancel" onclick={() => stopAndRunPending = null}>No</button>
      <button class="confirm-proceed" onclick={confirmStopAndRun}>Yes, stop and run</button>
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
  .lock-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }
  .lock-btn-locked {
    background: transparent;
    color: #d44;
  }
  .lock-btn-locked:hover:not(:disabled) {
    background: rgba(220, 60, 60, 0.12);
  }
  .lock-btn-unlocked {
    background: transparent;
    color: #2ea043;
  }
  .lock-btn-unlocked:hover:not(:disabled) {
    background: rgba(45, 160, 65, 0.12);
  }
  .lock-btn:disabled {
    background: transparent;
    color: var(--text-tertiary);
    cursor: default;
  }

  .btn {
    display: flex; align-items: center; gap: 5px;
    height: 28px; box-sizing: border-box;
    padding: 0 10px; font-size: 12px; font-weight: 500;
    border-radius: 6px;
    background: transparent; color: var(--text-secondary);
    transition: background 0.12s, color 0.12s, opacity 0.12s;
    flex-shrink: 0;
  }
  .btn:hover:not(:disabled) {
    background: var(--bg-hover); color: var(--text);
  }

  .btn-save { color: var(--text-secondary); }
  .btn-save:hover:not(:disabled) { color: var(--text); }
  .btn-save:disabled { opacity: 0.3; cursor: not-allowed; }
  .btn-revert { color: var(--text-secondary); }
  .btn-revert:hover:not(:disabled) { color: var(--text); }
  .btn-revert:disabled { opacity: 0.3; cursor: not-allowed; }

  .btn-run {
    background: rgba(var(--accent-rgb, 229, 115, 0), 0.15);
    color: var(--accent);
  }
  .btn-run:hover:not(:disabled) {
    background: rgba(var(--accent-rgb, 229, 115, 0), 0.25);
  }
  .btn-run:disabled { opacity: 0.3; cursor: not-allowed; }

  .btn-stop {
    background: rgba(220, 60, 60, 0.15);
    color: var(--red);
  }
  .btn-stop:hover { background: rgba(220, 60, 60, 0.25); }

  .btn-export { color: var(--text-secondary); }
  .btn-export:hover { color: var(--text); }

  .export-wrap { position: relative; }
  .export-backdrop {
    position: fixed; inset: 0; z-index: 99;
  }
  .export-dropdown {
    position: absolute; top: 100%; right: 0; z-index: 100;
    margin-top: 4px;
    min-width: 200px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
    padding: 4px 0;
  }
  .export-item {
    display: block; width: 100%;
    padding: 6px 12px;
    font-size: 12px; text-align: left;
    color: var(--text); background: none; border: none; border-radius: 0;
    cursor: pointer;
  }
  .export-item:hover { background: var(--accent); color: #fff; }

  .settings-btn {
    display: flex; align-items: center; justify-content: center;
    width: 28px; height: 28px;
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
    white-space: nowrap;
    transition: background 0.12s, border-color 0.12s, color 0.12s;
  }
  .toolchain-pill.pill-red {
    color: #f44; border-color: rgba(255, 68, 68, 0.4);
    background: rgba(255, 68, 68, 0.1);
  }
  .toolchain-pill.pill-yellow {
    color: #e8a820; border-color: rgba(232, 168, 32, 0.4);
    background: rgba(232, 168, 32, 0.1);
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

  .empty-books {
    margin-top: 24px; display: flex; flex-direction: column; align-items: center; gap: 8px;
  }
  .empty-books-label {
    font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em;
    color: var(--text-tertiary);
  }
  .empty-books-list { display: flex; gap: 8px; flex-wrap: wrap; justify-content: center; }
  .empty-book-btn {
    display: flex; align-items: center; gap: 5px;
    padding: 6px 12px;
    font-size: 12px; font-weight: 500; color: var(--text-secondary);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    transition: background 0.1s, border-color 0.1s, color 0.1s;
  }
  .empty-book-btn:hover { background: var(--bg-hover); border-color: var(--border-strong); color: var(--text); }
  .empty-book-icon { font-size: 13px; }

  .shortcut-key {
    font-family: var(--font-ui); font-size: 11px;
    background: var(--bg-elevated); border: 1px solid var(--border-strong);
    border-radius: var(--radius-xs); padding: 2px 7px;
    color: var(--text-secondary); text-align: center; justify-self: end;
  }

  .shortcut-desc { font-size: 12px; color: var(--text-tertiary); }

  /* ── Toast notification ── */
  .toast-stack {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 400;
    display: flex;
    flex-direction: column-reverse;
    gap: 6px;
    align-items: center;
    pointer-events: none;
  }
  .toast {
    background: rgba(30, 32, 40, 0.94);
    color: #e8e8e8;
    border: 1px solid rgba(255,255,255,0.12);
    border-radius: 8px;
    padding: 10px 18px;
    font-size: 13px;
    line-height: 1.4;
    backdrop-filter: blur(8px);
    box-shadow: 0 4px 20px rgba(0,0,0,0.45);
    white-space: nowrap;
    animation: toast-in 0.18s ease;
  }
  @keyframes toast-in {
    from { opacity: 0; transform: translateY(8px); }
    to   { opacity: 1; transform: translateY(0);   }
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
  .confirm-proceed {
    font-size: 12px; padding: 5px 12px; border-radius: 6px;
    background: var(--accent); border: 1px solid var(--accent);
    color: #fff;
  }
  .confirm-proceed:hover { background: var(--accent-hover); }
</style>
