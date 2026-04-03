<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'

  export type Settings = {
    font_size: number
    font_family: string
    tab_size: number
    cargo_path: string
    theme: string
  }

  let {
    settings = $bindable(),
    onclose,
    onsave,
    onthemechange,
  }: {
    settings: Settings
    onclose: () => void
    onsave: (s: Settings) => void
    onthemechange?: (theme: string) => void
  } = $props()

  // Local draft so changes are only applied on save
  let draft: Settings = $state({ ...settings })
  const originalTheme = settings.theme

  // Clang toolchain info (read-only display)
  let clangPath: string = $state('')
  let clangInstalled: boolean = $state(false)
  let clangVersion: string = $state('')

  async function detectClang() {
    try {
      const tc = await invoke<any>('check_toolchain')
      if (tc.clang) {
        clangInstalled = tc.clang.installed ?? false
        clangPath = tc.clang.path ?? ''
        clangVersion = tc.clang.version ?? ''
      }
    } catch (_) { /* ignore */ }
  }
  detectClang()

  function setTheme(theme: string) {
    draft.theme = theme
    onthemechange?.(theme)
  }

  function cancel() {
    onthemechange?.(originalTheme)
    onclose()
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') cancel()
  }

  async function save() {
    try {
      await invoke('save_settings', { settings: draft })
      settings = { ...draft }
      onsave(draft)
      onclose()
    } catch (err) {
      console.error('Failed to save settings:', err)
    }
  }

  function reset() {
    draft = {
      font_size: 13,
      font_family: 'Menlo',
      tab_size: 4,
      cargo_path: draft.cargo_path, // keep cargo path as-is
      theme: 'system',
    }
  }

  const fontFamilies = [
    'Menlo',
    'Monaco',
    'SF Mono',
    'Courier New',
    'JetBrains Mono',
    'Fira Code',
    'Source Code Pro',
    'IBM Plex Mono',
  ]
</script>

<svelte:window onkeydown={handleKey} />

<!-- Backdrop -->
<div class="backdrop" onclick={cancel} aria-hidden="true"></div>

<div class="modal" role="dialog" aria-modal="true" aria-label="Settings">
  <div class="modal-header">
    <div class="header-left">
      <span class="rs-badge">RS</span>
      <span class="modal-title">Settings</span>
    </div>
    <button class="close-btn" onclick={cancel} aria-label="Close">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  <div class="modal-body">

    <!-- Editor section -->
    <section>
      <h2>Editor</h2>

      <div class="setting-row">
        <label for="font-size">Font Size</label>
        <div class="input-group">
          <input
            id="font-size"
            type="number"
            min="8"
            max="32"
            bind:value={draft.font_size}
          />
          <span class="unit">px</span>
        </div>
      </div>

      <div class="setting-row">
        <label for="font-family">Font Family</label>
        <select id="font-family" bind:value={draft.font_family}>
          {#each fontFamilies as f}
            <option value={f}>{f}</option>
          {/each}
        </select>
      </div>

      <div class="setting-row">
        <label for="tab-size">Tab Size</label>
        <select id="tab-size" bind:value={draft.tab_size}>
          <option value={2}>2 spaces</option>
          <option value={4}>4 spaces</option>
          <option value={8}>8 spaces</option>
        </select>
      </div>
    </section>

    <!-- Appearance section -->
    <section>
      <h2>Appearance</h2>

      <div class="setting-row">
        <label>Theme</label>
        <div class="segmented">
          <button
            class="seg-btn"
            class:seg-active={draft.theme === 'system'}
            onclick={() => setTheme('system')}
          >System</button>
          <button
            class="seg-btn"
            class:seg-active={draft.theme === 'light'}
            onclick={() => setTheme('light')}
          >Light</button>
          <button
            class="seg-btn"
            class:seg-active={draft.theme === 'dark'}
            onclick={() => setTheme('dark')}
          >Dark</button>
          <button
            class="seg-btn"
            class:seg-active={draft.theme === 'rust'}
            onclick={() => setTheme('rust')}
          >Rust</button>
          <button
            class="seg-btn"
            class:seg-active={draft.theme === 'seagreen'}
            onclick={() => setTheme('seagreen')}
          >C</button>
        </div>
      </div>
    </section>

    <!-- Toolchain section -->
    <section>
      <h2>Toolchain</h2>

      <div class="setting-row">
        <label for="cargo-path">Cargo Path</label>
        <input
          id="cargo-path"
          type="text"
          bind:value={draft.cargo_path}
          class="path-input"
          spellcheck="false"
        />
      </div>

      <div class="setting-row">
        <label>Clang</label>
        {#if clangInstalled}
          <span class="clang-version">{clangVersion || 'installed'}</span>
        {:else}
          <div class="toolchain-missing">
            <span class="missing-label">Xcode Command Line Tools not installed</span>
            <p class="missing-hint">C/C++ support requires clang. Run in Terminal:</p>
            <code class="install-cmd">xcode-select --install</code>
          </div>
        {/if}
      </div>
    </section>

    <!-- Preview -->
    <section class="preview-section">
      <h2>Preview</h2>
      <div
        class="preview-box"
        style="font-size: {draft.font_size}px; font-family: '{draft.font_family}', monospace;"
      >
        <span class="preview-kw">fn</span> <span class="preview-fn">main</span>() &#123;<br/>
        &nbsp;&nbsp;&nbsp;&nbsp;<span class="preview-mac">println!</span>(<span class="preview-str">"Hello, playground!"</span>);<br/>
        &#125;
      </div>
    </section>
  </div>

  <div class="modal-footer">
    <button class="btn btn-secondary" onclick={reset}>Reset Defaults</button>
    <div class="footer-right">
      <button class="btn btn-secondary" onclick={cancel}>Cancel</button>
      <button class="btn btn-primary" onclick={save}>Save</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0; z-index: 299;
    background: rgba(0,0,0,0.55);
    backdrop-filter: blur(2px);
  }

  .modal {
    position: fixed;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    z-index: 300;
    width: min(480px, calc(100vw - 40px));
    max-height: calc(100vh - 80px);
    display: flex; flex-direction: column;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    box-shadow: 0 24px 80px rgba(0,0,0,0.7), 0 4px 16px rgba(0,0,0,0.4);
    overflow: hidden;
  }

  /* ── Header ── */
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .header-left { display: flex; align-items: center; gap: 8px; }
  .rs-badge {
    font-size: 8px; font-weight: 800; letter-spacing: 0.04em;
    background: var(--rust-orange); color: #fff;
    border-radius: 3px; padding: 2px 4px; line-height: 1.3;
  }
  .modal-title {
    font-size: 14px; font-weight: 700; color: var(--text);
  }
  .close-btn {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%;
    color: var(--text-tertiary);
    transition: background 0.1s, color 0.1s;
  }
  .close-btn:hover { background: var(--bg-hover); color: var(--text); }

  /* ── Body ── */
  .modal-body {
    flex: 1; overflow-y: auto;
    padding: 20px 24px;
    display: flex; flex-direction: column; gap: 24px;
  }

  section { display: flex; flex-direction: column; gap: 10px; }

  h2 {
    font-size: 11px; font-weight: 700; letter-spacing: 0.06em;
    text-transform: uppercase; color: var(--rust-orange);
    margin: 0;
  }

  /* ── Setting rows ── */
  .setting-row {
    display: flex; align-items: center; justify-content: space-between;
    gap: 12px;
  }

  label {
    font-size: 13px; color: var(--text-secondary); font-weight: 500;
    flex-shrink: 0;
  }

  input[type="number"],
  select {
    font-family: var(--font-mono); font-size: 12px;
    background: var(--bg-input); color: var(--text);
    border: 1px solid var(--border); border-radius: var(--radius-xs);
    padding: 4px 8px; outline: none;
    min-width: 80px;
  }
  input[type="number"]:focus,
  select:focus,
  .path-input:focus {
    border-color: var(--accent);
  }

  .clang-version {
    font-family: var(--font-mono); font-size: 12px;
    color: var(--text-secondary);
  }

  .setting-row:has(.toolchain-missing) {
    align-items: flex-start;
  }
  .toolchain-missing {
    flex: 1; min-width: 0;
    display: flex; flex-direction: column; gap: 4px;
  }
  .missing-label {
    font-size: 12px; font-weight: 600; color: #ff9f0a;
  }
  .missing-hint {
    font-size: 11px; color: var(--text-tertiary); margin: 0;
  }
  .install-cmd {
    display: block;
    font-family: var(--font-mono); font-size: 11px;
    color: var(--text); background: rgba(0,0,0,0.3);
    border: 1px solid var(--border);
    border-radius: 4px; padding: 6px 8px;
    user-select: all;
  }

  select {
    min-width: 140px;
    cursor: pointer;
  }
  select option {
    background: var(--bg-elevated);
    color: var(--text);
  }

  .input-group {
    display: flex; align-items: center; gap: 4px;
  }
  .input-group input { width: 60px; text-align: right; }
  .unit { font-size: 11px; color: var(--text-tertiary); }

  .path-input {
    font-family: var(--font-mono); font-size: 11px;
    background: var(--bg-input); color: var(--text);
    border: 1px solid var(--border); border-radius: var(--radius-xs);
    padding: 4px 8px; outline: none;
    flex: 1; min-width: 0;
  }

  /* ── Segmented control ── */
  .segmented {
    display: flex;
    border: 1px solid var(--border);
    border-radius: var(--radius-xs);
    overflow: hidden;
  }
  .seg-btn {
    font-size: 11px; font-weight: 600;
    padding: 4px 12px;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-right: 1px solid var(--border);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }
  .seg-btn:last-child { border-right: none; }
  .seg-btn:hover { background: var(--bg-hover); }
  .seg-btn.seg-active {
    background: var(--accent);
    color: var(--accent-text);
  }

  /* ── Preview ── */
  .preview-section {
    background: var(--bg-sidebar);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 12px 14px;
  }
  .preview-box {
    line-height: 1.7;
    color: var(--text);
    white-space: pre;
  }
  .preview-kw  { color: #fc5fa3; }
  .preview-fn  { color: #67b7a4; }
  .preview-mac { color: #b281eb; }
  .preview-str { color: #fc6a5d; }

  /* ── Footer ── */
  .modal-footer {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }
  .footer-right { display: flex; gap: 8px; }

  .btn {
    font-size: 12px; font-weight: 600;
    padding: 5px 14px; border-radius: 6px;
    cursor: pointer; transition: background 0.1s, border-color 0.1s;
  }
  .btn-secondary {
    color: var(--text-secondary);
    background: var(--bg-input);
    border: 1px solid var(--border);
  }
  .btn-secondary:hover {
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }
  .btn-primary {
    color: #fff;
    background: var(--accent);
    border: 1px solid var(--accent);
  }
  .btn-primary:hover {
    filter: brightness(1.15);
  }
</style>
