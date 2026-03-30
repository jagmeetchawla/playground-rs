<script lang="ts">
  import { createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  let { tabs, active, dirtyTabs }: {
    tabs: string[]
    active: string | null
    dirtyTabs: string[]
  } = $props()
</script>

<div class="tabbar" role="tablist">
  {#each tabs as name (name)}
    {@const isDirty = dirtyTabs.includes(name)}
    <!-- Use div+role instead of button so we can nest the close button inside -->
    <div
      class="tab"
      class:active={active === name}
      role="tab"
      tabindex="0"
      aria-selected={active === name}
      onclick={() => dispatch('activate', name)}
      onkeydown={(e) => e.key === 'Enter' && dispatch('activate', name)}
    >
      <!-- RS file badge -->
      <span class="file-badge">RS</span>

      <span class="tab-name">{name}{isDirty ? ' ●' : ''}</span>

      <button
        class="close-btn"
        title="Close tab"
        onclick={(e) => { e.stopPropagation(); dispatch('close', name) }}
      >×</button>
    </div>
  {/each}

  {#if tabs.length === 0}
    <span class="no-tabs">Open a playground from the sidebar</span>
  {/if}
</div>

<style>
  .tabbar {
    display: flex;
    align-items: stretch;
    height: var(--tab-height);
    background: var(--bg);
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
    overflow-y: hidden;
    flex-shrink: 0;
    /* Hide scrollbar but keep scrollable */
    scrollbar-width: none;
  }
  .tabbar::-webkit-scrollbar { display: none; }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 10px 0 10px;
    min-width: 100px;
    max-width: 180px;
    height: 100%;
    border-radius: 0;
    border-right: 1px solid var(--border);
    background: var(--bg-tab);
    color: var(--text-tertiary);
    font-size: 12px;
    font-weight: 400;
    white-space: nowrap;
    position: relative;
    transition: background 0.1s, color 0.1s;
    flex-shrink: 0;
  }

  .tab:hover {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }

  .tab.active {
    background: var(--bg-tab-active);
    color: var(--text);
    font-weight: 500;
  }

  /* Accent underline on active tab */
  .tab.active::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--accent);
  }

  .file-badge {
    font-size: 8px;
    font-weight: 800;
    letter-spacing: 0.03em;
    background: var(--rust-orange);
    color: #fff;
    border-radius: 3px;
    padding: 1px 3px;
    flex-shrink: 0;
    line-height: 1.4;
  }

  .tab-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    text-align: left;
  }

  .close-btn {
    flex-shrink: 0;
    width: 16px;
    height: 16px;
    border-radius: 3px;
    font-size: 14px;
    line-height: 1;
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    opacity: 0;
    transition: opacity 0.1s, background 0.1s, color 0.1s;
  }

  .tab:hover .close-btn,
  .tab.active .close-btn {
    opacity: 1;
  }

  .close-btn:hover {
    background: var(--bg-elevated);
    color: var(--text);
  }

  .no-tabs {
    padding: 0 16px;
    line-height: var(--tab-height);
    color: var(--text-tertiary);
    font-size: 12px;
    font-style: italic;
  }
</style>
