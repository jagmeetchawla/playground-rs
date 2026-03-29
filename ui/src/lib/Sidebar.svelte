<script lang="ts">
  import { createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  let { playgrounds, selected, dirty }: {
    playgrounds: string[]; selected: string | null; dirty: boolean
  } = $props()

  let contextMenu: { x: number; y: number; name: string } | null = $state(null)
  let renamingName: string | null = $state(null)
  let renameValue: string = $state('')

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
  <div class="sidebar-header">
    <span class="sidebar-title">PLAYGROUNDS</span>
    <button class="new-btn" title="New playground (⌘N)" onclick={() => dispatch('new')}>+</button>
  </div>

  <ul class="playground-list">
    {#each playgrounds as name (name)}
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
          <span class="name">{name}</span>
          {#if selected === name && dirty}
            <span class="dirty-dot" title="Unsaved changes">●</span>
          {/if}
        {/if}
      </li>
    {/each}

    {#if playgrounds.length === 0}
      <li class="empty-hint">No playgrounds yet</li>
    {/if}
  </ul>
</aside>

<!-- Context menu -->
{#if contextMenu}
  <div
    class="context-menu"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
  >
    <button onclick={() => startRename(contextMenu!.name)}>Rename</button>
    <button onclick={() => { dispatch('duplicate', contextMenu!.name); contextMenu = null }}>Duplicate</button>
    <hr />
    <button class="danger" onclick={() => { dispatch('delete', contextMenu!.name); contextMenu = null }}>Delete</button>
  </div>
{/if}

<style>
  .sidebar {
    width: 200px; flex-shrink: 0;
    background: var(--bg-panel); border-right: 1px solid var(--border);
    display: flex; flex-direction: column; overflow: hidden;
  }

  .sidebar-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px 6px;
    border-bottom: 1px solid var(--border);
  }

  .sidebar-title {
    font-size: 10px; font-weight: 700; letter-spacing: 0.08em;
    color: var(--text-dim); text-transform: uppercase;
  }

  .new-btn {
    background: none; color: var(--text-dim); font-size: 18px;
    line-height: 1; padding: 0 2px; width: 22px; height: 22px;
    display: flex; align-items: center; justify-content: center;
  }
  .new-btn:hover { color: var(--text); background: var(--bg-hover); border-radius: 3px; }

  .playground-list { list-style: none; overflow-y: auto; flex: 1; padding: 4px 0; }

  .playground-item {
    display: flex; align-items: center; justify-content: space-between;
    padding: 5px 12px; cursor: pointer; border-radius: 0;
    user-select: none;
  }
  .playground-item:hover  { background: var(--bg-hover); }
  .playground-item.active { background: var(--bg-active); color: white; }

  .name { font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  .dirty-dot { color: var(--accent); font-size: 10px; margin-left: 4px; flex-shrink: 0; }

  .empty-hint { padding: 8px 12px; color: var(--text-dim2); font-size: 12px; font-style: italic; }

  .rename-input { flex: 1; font-size: 13px; }

  /* Context menu */
  .context-menu {
    position: fixed; z-index: 1000;
    background: var(--bg-panel); border: 1px solid var(--border);
    border-radius: var(--radius); padding: 4px 0;
    box-shadow: 0 4px 16px rgba(0,0,0,0.4);
    min-width: 140px;
  }
  .context-menu button {
    display: block; width: 100%; text-align: left;
    background: none; color: var(--text); padding: 6px 14px; font-size: 13px;
    border-radius: 0;
  }
  .context-menu button:hover { background: var(--bg-active); }
  .context-menu button.danger { color: var(--red); }
  .context-menu hr { border: none; border-top: 1px solid var(--border); margin: 4px 0; }
</style>
