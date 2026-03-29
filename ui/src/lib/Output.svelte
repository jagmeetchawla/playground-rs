<script lang="ts">
  import { tick } from 'svelte'

  type OutputLine = { stream: 'stdout' | 'stderr' | 'info'; line: string }
  let { output }: { output: OutputLine[] } = $props()

  let container: HTMLDivElement

  // Auto-scroll to bottom on new output
  $effect(() => {
    output // track dependency
    tick().then(() => {
      if (container) container.scrollTop = container.scrollHeight
    })
  })

  function clear() {
    // Parent clears via binding — emit event upward
    // For now, done via the parent App.svelte binding
  }
</script>

<div class="output-panel">
  <div class="output-header">
    <span class="output-title">OUTPUT</span>
  </div>
  <div class="output-lines" bind:this={container}>
    {#each output as line (line)}
      <div class="line {line.stream}">{line.line}</div>
    {/each}
    {#if output.length === 0}
      <div class="line hint">Run a playground to see output here…</div>
    {/if}
  </div>
</div>

<style>
  .output-panel {
    width: 340px; flex-shrink: 0;
    display: flex; flex-direction: column;
    background: var(--bg-panel); border-left: 1px solid var(--border);
  }

  .output-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px 6px; border-bottom: 1px solid var(--border); flex-shrink: 0;
  }

  .output-title {
    font-size: 10px; font-weight: 700; letter-spacing: 0.08em;
    color: var(--text-dim); text-transform: uppercase;
  }

  .output-lines {
    flex: 1; overflow-y: auto; padding: 10px 12px;
    font-family: var(--font-mono); font-size: 12px; line-height: 1.6;
  }

  .line { white-space: pre-wrap; word-break: break-all; }
  .line.stdout { color: var(--text); }
  .line.stderr { color: var(--red); }
  .line.info   { color: var(--text-dim); font-style: italic; }
  .line.hint   { color: var(--text-dim2); font-style: italic; }
</style>
