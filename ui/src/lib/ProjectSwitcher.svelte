<script lang="ts">
  import { tick } from 'svelte'

  // ── Props — callback-based (Svelte 5 idiom, lets us await and catch errors) ──
  let {
    projects,
    active,
    onswitch,
    onnew,
    onrename,
    ondelete,
    pendingMode = $bindable(null as 'new' | 'rename' | 'delete-confirm' | null),
  }: {
    projects: string[]
    active: string
    onswitch: (name: string) => Promise<void>
    onnew:    (name: string) => Promise<void>
    onrename: (oldName: string, newName: string) => Promise<void>
    ondelete: (name: string) => Promise<void>
    pendingMode?: 'new' | 'rename' | 'delete-confirm' | null
  } = $props()

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

  // New project input
  let newValue    = $state('')
  let newError    = $state('')
  let newInputEl: HTMLInputElement | null = $state(null)

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

  function toggle() { open = !open; if (!open) { mode = 'list'; opError = '' } }
  function close()  { open = false; mode = 'list'; opError = ''; busy = false }

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
    try    { await onnew(name); console.log('[ProjectSwitcher] onnew ok'); close() }
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

  // ── Delete project ────────────────────────────────────────────────────────────
  async function confirmDelete() {
    busy = true; opError = ''
    try    { await ondelete(active); close() }
    catch  (err) { opError = String(err); busy = false }
  }
</script>

<div class="project-switcher">
  <button class="pill" onclick={toggle} class:open>
    <span class="pill-name">{active}</span>
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
        <ul class="project-list">
          {#each projects as name (name)}
            <li>
              <button
                class="project-item"
                class:active-project={name === active}
                onclick={() => selectProject(name)}
                disabled={busy}
              >
                <span class="project-dot" class:filled={name === active}></span>
                <span class="project-name">{name}</span>
              </button>
            </li>
          {/each}
        </ul>
        {#if opError}<p class="op-error">{opError}</p>{/if}
        <div class="divider"></div>
        <button class="menu-action" onclick={() => mode = 'new'}    disabled={busy}>New Project…</button>
        <div class="divider"></div>
        <button class="menu-action" onclick={() => mode = 'rename'} disabled={busy}>Rename Project…</button>
        <button
          class="menu-action danger"
          onclick={() => mode = 'delete-confirm'}
          disabled={busy || projects.length <= 1}
        >Delete Project…</button>

      {:else if mode === 'new'}
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
          <p class="inline-label">Rename "{active}"</p>
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
    max-width: 180px;
  }
  .pill:hover, .pill.open { background: var(--bg-hover); border-color: var(--border-strong); }
  .pill-name { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 130px; }
  .pill-caret { flex-shrink: 0; color: var(--text-tertiary); transition: transform 0.15s; }
  .pill.open .pill-caret { transform: rotate(180deg); }

  /* ── Popover ── */
  .popover {
    position: absolute; top: calc(100% + 6px); left: 0;
    min-width: 200px; max-width: 260px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    box-shadow: 0 8px 32px rgba(0,0,0,0.5), 0 2px 8px rgba(0,0,0,0.3);
    padding: 4px; z-index: 200;
  }

  /* ── Project list ── */
  .project-list { list-style: none; max-height: 200px; overflow-y: auto; margin-bottom: 2px; }

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

  .project-dot {
    width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0;
    border: 1.5px solid var(--text-tertiary);
  }
  .project-dot.filled { background: var(--accent); border-color: var(--accent); }

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
