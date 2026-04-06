<script lang="ts">
  import { onMount, tick, createEventDispatcher } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { getCurrentWebview } from '@tauri-apps/api/webview'

  const dispatch = createEventDispatcher()

  let {
    playgrounds,
    selected,
    dirtyTabs,
    cargoToml = '',
    projectType = 'rust',
    readOnly = false,
    onNewPlayground = () => {},
    renameTarget = $bindable(null),
  }: {
    playgrounds: string[]
    selected: string | null
    dirtyTabs: string[]
    cargoToml: string
    projectType?: import('./languages').ProjectType
    readOnly?: boolean
    onNewPlayground?: () => void
    renameTarget?: string | null
  } = $props()

  import { getLang } from './languages'
  import LanguageLogo from './LanguageLogo.svelte'
  let lang = $derived(getLang(projectType))

  // ── Sidebar tab ───────────────────────────────────────────────────────────────
  let sidebarTab: 'playgrounds' | 'content' = $state('playgrounds')

  // ── Playgrounds tab state ─────────────────────────────────────────────────────
  let contextMenu: { x: number; y: number; name: string } | null = $state(null)
  let renamingName: string | null = $state(null)
  let renameValue: string = $state('')
  let searchQuery: string = $state('')

  let filtered = $derived(
    searchQuery.trim() === ''
      ? playgrounds
      : playgrounds.filter(n => n.toLowerCase().includes(searchQuery.toLowerCase()))
  )


  // Allow parent to trigger rename mode via the renameTarget prop
  $effect(() => {
    if (renameTarget && playgrounds.includes(renameTarget)) {
      sidebarTab = 'playgrounds'
      startRename(renameTarget)
      renameTarget = null
    }
  })

  function openContext(e: MouseEvent, name: string) {
    e.preventDefault()
    contextMenu = { x: e.clientX, y: e.clientY, name }
  }

  function closeContext() {
    contextMenu = null
    fileContextMenu = null
  }

  function startRename(name: string) {
    renamingName = name
    renameValue = name
    contextMenu = null
  }

  function commitRename() {
    if (renamingName && renameValue && renameValue !== renamingName) {
      dispatch('rename', { old: renamingName, new: renameValue })
    }
    renamingName = null
  }

  function handleRenameKey(e: KeyboardEvent) {
    if (e.key === 'Enter') commitRename()
    if (e.key === 'Escape') renamingName = null
  }

  let cargoExpanded: boolean = $state(false)

  // ── Compiler flags state (file-based projects with build flags) ────────────────
  let flagsExpanded: boolean = $state(false)
  let cflagsText: string = $state('')
  let cxxflagsText: string = $state('')
  let zigflagsText: string = $state('')
  let swiftflagsText: string = $state('')

  // Load build flags when switching to a project with build flags
  $effect(() => {
    if (lang.hasBuildFlags) {
      invoke<any>('get_build_flags')
        .then(flags => {
          cflagsText = (flags.cflags ?? []).join(' ')
          cxxflagsText = (flags.cxxflags ?? []).join(' ')
          zigflagsText = (flags.zigflags ?? []).join(' ')
          swiftflagsText = (flags.swiftflags ?? []).join(' ')
        })
        .catch(() => {})
    }
  })

  function saveBuildFlags() {
    const cflags = cflagsText.trim().split(/\s+/).filter(Boolean)
    const cxxflags = cxxflagsText.trim().split(/\s+/).filter(Boolean)
    const zigflags = zigflagsText.trim().split(/\s+/).filter(Boolean)
    const swiftflags = swiftflagsText.trim().split(/\s+/).filter(Boolean)
    invoke('save_build_flags', { cflags, cxxflags, zigflags, swiftflags }).catch(console.error)
  }

  // ── Content tab state ─────────────────────────────────────────────────────────
  export type ContentFile = { filename: string; size_bytes: number; is_text: boolean }

  let contentFiles: ContentFile[] = $state([])
  let contentLoading: boolean = $state(false)
  let isDragOver: boolean = $state(false)

  let creatingFile: boolean = $state(false)
  let newFileValue: string = $state('')
  let newFileError: string = $state('')
  let newFileInputEl: HTMLInputElement | null = $state(null)

  let contentSearch: string = $state('')
  let filteredFiles = $derived(
    contentSearch.trim() === ''
      ? contentFiles
      : contentFiles.filter(f => f.filename.toLowerCase().includes(contentSearch.toLowerCase()))
  )

  let fileContextMenu: { x: number; y: number; filename: string } | null = $state(null)
  let renamingFile: string | null = $state(null)
  let renameFileValue: string = $state('')
  let renameFileInputEl: HTMLInputElement | null = $state(null)

  // Load content files whenever the Content tab is active
  $effect(() => {
    if (sidebarTab === 'content') loadContentFiles()
  })

  // Focus new-file input when creatingFile becomes true
  $effect(() => {
    if (creatingFile) {
      newFileValue = ''
      newFileError = ''
      tick().then(() => newFileInputEl?.focus())
    }
  })

  // Focus rename input when renamingFile changes
  $effect(() => {
    if (renamingFile) tick().then(() => renameFileInputEl?.focus())
  })

  async function loadContentFiles() {
    contentLoading = true
    try {
      contentFiles = await invoke<ContentFile[]>('list_content_files')
    } catch {
      contentFiles = []
    } finally {
      contentLoading = false
    }
  }

  function fileIcon(filename: string): string {
    if (!filename.includes('.')) return '📦'
    const ext = filename.split('.').pop()?.toLowerCase() ?? ''
    if (['png','jpg','jpeg','gif','webp','svg'].includes(ext)) return '🖼'
    const textExts = ['txt','md','rs','toml','yaml','yml','json','xml','html','htm',
                      'css','js','ts','csv','log','sh','bash','zsh','conf','ini','env']
    if (textExts.includes(ext)) return '📄'
    return '📦'
  }

  function isTextFile(filename: string): boolean {
    if (!filename.includes('.')) return false
    const ext = filename.split('.').pop()?.toLowerCase() ?? ''
    const textExts = ['txt','md','rs','toml','yaml','yml','json','xml','html','htm',
                      'css','js','ts','csv','log','sh','bash','zsh','conf','ini','env']
    return textExts.includes(ext)
  }

  function clickFile(file: ContentFile) {
    if (file.is_text) {
      dispatch('opencontentfile', { filename: file.filename })
    } else {
      invoke<string>('get_content_file_path', { filename: file.filename })
        .then(path => invoke('reveal_in_finder', { path }))
    }
  }

  function openFileContext(e: MouseEvent, filename: string) {
    e.preventDefault()
    e.stopPropagation()
    fileContextMenu = { x: e.clientX, y: e.clientY, filename }
  }

  async function deleteFile(filename: string) {
    fileContextMenu = null
    await invoke('delete_content_file', { filename })
    await loadContentFiles()
  }

  function startRenameFile(filename: string) {
    renamingFile = filename
    renameFileValue = filename
    fileContextMenu = null
  }

  async function commitRenameFile() {
    if (!renamingFile) return
    const newName = renameFileValue.trim()
    if (newName && newName !== renamingFile) {
      try {
        await invoke('rename_content_file', { oldFilename: renamingFile, newFilename: newName })
        await loadContentFiles()
      } catch { /* rename failed — just reset */ }
    }
    renamingFile = null
  }

  function handleRenameFileKey(e: KeyboardEvent) {
    if (e.key === 'Enter') commitRenameFile()
    if (e.key === 'Escape') renamingFile = null
  }

  async function revealFile(filename: string) {
    fileContextMenu = null
    const path = await invoke<string>('get_content_file_path', { filename })
    await invoke('reveal_in_finder', { path })
  }

  // ── New file ──────────────────────────────────────────────────────────────────
  function startNewFile() {
    creatingFile = true
  }

  async function confirmNewFile() {
    const filename = newFileValue.trim()
    if (!filename) { cancelNewFile(); return }
    if (filename.includes('/') || filename.includes('\\')) {
      newFileError = 'Invalid name'
      return
    }
    try {
      await invoke('create_content_file', { filename })
      creatingFile = false
      newFileValue = ''
      newFileError = ''
      await loadContentFiles()
      if (isTextFile(filename)) {
        dispatch('opencontentfile', { filename })
      }
    } catch (err: any) {
      newFileError = String(err).includes('already exists') ? 'Already exists' : String(err)
    }
  }

  function cancelNewFile() {
    creatingFile = false
    newFileValue = ''
    newFileError = ''
  }

  function handleNewFileKey(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); confirmNewFile() }
    if (e.key === 'Escape') cancelNewFile()
  }

  // ── Drag and drop ─────────────────────────────────────────────────────────────
  // Tauri v2 API: getCurrentWebview().onDragDropEvent()
  // (The v1 listen('tauri://drag-drop') API was removed in Tauri v2)
  onMount(async () => {
    const unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
      if (event.payload.type === 'enter' || event.payload.type === 'over') {
        if (sidebarTab === 'content') isDragOver = true
      } else if (event.payload.type === 'drop') {
        if (sidebarTab === 'content') {
          isDragOver = false
          const paths = (event.payload as any).paths as string[]
          for (const srcPath of paths) {
            try {
              await invoke('import_content_file', { srcPath })
            } catch { /* ignore individual file errors */ }
          }
          await loadContentFiles()
        } else {
          isDragOver = false
        }
      } else if (event.payload.type === 'leave') {
        isDragOver = false
      }
    })
    return unlisten
  })
</script>

<svelte:window onclick={closeContext} />

<aside class="sidebar">

  <!-- ── Tab strip ── -->
  <div class="tab-strip">
    <button
      class="strip-tab"
      class:active={sidebarTab === 'playgrounds'}
      onclick={() => sidebarTab = 'playgrounds'}
    >Playgrounds</button>
    <button
      class="strip-tab"
      class:active={sidebarTab === 'content'}
      onclick={() => sidebarTab = 'content'}
    >Content</button>
  </div>

  <!-- ════════════════════════════════════════ -->
  <!-- PLAYGROUNDS TAB                         -->
  <!-- ════════════════════════════════════════ -->
  {#if sidebarTab === 'playgrounds'}

    <div class="sidebar-header">
      <span class="sidebar-title">Playgrounds</span>
      <button class="icon-btn" title="New playground (⌘N)" onclick={onNewPlayground} disabled={readOnly}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="search-wrap">
      <svg class="search-icon" width="12" height="12" viewBox="0 0 12 12" fill="none">
        <circle cx="5" cy="5" r="3.5" stroke="currentColor" stroke-width="1.4"/>
        <path d="M8 8l2.5 2.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
      </svg>
      <input
        class="search-input" type="search" placeholder="Filter"
        bind:value={searchQuery} onclick={(e) => e.stopPropagation()}
      />
      {#if searchQuery}
        <button class="search-clear" onclick={() => searchQuery = ''}>×</button>
      {/if}
    </div>

    <ul class="playground-list">
      {#each filtered as name (name)}
        {@const isDirty = dirtyTabs.includes(name)}
        <li
          class="playground-item"
          class:active={selected === name}
          onclick={() => dispatch('select', name)}
          oncontextmenu={(e) => openContext(e, name)}
        >
          {#if renamingName === name}
            <input
              class="rename-input" bind:value={renameValue}
              onblur={commitRename} onkeydown={handleRenameKey}
              onclick={(e) => e.stopPropagation()} autofocus
            />
          {:else}
            {@const badgeType = lang.needsExtension ? (name.split('.').pop()?.toLowerCase() ?? '') : 'rs'}
            <span class="file-icon-logo">
              <LanguageLogo type={badgeType === 'rs' ? 'rust' : badgeType} size={14} />
            </span>
            {#if readOnly}
              <svg class="lock-icon" width="9" height="10" viewBox="0 0 10 11" fill="none">
                <rect x="1" y="5" width="8" height="5.5" rx="1.2" stroke="currentColor" stroke-width="1.1" fill="none"/>
                <path d="M3 5V3.5a2 2 0 0 1 4 0V5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" fill="none"/>
              </svg>
            {/if}
            <span class="name">{name}</span>
            {#if isDirty}<span class="dirty-dot" title="Unsaved changes">●</span>{/if}
          {/if}
        </li>
      {/each}

      {#if filtered.length === 0}
        <li class="empty-hint">
          {searchQuery ? 'No matches' : 'No playgrounds yet'}
        </li>
      {/if}
    </ul>

    <!-- Cargo.toml panel (Rust projects only) -->
    {#if lang.hasCargoToml}
    <div class="cargo-section">
      <div
        class="cargo-header"
        role="button" tabindex="0"
        onclick={() => cargoExpanded = !cargoExpanded}
        onkeydown={(e) => e.key === 'Enter' && (cargoExpanded = !cargoExpanded)}
      >
        <svg width="10" height="10" viewBox="0 0 10 10" class="chevron" class:open={cargoExpanded}>
          <path d="M2 3.5L5 6.5L8 3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" fill="none"/>
        </svg>
        <span class="file-icon-logo"><LanguageLogo type="cargo" size={14} /></span>
        <span class="cargo-label">Cargo.toml</span>
        <button
          class="cargo-edit-btn"
          title="Edit Cargo.toml"
          onclick={(e) => { e.stopPropagation(); dispatch('editcargo') }}
        >Edit</button>
      </div>
      {#if cargoExpanded}
        <div class="cargo-body">
          {#if cargoToml}
            <pre class="cargo-pre">{cargoToml}</pre>
          {:else}
            <span class="cargo-empty">Loading…</span>
          {/if}
        </div>
      {/if}
    </div>
    {/if}

    <!-- Build Flags panel (projects with build flags) -->
    {#if lang.hasBuildFlags}
    <div class="cargo-section">
      <div
        class="cargo-header"
        role="button" tabindex="0"
        onclick={() => flagsExpanded = !flagsExpanded}
        onkeydown={(e) => e.key === 'Enter' && (flagsExpanded = !flagsExpanded)}
      >
        <svg width="10" height="10" viewBox="0 0 10 10" class="chevron" class:open={flagsExpanded}>
          <path d="M2 3.5L5 6.5L8 3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" fill="none"/>
        </svg>
        <span class="cargo-label">Build Flags</span>
      </div>
      {#if flagsExpanded}
        <div class="flags-body">
          {#if projectType === 'clang'}
            <label class="flags-label">C flags</label>
            <input class="flags-input" type="text" value={cflagsText} placeholder="-lsqlite3" spellcheck="false"
              onchange={(e) => { cflagsText = e.currentTarget.value; saveBuildFlags() }} />
            <label class="flags-label">C++ flags</label>
            <input class="flags-input" type="text" value={cxxflagsText} placeholder="-std=c++17 -lsqlite3" spellcheck="false"
              onchange={(e) => { cxxflagsText = e.currentTarget.value; saveBuildFlags() }} />
          {:else if projectType === 'zig'}
            <label class="flags-label">Zig flags</label>
            <input class="flags-input" type="text" value={zigflagsText} placeholder="-O ReleaseSafe" spellcheck="false"
              onchange={(e) => { zigflagsText = e.currentTarget.value; saveBuildFlags() }} />
          {:else if projectType === 'swift'}
            <label class="flags-label">Swift flags</label>
            <input class="flags-input" type="text" value={swiftflagsText} placeholder="-O" spellcheck="false"
              onchange={(e) => { swiftflagsText = e.currentTarget.value; saveBuildFlags() }} />
          {/if}
          <span class="flags-hint">Space-separated. Saved to rustic.toml.</span>
        </div>
      {/if}
    </div>
    {/if}

  <!-- ════════════════════════════════════════ -->
  <!-- CONTENT TAB                             -->
  <!-- ════════════════════════════════════════ -->
  {:else}

    <div class="content-header">
      <span class="sidebar-title">Content</span>
      <button class="icon-btn" title="New file" onclick={startNewFile}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="search-wrap">
      <svg class="search-icon" width="12" height="12" viewBox="0 0 12 12" fill="none">
        <circle cx="5" cy="5" r="3.5" stroke="currentColor" stroke-width="1.4"/>
        <path d="M8 8l2.5 2.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
      </svg>
      <input
        class="search-input" type="search" placeholder="Filter"
        bind:value={contentSearch} onclick={(e) => e.stopPropagation()}
      />
      {#if contentSearch}
        <button class="search-clear" onclick={() => contentSearch = ''}>×</button>
      {/if}
    </div>

    <!-- Drop zone wraps the whole file list area -->
      <div
        class="content-pane"
        class:drag-over={isDragOver}
        ondragover={(e) => { e.preventDefault(); isDragOver = true }}
        ondragleave={(e) => {
          // Only clear if leaving the pane itself, not a child
          if (!e.currentTarget.contains(e.relatedTarget as Node)) isDragOver = false
        }}
        ondrop={(e) => { e.preventDefault(); isDragOver = false }}
        role="region"
        aria-label="Content files drop zone"
      >
        {#if contentLoading}
          <div class="content-loading">Loading…</div>

        {:else if contentFiles.length === 0 && !creatingFile}
          <div class="content-empty-drop" class:hidden={contentSearch.trim() !== ''}>
            <div class="drop-icon">
              <svg width="32" height="32" viewBox="0 0 32 32" fill="none" opacity="0.3">
                <rect x="4" y="2" width="20" height="26" rx="3" stroke="currentColor" stroke-width="1.5"/>
                <path d="M10 9h10M10 14h8M10 19h5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <path d="M24 20v8M20 24l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <p class="drop-label">Drop files here</p>
            <p class="drop-sub">or</p>
            <button class="new-file-btn" onclick={startNewFile}>+ New File</button>
          </div>

        {:else}
          <ul class="file-list">
            {#if filteredFiles.length === 0 && contentSearch.trim() !== ''}
              <li class="empty-hint">No matches</li>
            {/if}
            {#each filteredFiles as file (file.filename)}
              <li
                class="file-item"
                onclick={() => clickFile(file)}
                oncontextmenu={(e) => openFileContext(e, file.filename)}
              >
                {#if renamingFile === file.filename}
                  <span class="file-type-icon">{fileIcon(file.filename)}</span>
                  <input
                    class="rename-file-input"
                    bind:value={renameFileValue}
                    bind:this={renameFileInputEl}
                    onblur={commitRenameFile}
                    onkeydown={handleRenameFileKey}
                    onclick={(e) => e.stopPropagation()}
                  />
                {:else}
                  <span class="file-type-icon">{fileIcon(file.filename)}</span>
                  <span class="file-name">{file.filename}</span>
                {/if}
              </li>
            {/each}

            <!-- Inline new file input at bottom of list -->
            {#if creatingFile}
              <li class="new-file-item">
                <span class="file-type-icon">📄</span>
                <div class="new-file-wrap">
                  <input
                    class="new-file-input"
                    type="text"
                    placeholder="filename.txt"
                    bind:value={newFileValue}
                    bind:this={newFileInputEl}
                    onkeydown={handleNewFileKey}
                    onblur={() => setTimeout(() => { if (creatingFile) cancelNewFile() }, 150)}
                    onclick={(e) => e.stopPropagation()}
                  />
                  {#if newFileError}
                    <span class="new-file-error">{newFileError}</span>
                  {/if}
                </div>
                <button class="new-confirm" onclick={confirmNewFile} title="Create">↵</button>
              </li>
            {:else}
              <li class="add-file-row">
                <button class="add-file-btn" onclick={startNewFile}>+ New File</button>
              </li>
            {/if}
          </ul>
        {/if}

        <!-- Drag overlay hint -->
        {#if isDragOver}
          <div class="drag-overlay">
            <span>Drop to add to Content</span>
          </div>
        {/if}
      </div>
  {/if}

</aside>

<!-- ── Playground context menu ── -->
{#if contextMenu}
  <div
    class="context-menu"
    role="menu"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === 'Escape' && closeContext()}
  >
    {#if readOnly}
      <button onclick={() => { dispatch('copyToProject', contextMenu!.name); contextMenu = null }}>Copy to Project…</button>
    {:else}
      <button onclick={() => startRename(contextMenu!.name)}>Rename</button>
      <button onclick={() => { dispatch('duplicate', contextMenu!.name); contextMenu = null }}>Duplicate</button>
      <hr />
      <button class="danger" onclick={() => { dispatch('delete', contextMenu!.name); contextMenu = null }}>Delete</button>
    {/if}
  </div>
{/if}

<!-- ── File context menu ── -->
{#if fileContextMenu}
  <div
    class="context-menu"
    role="menu"
    style="left: {fileContextMenu.x}px; top: {fileContextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === 'Escape' && closeContext()}
  >
    <button onclick={() => startRenameFile(fileContextMenu!.filename)}>Rename</button>
    <button onclick={() => revealFile(fileContextMenu!.filename)}>Reveal in Finder</button>
    <hr />
    <button class="danger" onclick={() => deleteFile(fileContextMenu!.filename)}>Delete</button>
  </div>
{/if}

<style>
  .sidebar {
    width: 100%;
    flex-shrink: 0;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Tab strip ── */
  .tab-strip {
    display: flex;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .strip-tab {
    flex: 1;
    padding: 8px 4px 7px;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-tertiary);
    border-bottom: 2px solid transparent;
    transition: color 0.12s, border-color 0.12s;
    text-align: center;
    letter-spacing: 0.01em;
  }
  .strip-tab:hover { color: var(--text-secondary); }
  .strip-tab.active {
    color: var(--text);
    border-bottom-color: var(--accent);
  }

  /* ── Shared header ── */
  .sidebar-header, .content-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px 8px;
    flex-shrink: 0;
  }

  .content-title-row {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }

  .sidebar-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: -0.01em;
    white-space: nowrap;
  }

  .content-for {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .icon-btn {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: var(--radius-xs);
    color: var(--text-tertiary);
    transition: background 0.12s, color 0.12s;
    flex-shrink: 0;
  }
  .icon-btn:hover { background: var(--bg-hover); color: var(--text); }

  /* ── Search ── */
  .search-wrap {
    display: flex; align-items: center;
    margin: 0 8px 8px;
    background: rgba(255,255,255,0.08);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 5px 8px; gap: 6px;
    flex-shrink: 0;
  }
  .search-icon { color: var(--text-tertiary); flex-shrink: 0; }
  .search-input {
    flex: 1; background: none; border: none; border-radius: 0;
    padding: 0; font-size: 12px; color: var(--text); outline: none;
    -webkit-appearance: none; min-width: 0;
  }
  .search-input::-webkit-search-cancel-button { display: none; }
  .search-input::placeholder { color: var(--text-tertiary); }
  .search-clear {
    flex-shrink: 0; color: var(--text-tertiary); font-size: 14px;
    line-height: 1; padding: 0; width: 14px; height: 14px;
    display: flex; align-items: center; justify-content: center;
  }
  .search-clear:hover { color: var(--text); }

  /* ── Playground list ── */
  .playground-list {
    list-style: none;
    overflow-y: auto;
    flex: 1;
    padding: 2px 6px 4px;
    min-height: 0;
  }

  .new-item {
    display: flex; align-items: center; gap: 8px;
    padding: 4px 8px;
    border-radius: 8px;
    background: rgba(10, 132, 255, 0.12);
    border: 1px solid rgba(10, 132, 255, 0.3);
    margin-bottom: 2px;
  }

  .new-input-wrap { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .new-input {
    flex: 1; font-size: 12px; font-family: var(--font-mono);
    padding: 2px 4px; border-radius: var(--radius-xs);
    background: rgba(255,255,255,0.08); border: 1px solid var(--border-strong);
    color: var(--text); min-width: 0;
  }
  .new-input:focus { outline: none; border-color: var(--accent); }
  .new-error { font-size: 10px; color: var(--red); line-height: 1.3; }
  .new-confirm {
    flex-shrink: 0; width: 20px; height: 20px;
    display: flex; align-items: center; justify-content: center;
    border-radius: var(--radius-xs);
    background: var(--accent); color: #fff;
    font-size: 13px; line-height: 1;
  }
  .new-confirm:hover { background: var(--accent-hover); }

  .playground-item {
    display: flex; align-items: center; gap: 8px;
    padding: 6px 8px; border-radius: 8px;
    cursor: pointer; user-select: none;
    transition: background 0.1s;
  }
  .playground-item:hover:not(.active) { background: var(--bg-hover); }
  .playground-item.active { background: var(--accent); }
  .playground-item.active .name { color: #fff; }
  .playground-item.active .file-icon { background: rgba(255,255,255,0.25); color: #fff; }
  .playground-item.active .dirty-dot { color: rgba(255,255,255,0.7); }

  .file-icon-logo {
    flex-shrink: 0;
    display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px;
  }
  .file-icon {
    font-size: 7px; font-weight: 800; letter-spacing: 0.03em;
    background: var(--rust-orange); color: #fff;
    border-radius: 3px; padding: 2px 3px;
    flex-shrink: 0; line-height: 1.3;
    min-width: 18px; text-align: center;
  }
  .file-icon--toml {
    background: #6b7280; letter-spacing: 0; font-size: 6px;
  }

  .lock-icon { flex-shrink: 0; color: var(--text-tertiary); }

  .name {
    flex: 1; font-size: 13px;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    color: var(--text-secondary);
  }

  .dirty-dot { color: var(--accent); font-size: 8px; flex-shrink: 0; }
  .empty-hint { padding: 8px; color: var(--text-tertiary); font-size: 12px; font-style: italic; }
  .rename-input { flex: 1; font-size: 13px; padding: 2px 6px; border-radius: var(--radius-xs); }

  /* ── Cargo.toml panel ── */
  .cargo-section {
    flex-shrink: 0; border-top: 1px solid var(--border);
    display: flex; flex-direction: column; max-height: 200px;
  }
  .cargo-header {
    display: flex; align-items: center; gap: 6px;
    padding: 7px 10px; cursor: pointer; background: none; width: 100%;
    color: var(--text-secondary); font-size: 11px; font-weight: 600;
    transition: background 0.1s; flex-shrink: 0;
  }
  .cargo-header:hover { background: var(--bg-hover); }
  .chevron { color: var(--text-tertiary); transition: transform 0.15s; flex-shrink: 0; }
  .chevron.open { transform: rotate(0deg); }
  .chevron:not(.open) { transform: rotate(-90deg); }
  .cargo-label { flex: 1; text-align: left; letter-spacing: -0.01em; }
  .cargo-edit-btn {
    font-size: 10px; font-weight: 500; color: var(--accent);
    padding: 1px 6px; border: 1px solid rgba(10,132,255,0.3);
    border-radius: var(--radius-xs); transition: background 0.1s;
  }
  .cargo-edit-btn:hover { background: rgba(10,132,255,0.15); }
  .cargo-body { overflow-y: auto; flex: 1; min-height: 0; }
  .cargo-pre {
    margin: 0; padding: 8px 10px;
    font-family: var(--font-mono); font-size: 11px; line-height: 1.6;
    color: var(--text-secondary); white-space: pre-wrap; word-break: break-word;
  }
  .cargo-empty { display: block; padding: 8px 10px; font-size: 11px; color: var(--text-tertiary); font-style: italic; }

  /* ── Compiler flags panel ── */
  .flags-body {
    padding: 8px 10px;
    display: flex; flex-direction: column; gap: 4px;
  }
  .flags-label {
    font-size: 10px; font-weight: 600; color: var(--text-tertiary);
    letter-spacing: 0.03em; text-transform: uppercase;
  }
  .flags-input {
    font-family: var(--font-mono); font-size: 11px;
    padding: 4px 8px;
    background: rgba(0,0,0,0.2); color: var(--text);
    border: 1px solid var(--border); border-radius: var(--radius-xs);
    outline: none; width: 100%; box-sizing: border-box;
  }
  .flags-input:focus { border-color: var(--accent); }
  .flags-input::placeholder { color: var(--text-tertiary); opacity: 0.5; }
  .flags-hint {
    font-size: 9px; color: var(--text-tertiary);
    margin-top: 2px;
  }

  /* ── Content tab ── */
  .content-empty-state {
    flex: 1;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 6px; padding: 24px 16px; text-align: center;
  }
  .empty-title { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
  .empty-sub { font-size: 11px; color: var(--text-tertiary); }

  .content-pane {
    flex: 1; display: flex; flex-direction: column;
    overflow: hidden; position: relative;
    transition: outline 0.1s;
  }
  .content-pane.drag-over {
    outline: 2px dashed var(--accent);
    outline-offset: -4px;
  }

  .content-loading {
    padding: 16px; font-size: 12px; color: var(--text-tertiary); font-style: italic;
  }

  .content-empty-drop {
    flex: 1; display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    gap: 8px; padding: 24px 16px; text-align: center;
  }
  .drop-icon { margin-bottom: 4px; color: var(--text-tertiary); }
  .drop-label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
  .drop-sub { font-size: 11px; color: var(--text-tertiary); }
  .new-file-btn {
    font-size: 12px; font-weight: 500; color: var(--accent);
    border: 1px solid rgba(10,132,255,0.3);
    border-radius: var(--radius-sm); padding: 5px 14px;
    transition: background 0.1s;
  }
  .new-file-btn:hover { background: rgba(10,132,255,0.12); }

  /* ── File list ── */
  .file-list {
    list-style: none;
    overflow-y: auto;
    flex: 1; min-height: 0;
    padding: 2px 6px 4px;
  }

  .file-item {
    display: flex; align-items: center; gap: 7px;
    padding: 6px 8px; border-radius: 7px;
    cursor: pointer; user-select: none;
    transition: background 0.1s;
  }
  .file-item:hover { background: var(--bg-hover); }

  .file-type-icon { font-size: 13px; flex-shrink: 0; line-height: 1; }
  .file-name {
    flex: 1; font-size: 13px; color: var(--text-secondary);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }

  .rename-file-input {
    flex: 1; font-size: 12px; font-family: var(--font-mono);
    padding: 1px 4px; border-radius: var(--radius-xs);
    background: rgba(255,255,255,0.08); border: 1px solid var(--accent);
    color: var(--text); min-width: 0; outline: none;
  }

  /* ── New file inline ── */
  .new-file-item {
    display: flex; align-items: center; gap: 7px;
    padding: 4px 8px; border-radius: 7px;
    background: rgba(10,132,255,0.1);
    border: 1px solid rgba(10,132,255,0.25);
    margin-top: 2px;
  }

  .new-file-wrap { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .new-file-input {
    flex: 1; font-size: 12px; font-family: var(--font-mono);
    padding: 2px 4px; border-radius: var(--radius-xs);
    background: rgba(255,255,255,0.08); border: 1px solid var(--border-strong);
    color: var(--text); min-width: 0; outline: none;
  }
  .new-file-input:focus { border-color: var(--accent); }
  .new-file-error { font-size: 10px; color: var(--red); line-height: 1.3; }

  .add-file-row {
    padding: 4px 8px; margin-top: 2px;
  }
  .add-file-btn {
    width: 100%; font-size: 11px; font-weight: 500;
    color: var(--text-tertiary); padding: 5px 8px;
    border: 1px dashed var(--border-strong); border-radius: 7px;
    text-align: center; transition: color 0.1s, border-color 0.1s, background 0.1s;
  }
  .add-file-btn:hover { color: var(--accent); border-color: rgba(10,132,255,0.4); background: rgba(10,132,255,0.06); }

  /* ── Drag overlay ── */
  .drag-overlay {
    position: absolute; inset: 0;
    display: flex; align-items: center; justify-content: center;
    background: rgba(10,132,255,0.12);
    pointer-events: none;
  }
  .drag-overlay span {
    font-size: 13px; font-weight: 500;
    color: var(--accent); background: var(--bg-elevated);
    border: 1px solid rgba(10,132,255,0.4);
    border-radius: var(--radius-sm); padding: 8px 16px;
  }

  .hidden { display: none; }

  /* ── Context menus ── */
  .context-menu {
    position: fixed; z-index: 1000;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius); padding: 4px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.5), 0 2px 8px rgba(0,0,0,0.3);
    min-width: 150px;
  }
  .context-menu button {
    display: block; width: 100%; text-align: left;
    padding: 6px 10px; font-size: 13px;
    border-radius: var(--radius-xs); color: var(--text);
  }
  .context-menu button:hover { background: var(--accent); color: #fff; }
  .context-menu button.danger { color: var(--red); }
  .context-menu button.danger:hover { background: var(--red); color: #fff; }
  .context-menu hr { border: none; border-top: 1px solid var(--border); margin: 4px 0; }
</style>
