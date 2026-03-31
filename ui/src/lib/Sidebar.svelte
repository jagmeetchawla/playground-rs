<script lang="ts">
  import { createEventDispatcher, tick } from 'svelte'

  const dispatch = createEventDispatcher()

  let {
    playgrounds,
    selected,
    dirtyTabs,
    creatingNew = $bindable(false),
    cargoToml = '',
  }: {
    playgrounds: string[]
    selected: string | null
    dirtyTabs: string[]
    creatingNew: boolean
    cargoToml: string
  } = $props()

  // ── Playground list ───────────────────────────────────────────────────────────
  let contextMenu: { x: number; y: number; name: string } | null = $state(null)
  let renamingName: string | null = $state(null)
  let renameValue: string = $state('')
  let searchQuery: string = $state('')

  let filtered = $derived(
    searchQuery.trim() === ''
      ? playgrounds
      : playgrounds.filter(n => n.toLowerCase().includes(searchQuery.toLowerCase()))
  )

  // ── New playground inline input ───────────────────────────────────────────────
  let newNameValue: string = $state('')
  let newNameError: string = $state('')
  let newInputEl: HTMLInputElement | null = $state(null)

  // Focus the input whenever creatingNew becomes true
  $effect(() => {
    if (creatingNew) {
      newNameValue = ''
      newNameError = ''
      tick().then(() => newInputEl?.focus())
    }
  })

  function confirmNew() {
    const name = newNameValue.trim().toLowerCase()
    if (!name) { cancelNew(); return }
    if (!/^[a-z][a-z0-9_]*$/.test(name)) {
      newNameError = 'Lowercase letters, digits, underscores only'
      return
    }
    if (name.length > 64) {
      newNameError = 'Max 64 characters'
      return
    }
    creatingNew = false
    newNameValue = ''
    newNameError = ''
    dispatch('new', name)
  }

  function cancelNew() {
    creatingNew = false
    newNameValue = ''
    newNameError = ''
  }

  function handleNewKey(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); confirmNew() }
    if (e.key === 'Escape') cancelNew()
  }

  // ── Context menu / rename ─────────────────────────────────────────────────────
  function openContext(e: MouseEvent, name: string) {
    e.preventDefault()
    contextMenu = { x: e.clientX, y: e.clientY, name }
  }

  function closeContext() { contextMenu = null }

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

  // ── Cargo.toml panel ──────────────────────────────────────────────────────────
  let cargoExpanded: boolean = $state(true)
</script>

<svelte:window onclick={closeContext} />

<aside class="sidebar">
  <!-- ── Header ── -->
  <div class="sidebar-header">
    <span class="sidebar-title">Playgrounds</span>
    <button
      class="icon-btn"
      title="New playground (⌘N)"
      onclick={() => creatingNew = true}
    >
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
        <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  <!-- ── Search ── -->
  <div class="search-wrap">
    <svg class="search-icon" width="12" height="12" viewBox="0 0 12 12" fill="none">
      <circle cx="5" cy="5" r="3.5" stroke="currentColor" stroke-width="1.4"/>
      <path d="M8 8l2.5 2.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
    </svg>
    <input
      class="search-input"
      type="search"
      placeholder="Filter"
      bind:value={searchQuery}
      onclick={(e) => e.stopPropagation()}
    />
    {#if searchQuery}
      <button class="search-clear" onclick={() => searchQuery = ''}>×</button>
    {/if}
  </div>

  <!-- ── Playground list ── -->
  <ul class="playground-list">
    <!-- Inline new playground input -->
    {#if creatingNew}
      <li class="new-item">
        <span class="file-icon">RS</span>
        <div class="new-input-wrap">
          <input
            class="new-input"
            type="text"
            placeholder="playground_name"
            bind:value={newNameValue}
            bind:this={newInputEl}
            onkeydown={handleNewKey}
            onblur={() => {
              // Small delay so a click on confirm doesn't double-fire
              setTimeout(() => { if (creatingNew) cancelNew() }, 150)
            }}
            onclick={(e) => e.stopPropagation()}
          />
          {#if newNameError}
            <span class="new-error">{newNameError}</span>
          {/if}
        </div>
        <button class="new-confirm" onclick={confirmNew} title="Create">↵</button>
      </li>
    {/if}

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
            class="rename-input"
            bind:value={renameValue}
            onblur={commitRename}
            onkeydown={handleRenameKey}
            onclick={(e) => e.stopPropagation()}
            autofocus
          />
        {:else}
          <span class="file-icon">RS</span>
          <span class="name">{name}</span>
          {#if isDirty}
            <span class="dirty-dot" title="Unsaved changes">●</span>
          {/if}
        {/if}
      </li>
    {/each}

    {#if filtered.length === 0 && !creatingNew}
      <li class="empty-hint">
        {searchQuery ? 'No matches' : 'No playgrounds yet'}
      </li>
    {/if}
  </ul>

  <!-- ── Cargo.toml panel ── -->
  <div class="cargo-section">
    <div
      class="cargo-header"
      role="button"
      tabindex="0"
      onclick={() => cargoExpanded = !cargoExpanded}
      onkeydown={(e) => e.key === 'Enter' && (cargoExpanded = !cargoExpanded)}
    >
      <svg width="10" height="10" viewBox="0 0 10 10" class="chevron" class:open={cargoExpanded}>
        <path d="M2 3.5L5 6.5L8 3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" fill="none"/>
      </svg>
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
</aside>

<!-- ── Context menu ── -->
{#if contextMenu}
  <div
    class="context-menu"
    role="menu"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === 'Escape' && closeContext()}
  >
    <button onclick={() => startRename(contextMenu!.name)}>Rename</button>
    <button onclick={() => { dispatch('duplicate', contextMenu!.name); contextMenu = null }}>Duplicate</button>
    <hr />
    <button class="danger" onclick={() => { dispatch('delete', contextMenu!.name); contextMenu = null }}>Delete</button>
  </div>
{/if}

<style>
  .sidebar {
    width: var(--sidebar-width);
    flex-shrink: 0;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Header ── */
  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px 8px;
    flex-shrink: 0;
  }

  .sidebar-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: -0.01em;
  }

  .icon-btn {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: var(--radius-xs);
    color: var(--text-tertiary);
    transition: background 0.12s, color 0.12s;
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

  /* ── List ── */
  .playground-list {
    list-style: none;
    overflow-y: auto;
    flex: 1;
    padding: 2px 6px 4px;
    min-height: 0;
  }

  /* ── New playground input row ── */
  .new-item {
    display: flex; align-items: center; gap: 8px;
    padding: 4px 8px;
    border-radius: 8px;
    background: rgba(10, 132, 255, 0.12);
    border: 1px solid rgba(10, 132, 255, 0.3);
    margin-bottom: 2px;
  }

  .new-input-wrap {
    flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px;
  }

  .new-input {
    flex: 1; font-size: 12px; font-family: var(--font-mono);
    padding: 2px 4px; border-radius: var(--radius-xs);
    background: rgba(255,255,255,0.08); border: 1px solid var(--border-strong);
    color: var(--text); min-width: 0;
  }
  .new-input:focus { outline: none; border-color: var(--accent); }

  .new-error {
    font-size: 10px; color: var(--red); line-height: 1.3;
  }

  .new-confirm {
    flex-shrink: 0; width: 20px; height: 20px;
    display: flex; align-items: center; justify-content: center;
    border-radius: var(--radius-xs);
    background: var(--accent); color: #fff;
    font-size: 13px; line-height: 1;
  }
  .new-confirm:hover { background: var(--accent-hover); }

  /* ── Playground items ── */
  .playground-item {
    display: flex; align-items: center; gap: 8px;
    padding: 6px 8px; border-radius: 8px;
    cursor: pointer; user-select: none;
    transition: background 0.1s;
  }
  .playground-item:hover:not(.active) { background: var(--bg-hover); }
  .playground-item.active { background: var(--accent); }
  .playground-item.active .name { color: #fff; }
  .playground-item.active .file-icon {
    background: rgba(255,255,255,0.25); color: #fff;
  }
  .playground-item.active .dirty-dot { color: rgba(255,255,255,0.7); }

  .file-icon {
    font-size: 7px; font-weight: 800; letter-spacing: 0.03em;
    background: var(--rust-orange); color: #fff;
    border-radius: 3px; padding: 2px 3px;
    flex-shrink: 0; line-height: 1.3;
    min-width: 18px; text-align: center;
  }

  .name {
    flex: 1; font-size: 13px;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    color: var(--text-secondary);
  }

  .dirty-dot { color: var(--accent); font-size: 8px; flex-shrink: 0; }

  .empty-hint {
    padding: 8px 8px; color: var(--text-tertiary);
    font-size: 12px; font-style: italic;
  }

  .rename-input { flex: 1; font-size: 13px; padding: 2px 6px; border-radius: var(--radius-xs); }

  /* ── Cargo.toml panel ── */
  .cargo-section {
    flex-shrink: 0;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    max-height: 200px;
  }

  .cargo-header {
    display: flex; align-items: center; gap: 6px;
    padding: 7px 10px; cursor: pointer;
    background: none; width: 100%;
    color: var(--text-secondary); font-size: 11px; font-weight: 600;
    transition: background 0.1s;
    flex-shrink: 0;
  }
  .cargo-header:hover { background: var(--bg-hover); }

  .chevron {
    color: var(--text-tertiary);
    transition: transform 0.15s;
    flex-shrink: 0;
  }
  .chevron.open { transform: rotate(0deg); }
  .chevron:not(.open) { transform: rotate(-90deg); }

  .cargo-label { flex: 1; text-align: left; letter-spacing: -0.01em; }

  .cargo-edit-btn {
    font-size: 10px; font-weight: 500;
    color: var(--accent); padding: 1px 6px;
    border: 1px solid rgba(10,132,255,0.3);
    border-radius: var(--radius-xs);
    transition: background 0.1s;
  }
  .cargo-edit-btn:hover { background: rgba(10,132,255,0.15); }

  .cargo-body {
    overflow-y: auto; flex: 1; min-height: 0;
  }

  .cargo-pre {
    margin: 0; padding: 8px 10px;
    font-family: var(--font-mono); font-size: 10.5px; line-height: 1.6;
    color: var(--text-secondary);
    white-space: pre-wrap; word-break: break-word;
  }

  .cargo-empty {
    display: block; padding: 8px 10px;
    font-size: 11px; color: var(--text-tertiary); font-style: italic;
  }

  /* ── Context menu ── */
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
