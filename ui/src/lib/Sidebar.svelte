<script lang="ts">
  import { createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  let { playgrounds, selected, dirtyTabs }: {
    playgrounds: string[]
    selected: string | null
    dirtyTabs: string[]
  } = $props()

  let contextMenu: { x: number; y: number; name: string } | null = $state(null)
  let renamingName: string | null = $state(null)
  let renameValue: string = $state('')
  let searchQuery: string = $state('')

  let filtered = $derived(
    searchQuery.trim() === ''
      ? playgrounds
      : playgrounds.filter(n => n.toLowerCase().includes(searchQuery.toLowerCase()))
  )

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
</script>

<svelte:window onclick={closeContext} />

<aside class="sidebar">
  <!-- Header -->
  <div class="sidebar-header">
    <span class="sidebar-title">Playgrounds</span>
    <button
      class="icon-btn"
      title="New playground (⌘N)"
      onclick={() => dispatch('new')}
    >
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
        <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  <!-- Search -->
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

  <!-- Playground list -->
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
            class="rename-input"
            bind:value={renameValue}
            onblur={commitRename}
            onkeydown={handleRenameKey}
            onclick={(e) => e.stopPropagation()}
            autofocus
          />
        {:else}
          <!-- RS file icon -->
          <span class="file-icon">RS</span>
          <span class="name">{name}</span>
          {#if isDirty}
            <span class="dirty-dot" title="Unsaved changes">●</span>
          {/if}
        {/if}
      </li>
    {/each}

    {#if filtered.length === 0}
      <li class="empty-hint">
        {searchQuery ? 'No matches' : 'No playgrounds yet'}
      </li>
    {/if}
  </ul>
</aside>

<!-- Context menu -->
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
  }

  .sidebar-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: -0.01em;
  }

  .icon-btn {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-xs);
    color: var(--text-tertiary);
    transition: background 0.12s, color 0.12s;
  }
  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  /* ── Search ── */
  .search-wrap {
    display: flex;
    align-items: center;
    margin: 0 8px 8px;
    background: rgba(255,255,255,0.08);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 5px 8px;
    gap: 6px;
  }

  .search-icon {
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    border-radius: 0;
    padding: 0;
    font-size: 12px;
    color: var(--text);
    outline: none;
    min-width: 0;
    /* Remove browser search input decorations */
    -webkit-appearance: none;
  }
  .search-input::-webkit-search-cancel-button { display: none; }
  .search-input::placeholder { color: var(--text-tertiary); }

  .search-clear {
    flex-shrink: 0;
    color: var(--text-tertiary);
    font-size: 14px;
    line-height: 1;
    padding: 0;
    width: 14px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .search-clear:hover { color: var(--text); }

  /* ── List ── */
  .playground-list {
    list-style: none;
    overflow-y: auto;
    flex: 1;
    padding: 2px 6px 8px;
  }

  .playground-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 8px;
    cursor: pointer;
    user-select: none;
    transition: background 0.1s;
  }

  .playground-item:hover:not(.active) {
    background: var(--bg-hover);
  }

  /* Blue pill — Apple style */
  .playground-item.active {
    background: var(--accent);
  }
  .playground-item.active .name { color: #fff; }
  .playground-item.active .file-icon {
    background: rgba(255,255,255,0.25);
    color: #fff;
  }
  .playground-item.active .dirty-dot {
    color: rgba(255,255,255,0.7);
  }

  .file-icon {
    font-size: 7px;
    font-weight: 800;
    letter-spacing: 0.03em;
    background: var(--rust-orange);
    color: #fff;
    border-radius: 3px;
    padding: 2px 3px;
    flex-shrink: 0;
    line-height: 1.3;
    min-width: 18px;
    text-align: center;
  }

  .name {
    flex: 1;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-secondary);
  }

  .dirty-dot {
    color: var(--accent);
    font-size: 8px;
    flex-shrink: 0;
  }

  .empty-hint {
    padding: 8px 8px;
    color: var(--text-tertiary);
    font-size: 12px;
    font-style: italic;
  }

  .rename-input {
    flex: 1;
    font-size: 13px;
    padding: 2px 6px;
    border-radius: var(--radius-xs);
  }

  /* ── Context menu ── */
  .context-menu {
    position: fixed;
    z-index: 1000;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    padding: 4px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.5), 0 2px 8px rgba(0,0,0,0.3);
    min-width: 150px;
  }

  .context-menu button {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 10px;
    font-size: 13px;
    border-radius: var(--radius-xs);
    color: var(--text);
  }
  .context-menu button:hover { background: var(--accent); color: #fff; }
  .context-menu button.danger { color: var(--red); }
  .context-menu button.danger:hover { background: var(--red); color: #fff; }

  .context-menu hr {
    border: none;
    border-top: 1px solid var(--border);
    margin: 4px 0;
  }
</style>
