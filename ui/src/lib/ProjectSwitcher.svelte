<script lang="ts">
  import { tick } from 'svelte'
  import { getLang, allLanguages, type ProjectType } from './languages'
  import type { EditionConfig } from './editions'

  // ── Props — callback-based (Svelte 5 idiom, lets us await and catch errors) ──
  let {
    projects,
    active,
    projectType = 'rust',
    projectTypes = {},
    projectSources = {},
    projectReadonly = {},
    enabledLanguages = ['rust'],
    onswitch,
    onnew,
    onrename,
    ondelete,
    onduplicate,
    onloadbook,
    edition,
    pendingMode = $bindable(null as 'new' | 'rename' | 'delete-confirm' | null),
  }: {
    projects: string[]
    active: string
    projectType?: ProjectType
    projectTypes?: Record<string, ProjectType>
    projectSources?: Record<string, string>
    projectReadonly?: Record<string, boolean>
    enabledLanguages?: string[]
    onswitch: (name: string) => Promise<void>
    onnew:    (name: string, ptype: ProjectType) => Promise<void>
    onrename: (oldName: string, newName: string) => Promise<void>
    ondelete: (name: string) => Promise<void>
    onduplicate: (name: string) => Promise<void>
    onloadbook: (ptype: ProjectType) => Promise<void>
    edition: EditionConfig
    pendingMode?: 'new' | 'rename' | 'delete-confirm' | null
  } = $props()

  let activeLang = $derived(getLang(projectType))

  // Open the popover in the requested mode when driven from outside (e.g. menu bar).
  $effect(() => {
    if (pendingMode) {
      open = true
      mode = pendingMode
      pendingMode = null
    }
  })

  let open    = $state(false)
  let mode: 'list' | 'new' | 'rename' | 'delete-confirm' = $state('list')
  let busy    = $state(false)    // disable buttons while async op is in-flight
  let opError = $state('')       // surface backend errors inline

  // Search filter
  let filterText = $state('')
  let filterEl: HTMLInputElement | null = $state(null)

  // Collapsible book sections — collapsed by default
  let expandedSections: Record<string, boolean> = $state({})

  // All books in stable display order, filtered by enabled languages
  let ALL_BOOKS = $derived(
    allLanguages()
      .filter(l => enabledLanguages.includes(l.type) && l.book)
      .map(l => ({ key: l.book!.sourceTag, label: l.book!.commandLabel, langType: l.type }))
  )

  // Group projects: user projects + book sections
  const grouped = $derived.by(() => {
    const lower = filterText.toLowerCase()
    const filtered = lower
      ? projects.filter(p => p.toLowerCase().includes(lower))
      : projects

    const userProjects: string[] = []
    const bookProjects: Record<string, string[]> = {}

    for (const name of filtered) {
      const source = projectSources[name]
      if (source && ALL_BOOKS.some(b => b.key === source)) {
        if (!bookProjects[source]) bookProjects[source] = []
        bookProjects[source].push(name)
      } else {
        userProjects.push(name)
      }
    }

    const bookSections = ALL_BOOKS.map(b => ({
      ...b,
      projects: bookProjects[b.key] ?? [],
      loaded: !!bookProjects[b.key]?.length,
    }))

    return { userProjects, bookSections }
  })

  // New project input
  let newValue    = $state('')
  let newError    = $state('')
  let newInputEl: HTMLInputElement | null = $state(null)
  let newProjectType: ProjectType = $state('rust')
  let newLang = $derived(getLang(newProjectType))

  // Rename input
  let renameValue   = $state('')
  let renameError   = $state('')
  let renameInputEl: HTMLInputElement | null = $state(null)

  $effect(() => {
    if (mode === 'new') {
      newValue = ''; newError = ''; opError = ''
      tick().then(() => newInputEl?.focus())
    }
    if (mode === 'rename') {
      renameValue = active; renameError = ''; opError = ''
      tick().then(() => { renameInputEl?.focus(); renameInputEl?.select() })
    }
  })

  function toggle() { open = !open; if (!open) { mode = 'list'; opError = ''; filterText = '' } else { tick().then(() => filterEl?.focus()) } }
  function close()  { open = false; mode = 'list'; opError = ''; busy = false; filterText = '' }

  async function selectProject(name: string) {
    if (name === active) { close(); return }
    busy = true
    opError = ''
    try { await onswitch(name) } catch (err) { opError = String(err); busy = false; return }
    close()
  }

  // ── New project ───────────────────────────────────────────────────────────────
  async function confirmNew() {
    const name = newValue.trim().toLowerCase()
    console.log('[ProjectSwitcher] confirmNew', name)
    if (!name) { mode = 'list'; return }
    if (!/^[a-z][a-z0-9_]*$/.test(name)) { newError = 'Lowercase letters, digits, underscores only'; return }
    if (name.length > 64)                 { newError = 'Max 64 characters'; return }
    if (projects.includes(name))          { newError = 'Already exists'; return }
    busy = true; opError = ''
    try    { await onnew(name, newProjectType); console.log('[ProjectSwitcher] onnew ok'); close() }
    catch  (err) { console.error('[ProjectSwitcher] onnew failed', err); opError = String(err); busy = false }
  }

  function handleNewKey(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); confirmNew() }
    if (e.key === 'Escape') { mode = 'list'; opError = '' }
  }

  // ── Rename project ────────────────────────────────────────────────────────────
  async function confirmRename() {
    const name = renameValue.trim().toLowerCase()
    console.log('[ProjectSwitcher] confirmRename', active, '->', name)
    if (!name || name === active) { mode = 'list'; return }
    if (!/^[a-z][a-z0-9_]*$/.test(name)) { renameError = 'Lowercase letters, digits, underscores only'; return }
    if (name.length > 64)                 { renameError = 'Max 64 characters'; return }
    if (projects.includes(name))          { renameError = 'Already exists'; return }
    busy = true; opError = ''
    try    { await onrename(active, name); console.log('[ProjectSwitcher] onrename ok'); close() }
    catch  (err) { console.error('[ProjectSwitcher] onrename failed', err); opError = String(err); busy = false }
  }

  function handleRenameKey(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); confirmRename() }
    if (e.key === 'Escape') { mode = 'list'; opError = '' }
  }

  // ── Duplicate project ─────────────────────────────────────────────────────────
  async function duplicateProject() {
    busy = true; opError = ''
    try { await onduplicate(active); close() }
    catch (err) { opError = String(err); busy = false }
  }

  // ── Delete project ────────────────────────────────────────────────────────────
  async function confirmDelete() {
    busy = true; opError = ''
    try    { await ondelete(active); close() }
    catch  (err) { opError = String(err); busy = false }
  }
</script>

<div class="project-switcher">
  <button class="pill" onclick={toggle} class:open>
    {#if !edition.isSingleLanguage}
      <span class="pill-badge" class:clang={projectType === 'clang'} class:zig={projectType === 'zig'} class:swift={projectType === 'swift'}>{activeLang.badge}</span>
    {/if}
    <span class="pill-name">{active}</span>
    {#if activeLang.experimental && !edition.isSingleLanguage}<span class="exp-tag">exp</span>{/if}
    <svg class="pill-caret" width="10" height="6" viewBox="0 0 10 6">
      <path d="M1 1l4 4 4-4" stroke="currentColor" stroke-width="1.5"
            stroke-linecap="round" fill="none"/>
    </svg>
  </button>

  {#if open}
    <!-- Invisible backdrop — click outside the popover to close -->
    <div class="backdrop" onclick={close} aria-hidden="true"></div>
    <div class="popover">

      {#if mode === 'list'}
        <!-- Search filter -->
        <div class="filter-row">
          <input
            class="filter-input"
            type="text"
            placeholder="Filter projects…"
            bind:value={filterText}
            bind:this={filterEl}
            onkeydown={(e) => { if (e.key === 'Escape') { if (filterText) { filterText = '' } else { close() } } }}
          />
        </div>

        <ul class="project-list">
          <!-- User projects -->
          {#each grouped.userProjects as name (name)}
            {@const itemPtype = projectTypes[name] ?? 'rust'}
            {@const itemLang = getLang(itemPtype)}
            <li>
              <button
                class="project-item"
                class:active-project={name === active}
                onclick={() => selectProject(name)}
                disabled={busy}
              >
                {#if !edition.isSingleLanguage}
                  <span class="item-badge" class:clang={itemPtype === 'clang'} class:zig={itemPtype === 'zig'} class:swift={itemPtype === 'swift'}>{itemLang.badge}</span>
                {/if}
                <span class="project-name">{name}</span>
                {#if itemLang.experimental && !edition.isSingleLanguage}<span class="exp-tag">exp</span>{/if}
                {#if name === active}
                  <svg class="active-check" width="10" height="8" viewBox="0 0 10 8" fill="none">
                    <path d="M1 4l3 3 5-6" stroke="currentColor" stroke-width="1.6"
                          stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                {/if}
              </button>
            </li>
          {/each}

        </ul>
        {#if opError}<p class="op-error">{opError}</p>{/if}
        <div class="divider"></div>
        <button class="menu-action" onclick={() => mode = 'new'}    disabled={busy}>New Project…</button>
        <button class="menu-action" onclick={duplicateProject} disabled={busy}>Duplicate Project</button>
        {#if !projectReadonly[active]}
          <div class="divider"></div>
          <button class="menu-action" onclick={() => mode = 'rename'} disabled={busy}>Rename Project…</button>
          <button
            class="menu-action danger"
            onclick={() => mode = 'delete-confirm'}
            disabled={busy || projects.length <= 1}
          >Delete Project…</button>
        {/if}

        <!-- Books section — only shown if at least one book is loaded -->
        {#if grouped.bookSections.some(s => s.loaded)}
          <div class="divider"></div>
          <div class="books-header">Books</div>
          <ul class="project-list books-list">
            {#each grouped.bookSections as section (section.key)}
              {#if section.loaded}
                <li class="submenu-parent">
                  <button class="project-item submenu-trigger">
                    <span class="section-book-icon">📖</span>
                    <span class="project-name">{section.label}</span>
                    <span class="section-count">{section.projects.length}</span>
                    <svg class="submenu-arrow" width="6" height="10" viewBox="0 0 6 10">
                      <path d="M1 1l4 4-4 4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" fill="none"/>
                    </svg>
                  </button>
                  <div class="submenu">
                    <ul class="project-list">
                      {#each section.projects as name (name)}
                        {@const itemPtype = projectTypes[name] ?? 'rust'}
                        {@const itemLang = getLang(itemPtype)}
                        <li>
                          <button
                            class="project-item"
                            class:active-project={name === active}
                            onclick={() => selectProject(name)}
                            disabled={busy}
                          >
                            {#if !edition.isSingleLanguage}
                              <span class="item-badge" class:clang={itemPtype === 'clang'} class:zig={itemPtype === 'zig'} class:swift={itemPtype === 'swift'}>{itemLang.badge}</span>
                            {/if}
                            <span class="project-name">{name}</span>
                            {#if name === active}
                              <svg class="active-check" width="10" height="8" viewBox="0 0 10 8" fill="none">
                                <path d="M1 4l3 3 5-6" stroke="currentColor" stroke-width="1.6"
                                      stroke-linecap="round" stroke-linejoin="round"/>
                              </svg>
                            {/if}
                          </button>
                        </li>
                      {/each}
                    </ul>
                  </div>
                </li>
              {/if}
            {/each}
          </ul>
        {/if}

      {:else if mode === 'new'}
        <div class="inline-input-section">
          <p class="inline-label">
            {#if !edition.isSingleLanguage}
              <span class="form-badge" class:clang={newProjectType === 'clang'} class:zig={newProjectType === 'zig'} class:swift={newProjectType === 'swift'}>{newLang.badge}</span>
            {/if}
            New Project
          </p>
          {#if !edition.isSingleLanguage}
          <div class="type-toggle">
            {#each allLanguages().filter(l => enabledLanguages.includes(l.type)) as l (l.type)}
              <button
                class="type-btn" class:active={newProjectType === l.type}
                onclick={() => newProjectType = l.type}
              >{l.label}</button>
            {/each}
          </div>
          <span class="type-hint">
            {newProjectType === 'rust' ? 'Cargo workspace with deps and live checking'
              : newProjectType === 'clang' ? 'C/C++ with clang — compiler flags in rustic.toml'
              : newProjectType === 'zig' ? 'Zig playground (experimental) — zig run with flags in rustic.toml'
              : 'Swift playground — swiftc compile & run'}
          </span>
          {/if}
          <input
            class="inline-input"
            class:error={!!newError}
            type="text"
            placeholder="project_name"
            bind:value={newValue}
            bind:this={newInputEl}
            onkeydown={handleNewKey}
            onclick={(e) => e.stopPropagation()}
            disabled={busy}
          />
          {#if newError} <span class="inline-error">{newError}</span>{/if}
          {#if opError}  <span class="inline-error">{opError}</span>{/if}
          <div class="inline-actions">
            <button class="btn-cancel"  onclick={() => { mode = 'list'; opError = '' }} disabled={busy}>Cancel</button>
            <button class="btn-confirm" onclick={confirmNew} disabled={busy}>
              {busy ? 'Creating…' : 'Create'}
            </button>
          </div>
        </div>

      {:else if mode === 'rename'}
        <div class="inline-input-section">
          <p class="inline-label">
            <span class="form-badge" class:clang={projectType === 'clang'} class:zig={projectType === 'zig'} class:swift={projectType === 'swift'}>{activeLang.badge}</span>
            Rename "{active}"
          </p>
          <input
            class="inline-input"
            class:error={!!renameError}
            type="text"
            bind:value={renameValue}
            bind:this={renameInputEl}
            onkeydown={handleRenameKey}
            onclick={(e) => e.stopPropagation()}
            disabled={busy}
          />
          {#if renameError}<span class="inline-error">{renameError}</span>{/if}
          {#if opError}    <span class="inline-error">{opError}</span>{/if}
          <div class="inline-actions">
            <button class="btn-cancel"  onclick={() => { mode = 'list'; opError = '' }} disabled={busy}>Cancel</button>
            <button class="btn-confirm" onclick={confirmRename} disabled={busy}>
              {busy ? 'Renaming…' : 'Rename'}
            </button>
          </div>
        </div>

      {:else if mode === 'delete-confirm'}
        <div class="inline-input-section">
          <p class="inline-label">Delete "{active}"?</p>
          <p class="delete-warn">
            This will permanently delete all playgrounds and content files in this project.
          </p>
          {#if opError}<span class="inline-error">{opError}</span>{/if}
          <div class="inline-actions">
            <button class="btn-cancel" onclick={() => { mode = 'list'; opError = '' }} disabled={busy}>Cancel</button>
            <button class="btn-danger" onclick={confirmDelete} disabled={busy}>
              {busy ? 'Deleting…' : 'Delete'}
            </button>
          </div>
        </div>
      {/if}

    </div>
  {/if}
</div>

<style>
  .project-switcher { position: relative; }

  /* Full-screen transparent backdrop — sits behind the popover, closes on click */
  .backdrop {
    position: fixed; inset: 0; z-index: 199;
  }

  /* ── Pill button ── */
  .pill {
    display: flex; align-items: center; gap: 6px;
    padding: 4px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 13px; font-weight: 600; color: var(--text);
    transition: background 0.1s, border-color 0.1s;
    min-width: 240px; max-width: 320px;
  }
  .pill:hover, .pill.open { background: var(--bg-hover); border-color: var(--border-strong); }
  .pill-badge {
    font-size: 7.5px; font-weight: 800; letter-spacing: 0.04em;
    background: var(--rust-orange); color: #fff;
    border-radius: 3px; padding: 2px 4px;
    line-height: 1.3; flex-shrink: 0;
  }
  .pill-badge.clang {
    background: #4a9; font-size: 7px;
  }
  .pill-badge.zig {
    background: #f7a41d; font-size: 6.5px;
  }
  .pill-badge.swift {
    background: #f05138; font-size: 7px;
  }
  .pill-name { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .exp-tag {
    font-size: 7px; font-weight: 700; letter-spacing: 0.03em;
    text-transform: uppercase;
    background: rgba(247, 164, 29, 0.15);
    color: #f7a41d;
    border: 1px solid rgba(247, 164, 29, 0.3);
    border-radius: 3px;
    padding: 0.5px 3px;
    line-height: 1.3;
    flex-shrink: 0;
  }
  .pill-caret { flex-shrink: 0; color: var(--text-tertiary); transition: transform 0.15s; }
  .pill.open .pill-caret { transform: rotate(180deg); }

  /* ── Popover ── */
  .popover {
    position: absolute; top: calc(100% + 6px); left: 0;
    min-width: 240px; max-width: 320px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    box-shadow: 0 8px 32px rgba(0,0,0,0.5), 0 2px 8px rgba(0,0,0,0.3);
    padding: 4px; z-index: 200;
    overflow: visible;
  }

  /* ── Filter ── */
  .filter-row { padding: 4px 4px 2px; }
  .filter-input {
    width: 100%; padding: 5px 8px;
    font-size: 12px; font-family: var(--font-mono);
    background: rgba(255,255,255,0.06); color: var(--text);
    border: 1px solid var(--border); border-radius: var(--radius-xs);
    outline: none;
  }
  .filter-input:focus { border-color: var(--accent); }
  .filter-input::placeholder { color: var(--text-tertiary); }

  /* ── Project list ── */
  .project-list { list-style: none; max-height: 280px; overflow-y: auto; margin-bottom: 2px; }

  .project-item {
    display: flex; align-items: center; gap: 8px;
    width: 100%; padding: 6px 10px;
    border-radius: var(--radius-xs);
    font-size: 13px; color: var(--text); text-align: left;
    transition: background 0.1s;
  }
  .project-item:hover:not(:disabled) { background: var(--bg-hover); }
  .project-item.active-project        { background: rgba(10,132,255,0.12); }
  .project-item:disabled              { opacity: 0.5; cursor: not-allowed; }

  /* RS badge in list rows */
  .item-badge {
    font-size: 7px; font-weight: 800; letter-spacing: 0.04em;
    background: var(--rust-orange); color: #fff;
    border-radius: 3px; padding: 2px 3px;
    line-height: 1.3; flex-shrink: 0;
    min-width: 16px; text-align: center;
  }
  .item-badge.clang {
    background: #4a9; font-size: 6px;
  }
  .item-badge.zig {
    background: #f7a41d; font-size: 6px;
  }
  .item-badge.swift {
    background: #f05138; font-size: 6.5px;
  }
  .active-project .item-badge { background: var(--rust-orange); }
  .active-project .item-badge.clang { background: #4a9; }
  .active-project .item-badge.zig { background: #f7a41d; }
  .active-project .item-badge.swift { background: #f05138; }

  /* Unloaded book row */
  .load-book-item { color: var(--text-tertiary); }
  .load-book-item:hover:not(:disabled) { color: var(--text); }
  .load-tag {
    font-size: 9px; font-weight: 600; letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--accent);
    background: rgba(10, 132, 255, 0.1);
    border-radius: 3px; padding: 1px 5px;
    flex-shrink: 0;
  }

  /* Books section — overflow visible so submenus can fly out */
  .books-list { list-style: none; overflow: visible; }
  .books-header {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-tertiary);
    padding: 2px 10px 4px;
    list-style: none;
  }

  /* Submenu flyout (macOS-style) */
  .submenu-parent { position: relative; list-style: none; }
  .submenu-trigger { cursor: default; }
  .submenu-arrow { flex-shrink: 0; color: var(--text-tertiary); margin-left: auto; }
  .section-book-icon { font-size: 11px; flex-shrink: 0; }
  .section-count {
    font-size: 9px; font-weight: 600;
    color: var(--text-tertiary);
    background: rgba(255,255,255,0.06);
    border-radius: 3px; padding: 1px 4px;
  }
  .submenu {
    display: none;
    position: absolute;
    left: 100%; top: -4px;
    min-width: 220px; max-width: 300px;
    max-height: 320px; overflow-y: auto;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    box-shadow: 0 8px 32px rgba(0,0,0,0.5), 0 2px 8px rgba(0,0,0,0.3);
    padding: 4px;
    z-index: 210;
  }
  /* Invisible bridge so the mouse can travel from trigger to submenu */
  .submenu::before {
    content: '';
    position: absolute;
    top: 0; bottom: 0;
    left: -12px; width: 12px;
  }
  .submenu .project-list { list-style: none; max-height: none; }
  .submenu-parent:hover > .submenu { display: block; }
  .submenu-parent:hover > .submenu-trigger { background: var(--bg-hover); }

  /* Active checkmark */
  .active-check { flex-shrink: 0; color: var(--accent); }

  /* RS / Clang badge in form label rows */
  .form-badge {
    display: inline-block;
    font-size: 7px; font-weight: 800; letter-spacing: 0.04em;
    background: var(--rust-orange); color: #fff;
    border-radius: 3px; padding: 2px 3px;
    line-height: 1.3;
    vertical-align: middle;
    margin-right: 2px;
  }
  .form-badge.clang {
    background: #4a9; font-size: 6px;
  }
  .form-badge.zig {
    background: #f7a41d; font-size: 6px;
  }
  .form-badge.swift {
    background: #f05138; font-size: 6.5px;
  }

  /* Type toggle for new project */
  .type-toggle {
    display: flex; gap: 0; margin-bottom: 6px;
    border: 1px solid var(--border); border-radius: 5px; overflow: hidden;
  }
  .type-btn {
    flex: 1; padding: 4px 0; font-size: 11px; font-weight: 600;
    text-align: center; cursor: pointer;
    background: transparent; color: var(--text-secondary);
    border: none; transition: background 0.1s, color 0.1s;
  }
  .type-btn { border-right: 1px solid var(--border); }
  .type-btn:last-child { border-right: none; }
  .type-btn.active {
    background: var(--bg-hover); color: var(--text);
  }
  .type-hint {
    display: block; font-size: 10px; color: var(--text-tertiary);
    margin-bottom: 6px; line-height: 1.4;
  }

  .project-name {
    flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    font-family: var(--font-mono); font-size: 12px;
  }

  /* ── Divider + menu actions ── */
  .divider { border-top: 1px solid var(--border); margin: 4px 0; }

  .menu-action {
    display: block; width: 100%; padding: 6px 10px;
    border-radius: var(--radius-xs);
    font-size: 13px; color: var(--text); text-align: left;
    transition: background 0.1s;
  }
  .menu-action:hover:not(:disabled) { background: var(--bg-hover); }
  .menu-action.danger               { color: var(--red); }
  .menu-action.danger:hover:not(:disabled) { background: var(--red); color: #fff; }
  .menu-action:disabled             { opacity: 0.35; cursor: not-allowed; }

  /* ── Op-level error (shown in list mode) ── */
  .op-error {
    font-size: 11px; color: var(--red); padding: 4px 10px 0;
    line-height: 1.4;
  }

  /* ── Inline input section ── */
  .inline-input-section {
    padding: 8px 8px 6px; display: flex; flex-direction: column; gap: 6px;
  }
  .inline-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); letter-spacing: 0.02em; }
  .inline-input {
    font-size: 12px; font-family: var(--font-mono); padding: 5px 8px;
    background: rgba(255,255,255,0.06); border: 1px solid var(--border-strong);
    border-radius: var(--radius-xs); color: var(--text); outline: none;
  }
  .inline-input:focus          { border-color: var(--accent); }
  .inline-input.error          { border-color: var(--red); }
  .inline-input:disabled       { opacity: 0.5; }
  .inline-error  { font-size: 10px; color: var(--red); line-height: 1.3; }
  .delete-warn   { font-size: 11px; color: var(--text-tertiary); line-height: 1.5; }

  .inline-actions { display: flex; justify-content: flex-end; gap: 6px; margin-top: 2px; }

  .btn-cancel {
    font-size: 12px; font-weight: 500; padding: 4px 10px;
    border-radius: var(--radius-xs); color: var(--text-secondary);
    background: rgba(255,255,255,0.06); border: 1px solid var(--border);
    transition: background 0.1s;
  }
  .btn-cancel:hover:not(:disabled) { background: var(--bg-hover); }
  .btn-cancel:disabled             { opacity: 0.4; cursor: not-allowed; }

  .btn-confirm {
    font-size: 12px; font-weight: 600; padding: 4px 10px;
    border-radius: var(--radius-xs); background: var(--accent); color: #fff;
    transition: background 0.1s;
  }
  .btn-confirm:hover:not(:disabled) { background: var(--accent-hover); }
  .btn-confirm:disabled             { opacity: 0.5; cursor: not-allowed; }

  .btn-danger {
    font-size: 12px; font-weight: 600; padding: 4px 10px;
    border-radius: var(--radius-xs); background: var(--red); color: #fff;
    transition: opacity 0.1s;
  }
  .btn-danger:hover:not(:disabled) { opacity: 0.85; }
  .btn-danger:disabled             { opacity: 0.4; cursor: not-allowed; }
</style>
