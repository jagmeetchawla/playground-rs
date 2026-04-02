<script lang="ts">
  import { tick, createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  export type OutputLine = { stream: 'stdout' | 'stderr' | 'info' | 'stdin'; line: string }
  export type RunBlock = {
    runNum: number
    command: string
    startedAt: string
    status: 'compiling' | 'running' | 'success' | 'error'
    exitCode: number | null
    compilerLines: OutputLine[]
    programLines: OutputLine[]
    collapsed: boolean
    programStarted: boolean
  }

  let { runs, status }: {
    runs: RunBlock[]
    status: string
  } = $props()

  let container: HTMLElement

  // Auto-scroll to bottom when the latest run has new lines
  $effect(() => {
    // Touch reactive deps
    const last = runs.at(-1)
    if (!last || last.collapsed) return
    last.compilerLines.length
    last.programLines.length
    tick().then(() => {
      if (container) container.scrollTop = container.scrollHeight
    })
  })

  let hasRuns = $derived(runs.length > 0)

  function statusIcon(b: RunBlock): string {
    if (b.status === 'success') return '✓'
    if (b.status === 'error')   return '✗'
    if (b.status === 'running') return '▶'
    return '…'
  }

  function statusClass(b: RunBlock): string {
    if (b.status === 'success') return 'ok'
    if (b.status === 'error')   return 'err'
    return 'dim'
  }

  let stdinValue = $state('')
  let stdinInput = $state<HTMLInputElement | null>(null)

  function handleStdin(e: KeyboardEvent) {
    if (e.key === 'Enter' && stdinValue.trim() !== '') {
      dispatch('stdin', stdinValue)
      stdinValue = ''
    }
  }

  // Auto-focus the stdin input when a program starts running
  $effect(() => {
    const last = runs.at(-1)
    if (last?.status === 'running' && stdinInput) {
      tick().then(() => stdinInput?.focus())
    }
  })
</script>

<div class="output-panel">
  <!-- ── Header ── -->
  <div class="output-header">
    <div class="header-left">
      <span class="panel-title">Console</span>
      {#if status === 'compiling' || status === 'running'}
        <span class="spinner" title="Running…"></span>
      {/if}
    </div>
    <div class="header-right">
      {#if hasRuns}
        <button class="clear-btn" onclick={() => dispatch('clear')}>Clear</button>
      {/if}
    </div>
  </div>

  <!-- ── Run blocks ── -->
  <div class="runs-container" bind:this={container}>
    {#each runs as block (block.runNum)}
      <div class="run-block" class:collapsed={block.collapsed}>
        <!-- Block header — click to collapse/expand -->
        <button
          class="run-header"
          onclick={() => dispatch('toggle', block.runNum)}
        >
          <svg class="chevron" class:open={!block.collapsed} width="10" height="10" viewBox="0 0 10 10">
            <path d="M2 3.5L5 6.5L8 3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" fill="none"/>
          </svg>
          <span class="run-num">Run #{block.runNum}</span>
          <span class="run-cmd">{block.command}</span>
          <span class="run-time">{block.startedAt}</span>
          <span class="run-status {statusClass(block)}">{statusIcon(block)}</span>
        </button>

        <!-- Block body — only when expanded -->
        {#if !block.collapsed}
          <div class="run-body">
            <!-- Compiler sub-block -->
            {#if block.compilerLines.length > 0}
              <div class="sub-block">
                <div class="sub-header">
                  <svg width="9" height="9" viewBox="0 0 10 10" fill="none" style="margin-right:3px;vertical-align:middle;opacity:0.6">
                    <path d="M2 2l3 3-3 3M6 8h3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                  Compiler
                </div>
                <div class="code-box compiler-box">
                  {#each block.compilerLines as line}
                    <div class="line {line.stream}">{line.line}</div>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Output sub-block -->
            {#if block.programLines.length > 0}
              <div class="sub-block">
                <div class="sub-header">Output</div>
                <div class="code-box output-box">
                  {#each block.programLines as line}
                    <div class="line {line.stream}">{line.line}</div>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Running indicator inside the block -->
            {#if block.status === 'compiling'}
              <div class="run-in-progress">
                <span class="spinner-sm"></span>
                <span class="dim">Compiling…</span>
              </div>
            {/if}

            <!-- Stdin input — shown when program is running -->
            {#if block.status === 'running'}
              <div class="stdin-row">
                <span class="stdin-prompt">›</span>
                <input
                  bind:this={stdinInput}
                  bind:value={stdinValue}
                  onkeydown={handleStdin}
                  class="stdin-input"
                  placeholder="Type input and press Enter…"
                  spellcheck="false"
                  autocomplete="off"
                />
              </div>
            {/if}

            <!-- Empty run (compilation succeeded but no output) -->
            {#if block.status === 'success' && block.programLines.length === 0 && block.compilerLines.length === 0}
              <div class="dim" style="padding: 6px 12px; font-size: 11px; font-style: italic;">No output</div>
            {/if}
          </div>
        {/if}
      </div>
    {/each}

    {#if !hasRuns}
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
    width: 100%; height: 100%;
    display: flex; flex-direction: column;
    background: var(--bg-sidebar);
  }

  /* ── Header ── */
  .output-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 0 12px; height: var(--tab-height);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .header-left { display: flex; align-items: center; gap: 8px; }
  .header-right { display: flex; align-items: center; gap: 8px; }

  .panel-title {
    font-size: 12px; font-weight: 600;
    color: var(--text-secondary); letter-spacing: -0.01em;
  }

  .spinner {
    width: 12px; height: 12px;
    border: 1.5px solid var(--border-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  .clear-btn {
    font-size: 11px; font-weight: 500; color: var(--text-tertiary);
    padding: 2px 7px; border-radius: var(--radius-xs);
    border: 1px solid var(--border);
    transition: color 0.1s, border-color 0.1s, background 0.1s;
  }
  .clear-btn:hover { color: var(--text); border-color: var(--border-strong); background: var(--bg-hover); }

  /* ── Run blocks ── */
  .runs-container {
    flex: 1; overflow-y: auto; padding: 6px 0;
  }

  .run-block {
    border-bottom: 1px solid var(--border);
  }
  .run-block:last-child { border-bottom: none; }

  .run-header {
    display: flex; align-items: center; gap: 6px;
    width: 100%; padding: 6px 10px;
    font-size: 11px; color: var(--text-tertiary);
    background: none; text-align: left;
    transition: background 0.1s;
    cursor: pointer;
  }
  .run-header:hover { background: var(--bg-hover); }

  .chevron {
    flex-shrink: 0; color: var(--text-tertiary);
    transition: transform 0.15s;
    transform: rotate(-90deg);
  }
  .chevron.open { transform: rotate(0deg); }

  .run-num { font-weight: 600; color: var(--text-secondary); flex-shrink: 0; }

  .run-cmd {
    flex: 1; font-family: var(--font-mono); font-size: 10px;
    color: var(--text-tertiary); overflow: hidden;
    text-overflow: ellipsis; white-space: nowrap; min-width: 0;
  }

  .run-time { font-size: 10px; flex-shrink: 0; color: var(--text-tertiary); }

  .run-status { font-size: 11px; font-weight: 700; flex-shrink: 0; }
  .run-status.ok  { color: var(--green); }
  .run-status.err { color: var(--red); }
  .run-status.dim { color: var(--text-tertiary); }

  /* ── Block body ── */
  .run-body { padding: 0 0 6px; }

  .sub-block { margin: 4px 0 0; }

  .sub-header {
    font-size: 9px; font-weight: 700; letter-spacing: 0.06em;
    text-transform: uppercase; color: var(--text-tertiary);
    padding: 4px 10px 4px;
    display: flex; align-items: center;
  }

  /* ── Code box (shared by compiler + output sub-blocks) ── */
  .code-box {
    border-radius: 5px;
    border: 1px solid var(--border);
    padding: 6px 2px;
    overflow-x: auto;
  }

  /* Compiler block — slightly darker / more distinct background */
  .compiler-box {
    background: rgba(0, 0, 0, 0.28);
    border-color: rgba(255, 255, 255, 0.07);
  }

  /* Program output block — neutral surface, same family but lighter */
  .output-box {
    background: rgba(0, 0, 0, 0.14);
    border-color: rgba(255, 255, 255, 0.06);
  }

  .line {
    font-family: var(--font-mono); font-size: 11.5px; line-height: 1.65;
    white-space: pre-wrap; word-break: break-all;
    padding: 0 8px; border-radius: var(--radius-xs);
  }
  .line:hover { background: rgba(255,255,255,0.04); }
  .line.stdout { color: var(--text); }
  .line.stderr { color: #ff8080; }
  .line.info   { color: var(--text-tertiary); }
  .line.stdin  { color: var(--accent); opacity: 0.85; }

  .run-in-progress {
    display: flex; align-items: center; gap: 6px;
    padding: 6px 12px; color: var(--text-tertiary); font-size: 11px;
  }

  .spinner-sm {
    width: 10px; height: 10px;
    border: 1.5px solid var(--border-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  .dim { color: var(--text-tertiary); }

  /* ── Stdin input ── */
  .stdin-row {
    display: flex; align-items: center; gap: 4px;
    padding: 4px 8px; margin: 4px 0 0;
    border-top: 1px solid var(--border);
  }
  .stdin-prompt {
    font-family: var(--font-mono); font-size: 12px; font-weight: 700;
    color: var(--accent); flex-shrink: 0; opacity: 0.7;
  }
  .stdin-input {
    flex: 1; font-family: var(--font-mono); font-size: 11.5px;
    background: rgba(0, 0, 0, 0.2); color: var(--text);
    border: 1px solid var(--border); border-radius: var(--radius-xs);
    padding: 3px 6px; outline: none;
  }
  .stdin-input:focus { border-color: var(--accent); }
  .stdin-input::placeholder { color: var(--text-tertiary); opacity: 0.5; }

  /* ── Empty state ── */
  .empty-state {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; gap: 10px; height: 100%;
    color: var(--text-tertiary); font-size: 12px;
    padding: 40px 20px; text-align: center;
  }

  @keyframes spin { to { transform: rotate(360deg); } }
</style>
