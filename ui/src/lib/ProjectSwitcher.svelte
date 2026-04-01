<script lang="ts">
  import { tick, createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  let {
    projects,
    active,
  }: {
    projects: string[]
    active: string
  } = $props()

  let open = $state(false)
  let mode: 'list' | 'new' | 'rename' | 'delete-confirm' = $state('list')

  // New project input
  let newValue = $state('')
  let newError = $state('')
  let newInputEl: HTMLInputElement | null = $state(null)

  // Rename input
  let renameValue = $state('')
  let renameError = $state('')
  let renameInputEl: HTMLInputElement | null = $state(null)

  $effect(() => {
    if (mode === 'new') {
      newValue = ''
      newError = ''
      tick().then(() => newInputEl?.focus())
    }
    if (mode === 'rename') {
      renameValue = active
      renameError = ''
      tick().then(() => { renameInputEl?.focus(); renameInputEl?.select() })
    }
  })

  function toggle() { open = !open; if (!open) mode = 'list' }
  function close()  { open = false; mode = 'list' }

  function selectProject(name: string) {
    if (name !== active) dispatch('switch', name)
    close()
  }

  // ── New project ───────────────────────────────────────────────────────────────
  function confirmNew() {
    const name = newValue.trim().toLowerCase()
    if (!name) { mode = 'list'; return }
    if (!/^[a-z][a-z0-9_]*$/.test(name)) {
      newError = 'Lowercase letters, digits, underscores only'
      return
    }
    if (name.length > 64) { newError = 'Max 64 characters'; return }
    if (projects.includes(name)) { newError = 'Already exists'; return }
    dispatch('new', name)
    close()
  }

  function handleNewKey(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); confirmNew() }
    if (e.key === 'Escape') { mode = 'list' }
  }

  // ── Rename project ────────────────────────────────────────────────────────────
  function confirmRename() {
    const name = renameValue.trim().toLowerCase()
    if (!name || name === active) { mode = 'list'; return }
    if (!/^[a-z][a-z0-9_]*$/.test(name)) {
      renameError = 'Lowercase letters, digits, underscores only'
      return
    }
    if (name.length > 64) { renameError = 'Max 64 characters'; return }
    if (projects.includes(name)) { renameError = 'Already exists'; return }
    dispatch('rename', { old: active, new: name })
    close()
  }

  function handleRenameKey(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); confirmRename() }
    if (e.key === 'Escape') { mode = 'list' }
  }

  // ── Delete project ────────────────────────────────────────────────────────────
  function confirmDelete() {
    dispatch('delete', active)
    close()
  }
</script>

<!-- Click-outside to close -->
<svelte:window onclick={(e) => {
  if (open && !(e.target as Element)?.closest('.project-switcher')) close()
}} />

<div class="project-switcher">
  <button class="pill" onclick={toggle} class:open>
    <span class="pill-name">{active}</span>
    <svg class="pill-caret" width="10" height="6" viewBox="0 0 10 6">
      <path d="M1 1l4 4 4-4" stroke="currentColor" stroke-width="1.5"
            stroke-linecap="round" fill="none"/>
    </svg>
  </button>

  {#if open}
    <div class="popover">

      {#if mode === 'list'}
        <!-- Project list -->
        <ul class="project-list">
          {#each projects as name (name)}
            <li>
              <button
                class="project-item"
                class:active-project={name === active}
                onclick={() => selectProject(name)}
              >
                <span class="project-dot" class:filled={name === active}></span>
                <span class="project-name">{name}</span>
              </button>
            </li>
          {/each}
        </ul>

        <div class="divider"></div>
        <button class="menu-action" onclick={() => mode = 'new'}>New Project…</button>
        <div class="divider"></div>
        <button class="menu-action" onclick={() => mode = 'rename'}>Rename Project…</button>
        <button
          class="menu-action danger"
          onclick={() => mode = 'delete-confirm'}
          disabled={projects.length <= 1}
        >Delete Project…</button>

      {:else if mode === 'new'}
        <!-- Inline new project input -->
        <div class="inline-input-section">
          <p class="inline-label">New Project</p>
          <input
            class="inline-input"
            class:error={!!newError}
            type="text"
            placeholder="project_name"
            bind:value={newValue}
            bind:this={newInputEl}
            onkeydown={handleNewKey}
            onclick={(e) => e.stopPropagation()}
          />
          {#if newError}<span class="inline-error">{newError}</span>{/if}
          <div class="inline-actions">
            <button class="btn-cancel" onclick={() => mode = 'list'}>Cancel</button>
            <button class="btn-confirm" onclick={confirmNew}>Create</button>
          </div>
        </div>

      {:else if mode === 'rename'}
        <!-- Inline rename input -->
        <div class="inline-input-section">
          <p class="inline-label">Rename "{active}"</p>
          <input
            class="inline-input"
            class:error={!!renameError}
            type="text"
            bind:value={renameValue}
            bind:this={renameInputEl}
            onkeydown={handleRenameKey}
            onclick={(e) => e.stopPropagation()}
          />
          {#if renameError}<span class="inline-error">{renameError}</span>{/if}
          <div class="inline-actions">
            <button class="btn-cancel" onclick={() => mode = 'list'}>Cancel</button>
            <button class="btn-confirm" onclick={confirmRename}>Rename</button>
          </div>
        </div>

      {:else if mode === 'delete-confirm'}
        <!-- Delete confirmation -->
        <div class="inline-input-section">
          <p class="inline-label">Delete "{active}"?</p>
          <p class="delete-warn">
            This will permanently delete all playgrounds and content files in this project.
          </p>
          <div class="inline-actions">
            <button class="btn-cancel" onclick={() => mode = 'list'}>Cancel</button>
            <button class="btn-danger" onclick={confirmDelete}>Delete</button>
          </div>
        </div>
      {/if}

    </div>
  {/if}
</div>

<style>
  .project-switcher {
    position: relative;
  }

  /* ── Pill button ── */
  .pill {
    display: flex; align-items: center; gap: 6px;
    padding: 4px 10px 4px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 13px; font-weight: 600;
    color: var(--text);
    transition: background 0.1s, border-color 0.1s;
    max-width: 180px;
  }
  .pill:hover, .pill.open {
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }
  .pill-name {
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    max-width: 130px;
  }
  .pill-caret {
    flex-shrink: 0; color: var(--text-tertiary);
    transition: transform 0.15s;
  }
  .pill.open .pill-caret { transform: rotate(180deg); }

  /* ── Popover ── */
  .popover {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    min-width: 200px;
    max-width: 260px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    box-shadow: 0 8px 32px rgba(0,0,0,0.5), 0 2px 8px rgba(0,0,0,0.3);
    padding: 4px;
    z-index: 200;
  }

  /* ── Project list ── */
  .project-list {
    list-style: none;
    max-height: 200px;
    overflow-y: auto;
    margin-bottom: 2px;
  }

  .project-item {
    display: flex; align-items: center; gap: 8px;
    width: 100%; padding: 6px 10px;
    border-radius: var(--radius-xs);
    font-size: 13px; color: var(--text);
    text-align: left;
    transition: background 0.1s;
  }
  .project-item:hover    { background: var(--bg-hover); }
  .project-item.active-project { background: rgba(10,132,255,0.12); }

  .project-dot {
    width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0;
    border: 1.5px solid var(--text-tertiary);
    transition: background 0.1s, border-color 0.1s;
  }
  .project-dot.filled {
    background: var(--accent);
    border-color: var(--accent);
  }

  .project-name {
    flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    font-family: var(--font-mono); font-size: 12px;
  }

  /* ── Divider ── */
  .divider { border-top: 1px solid var(--border); margin: 4px 0; }

  /* ── Menu actions ── */
  .menu-action {
    display: block; width: 100%;
    padding: 6px 10px; border-radius: var(--radius-xs);
    font-size: 13px; color: var(--text); text-align: left;
    transition: background 0.1s;
  }
  .menu-action:hover:not(:disabled) { background: var(--bg-hover); }
  .menu-action.danger               { color: var(--red); }
  .menu-action.danger:hover:not(:disabled) { background: var(--red); color: #fff; }
  .menu-action:disabled             { opacity: 0.35; cursor: not-allowed; }

  /* ── Inline input section (new / rename / delete) ── */
  .inline-input-section {
    padding: 8px 8px 6px;
    display: flex; flex-direction: column; gap: 6px;
  }

  .inline-label {
    font-size: 11px; font-weight: 600;
    color: var(--text-secondary); letter-spacing: 0.02em;
  }

  .inline-input {
    font-size: 12px; font-family: var(--font-mono);
    padding: 5px 8px;
    background: rgba(255,255,255,0.06);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-xs);
    color: var(--text); outline: none;
  }
  .inline-input:focus { border-color: var(--accent); }
  .inline-input.error { border-color: var(--red); }

  .inline-error { font-size: 10px; color: var(--red); line-height: 1.3; }

  .delete-warn {
    font-size: 11px; color: var(--text-tertiary); line-height: 1.5;
  }

  .inline-actions {
    display: flex; justify-content: flex-end; gap: 6px; margin-top: 2px;
  }

  .btn-cancel {
    font-size: 12px; font-weight: 500;
    padding: 4px 10px; border-radius: var(--radius-xs);
    color: var(--text-secondary);
    background: rgba(255,255,255,0.06);
    border: 1px solid var(--border);
    transition: background 0.1s;
  }
  .btn-cancel:hover { background: var(--bg-hover); }

  .btn-confirm {
    font-size: 12px; font-weight: 600;
    padding: 4px 10px; border-radius: var(--radius-xs);
    background: var(--accent); color: #fff;
    transition: background 0.1s;
  }
  .btn-confirm:hover { background: var(--accent-hover); }

  .btn-danger {
    font-size: 12px; font-weight: 600;
    padding: 4px 10px; border-radius: var(--radius-xs);
    background: var(--red); color: #fff;
    transition: opacity 0.1s;
  }
  .btn-danger:hover { opacity: 0.85; }
</style>
