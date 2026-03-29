<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke, Channel } from '@tauri-apps/api/core'
  import Sidebar from './lib/Sidebar.svelte'
  import Editor from './lib/Editor.svelte'
  import Output from './lib/Output.svelte'

  type Status = 'idle' | 'checking' | 'compiling' | 'running' | 'error'
  type OutputLine = { stream: 'stdout' | 'stderr' | 'info'; line: string }

  let playgrounds: string[] = $state([])
  let selected: string | null = $state(null)
  let code: string = $state('')
  let dirty: boolean = $state(false)
  let output: OutputLine[] = $state([])
  let status: Status = $state('idle')

  onMount(async () => {
    playgrounds = await invoke<string[]>('list_playgrounds')
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  })

  function handleKey(e: KeyboardEvent) {
    if (e.metaKey && e.key === 'r') { e.preventDefault(); run() }
    if (e.metaKey && e.key === '.') { e.preventDefault(); stop() }
    if (e.metaKey && e.key === 's') { e.preventDefault(); save() }
    if (e.metaKey && e.key === 'n') { e.preventDefault(); requestNewPlayground() }
  }

  async function selectPlayground(name: string) {
    selected = name
    code = await invoke<string>('load_playground', { name })
    dirty = false
    output = []
  }

  async function save() {
    if (!selected) return
    await invoke('save_playground', { name: selected, content: code })
    dirty = false
  }

  async function run() {
    if (!selected || status !== 'idle') return
    await save()
    output = [{ stream: 'info', line: `▶ cargo run --bin ${selected}` }]
    status = 'compiling'

    const channel = new Channel()
    channel.onmessage = (msg: any) => {
      if (msg.stream === 'complete') {
        status = msg.code === 0 ? 'idle' : 'error'
      } else {
        output = [...output, { stream: msg.stream, line: msg.line }]
        if (msg.stream === 'stdout') status = 'running'
      }
    }

    try {
      await invoke('run_playground', { name: selected, onOutput: channel })
    } catch (e) {
      output = [...output, { stream: 'stderr', line: String(e) }]
      status = 'error'
    }
  }

  function stop() {
    // TODO: kill process — implement in v1.0 iteration 2
    output = [...output, { stream: 'info', line: '— stopped —' }]
    status = 'idle'
  }

  function onCodeChange(newCode: string) {
    code = newCode
    dirty = true
  }

  async function requestNewPlayground() {
    const name = window.prompt('Playground name (e.g. my_demo):')
    if (!name) return
    await invoke('new_playground', { name })
    playgrounds = await invoke<string[]>('list_playgrounds')
    await selectPlayground(name)
  }

  async function onRename(e: CustomEvent<{ old: string; new: string }>) {
    await invoke('rename_playground', { oldName: e.detail.old, newName: e.detail.new })
    playgrounds = await invoke<string[]>('list_playgrounds')
    if (selected === e.detail.old) {
      selected = e.detail.new
    }
  }

  async function onDelete(e: CustomEvent<string>) {
    const name = e.detail
    if (!confirm(`Delete playground "${name}"? This cannot be undone.`)) return
    await invoke('delete_playground', { name })
    playgrounds = await invoke<string[]>('list_playgrounds')
    if (selected === name) { selected = null; code = ''; dirty = false }
  }

  async function onDuplicate(e: CustomEvent<string>) {
    const newName = await invoke<string>('duplicate_playground', { name: e.detail })
    playgrounds = await invoke<string[]>('list_playgrounds')
    await selectPlayground(newName)
  }

  const statusLabel: Record<Status, string> = {
    idle: 'idle', checking: 'checking…', compiling: 'compiling…',
    running: 'running', error: 'error',
  }
</script>

<div class="app">
  <!-- Toolbar -->
  <header class="toolbar">
    <span class="app-name">Rust Playground</span>
    <span class="status" class:error={status === 'error'} class:running={status === 'running' || status === 'compiling'}>
      {statusLabel[status]}
    </span>
    {#if status === 'running' || status === 'compiling'}
      <button class="btn-stop" onclick={stop}>■ Stop</button>
    {:else}
      <button class="btn-run" onclick={run} disabled={!selected}>▶ Run</button>
    {/if}
  </header>

  <!-- Main three-panel layout -->
  <div class="main">
    <Sidebar
      {playgrounds}
      {selected}
      {dirty}
      on:select={(e) => selectPlayground(e.detail)}
      on:new={requestNewPlayground}
      on:rename={onRename}
      on:delete={onDelete}
      on:duplicate={onDuplicate}
    />

    <div class="editor-area">
      {#if selected}
        <Editor {code} on:change={(e) => onCodeChange(e.detail)} />
      {:else}
        <div class="empty-state">
          <p>No playground selected</p>
          <p class="hint">Pick one from the sidebar or <button class="link-btn" onclick={requestNewPlayground}>create a new one</button></p>
        </div>
      {/if}
    </div>

    <Output {output} />
  </div>
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }

  .toolbar {
    display: flex; align-items: center; gap: 10px;
    padding: 0 12px; height: 38px;
    background: var(--bg-panel); border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .app-name { font-weight: 600; flex: 1; font-size: 13px; }

  .status {
    font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em;
    color: var(--text-dim); min-width: 80px; text-align: center;
  }
  .status.error   { color: var(--red); }
  .status.running { color: var(--green); }

  .btn-run  { background: #166534; color: #86efac; padding: 5px 14px; font-size: 12px; font-weight: 600; }
  .btn-run:hover:not(:disabled) { background: #15803d; }
  .btn-stop { background: #7f1d1d; color: #fca5a5; padding: 5px 14px; font-size: 12px; font-weight: 600; }
  .btn-stop:hover { background: #991b1b; }

  .main { display: flex; flex: 1; overflow: hidden; }

  .editor-area { flex: 1; display: flex; overflow: hidden; }

  .empty-state {
    flex: 1; display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    gap: 8px; color: var(--text-dim);
  }
  .empty-state p { font-size: 14px; }
  .empty-state .hint { font-size: 12px; color: var(--text-dim2); }

  .link-btn {
    background: none; color: var(--accent); text-decoration: underline;
    padding: 0; font-size: 12px; display: inline;
  }
</style>
