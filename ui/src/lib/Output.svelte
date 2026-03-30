<script lang="ts">
  import { tick, createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  type OutputLine = { stream: 'stdout' | 'stderr' | 'info'; line: string }

  let { output, status }: {
    output: OutputLine[]
    status: string
  } = $props()

  let container: HTMLDivElement

  $effect(() => {
    output // track dependency
    tick().then(() => {
      if (container) container.scrollTop = container.scrollHeight
    })
  })

  // Count errors in output
  let errorCount = $derived(output.filter(l => l.stream === 'stderr').length)
</script>

<div class="output-panel">
  <!-- Header — "App Preview" style -->
  <div class="output-header">
    <div class="header-left">
      <span class="panel-title">Console</span>
    </div>
    <div class="header-right">
      {#if errorCount > 0}
        <span class="error-badge">{errorCount}</span>
      {/if}
      {#if status === 'running' || status === 'compiling'}
        <span class="spinner" title="Running…"></span>
      {/if}
      {#if output.length > 0}
        <button class="clear-btn" onclick={() => dispatch('clear')} title="Clear console">
          Clear
        </button>
      {/if}
    </div>
  </div>

  <!-- Output lines -->
  <div class="output-lines" bind:this={container}>
    {#each output as entry, i (i)}
      <div class="line {entry.stream}">
        {#if entry.stream === 'info'}
          <span class="info-prefix">›</span>
        {/if}
        {entry.line}
      </div>
    {/each}

    {#if output.length === 0}
      <div class="empty-state">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" opacity="0.3">
          <polygon points="5,3 19,12 5,21" fill="currentColor"/>
        </svg>
        <span>Run a playground to see output</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .output-panel {
    width: 320px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-sidebar);
    border-left: 1px solid var(--border);
  }

  /* ── Header ── */
  .output-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    height: var(--tab-height);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .header-left { display: flex; align-items: center; gap: 8px; }
  .header-right { display: flex; align-items: center; gap: 8px; }

  .clear-btn {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-tertiary);
    padding: 2px 7px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--border);
    transition: color 0.1s, border-color 0.1s, background 0.1s;
  }
  .clear-btn:hover {
    color: var(--text);
    border-color: var(--border-strong);
    background: var(--bg-hover);
  }

  .panel-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: -0.01em;
  }

  .error-badge {
    font-size: 10px;
    font-weight: 700;
    background: var(--red);
    color: #fff;
    border-radius: 8px;
    padding: 1px 5px;
    min-width: 16px;
    text-align: center;
    line-height: 1.5;
  }

  /* CSS spinner — matches the running indicator */
  .spinner {
    width: 12px;
    height: 12px;
    border: 1.5px solid var(--border-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* ── Lines ── */
  .output-lines {
    flex: 1;
    overflow-y: auto;
    padding: 8px 4px;
  }

  .line {
    display: flex;
    gap: 8px;
    padding: 1px 10px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    line-height: 1.65;
    white-space: pre-wrap;
    word-break: break-all;
    border-radius: var(--radius-xs);
  }
  .line:hover { background: var(--bg-hover); }

  .line.stdout { color: var(--text); }
  .line.stderr { color: var(--red); }
  .line.info   { color: var(--text-tertiary); }

  .info-prefix {
    color: var(--accent);
    flex-shrink: 0;
    font-weight: 700;
  }

  /* ── Empty state ── */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    height: 100%;
    color: var(--text-tertiary);
    font-size: 12px;
    padding: 40px 20px;
    text-align: center;
  }
</style>
