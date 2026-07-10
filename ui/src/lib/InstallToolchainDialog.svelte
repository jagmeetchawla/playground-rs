<script lang="ts">
  // v0.4+ modal for installing a Rust toolchain by channel (stable/beta/nightly)
  // or specific version. Invokes run_toolchain_fix with the InstallToolchain
  // action variant added to ToolchainFixAction in v0.4. Streams rustup's output
  // to a scrolling log via the same Channel pattern the ToolchainFixWizard uses
  // for InstallRustup, UpdateRust, and AddComponent.
  //
  // Wired from App.svelte via showInstallToolchain state (null = closed, string
  // = preferred version to pre-fill from the "install newer stable?" hint).

  import { invoke, Channel } from '@tauri-apps/api/core'

  type ChannelKind = 'stable' | 'beta' | 'nightly' | 'specific'

  let {
    preferredVersion,
    onclose,
    oninstalled,
  }: {
    /** Pre-fill hint. Empty string when opened without a preference; a semver
        string like "1.96.0" when opened from the picker's upgrade hint. */
    preferredVersion: string
    onclose: () => void
    /** Called on successful install, with the toolchain name that was
        installed (e.g. "stable", "nightly", "1.90.0"). Parent should
        typically set it active and refresh the pill — the user's mental
        model is "install this and use it", not "install this and leave it
        sitting there". Non-blocking. */
    oninstalled: (name: string) => Promise<void>
  } = $props()

  // If we opened with a preferred version, jump straight to "specific version"
  // radio with that value; otherwise default to stable.
  let selectedChannel: ChannelKind = $state(preferredVersion ? 'specific' : 'stable')
  let specificVersion: string = $state(preferredVersion)

  let installing = $state(false)
  let output: string[] = $state([])
  let complete: 'success' | 'error' | null = $state(null)
  let logEl: HTMLPreElement | null = $state(null)

  // The name we pass to `rustup toolchain install`. Channels are their bare
  // name; "specific" uses the raw input string (rustup validates format).
  let toolchainName = $derived(
    selectedChannel === 'specific' ? specificVersion.trim() : selectedChannel
  )

  let canInstall = $derived(
    !installing &&
    complete !== 'success' &&
    toolchainName.length > 0
  )

  async function runInstall() {
    if (!canInstall) return
    installing = true
    output = []
    complete = null

    const ch = new Channel<{ stream: string; line?: string; code?: number }>()
    ch.onmessage = (msg) => {
      if (msg.stream === 'complete') {
        installing = false
        complete = (msg.code ?? -1) === 0 ? 'success' : 'error'
        if (complete === 'success') {
          // Fire and forget — parent's switch-and-refresh shouldn't block
          // our success UI transition. Any error there is ignored because
          // the install itself succeeded and the app is still functional
          // even if the pill hasn't visually updated yet.
          oninstalled(toolchainName).catch(() => {})
        }
      } else if (msg.line !== undefined) {
        output = [...output, msg.line]
        // Auto-scroll to bottom on new output
        queueMicrotask(() => {
          if (logEl) logEl.scrollTop = logEl.scrollHeight
        })
      }
    }

    try {
      await invoke('run_toolchain_fix', {
        action: { type: 'InstallToolchain', name: toolchainName },
        onOutput: ch,
      })
    } catch (e) {
      installing = false
      complete = 'error'
      output = [...output, `\nError: ${e}`]
    }
  }

  function requestClose() {
    // Guard: don't allow close while an install is in flight. Killing the
    // rustup subprocess mid-download can leave things in a weird partial
    // state, and the user can't undo that from inside the app. They can
    // still force-quit the app if they really need to bail.
    if (installing) return
    onclose()
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && !installing) {
      onclose()
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="modal-backdrop" onclick={requestClose} aria-hidden="true"></div>
<div class="modal" role="dialog" aria-modal="true" aria-label="Install Rust toolchain">
  <div class="modal-header">
    <div class="header-left">
      <span class="rs-badge">RS</span>
      <span class="modal-title">Install Rust toolchain</span>
    </div>
    <button class="close-btn" onclick={requestClose} disabled={installing} aria-label="Close">×</button>
  </div>

  <div class="modal-body">
    {#if complete === 'success'}
      <div class="result-panel result-success">
        <span class="result-mark">✓</span>
        <div class="result-text">
          <div class="result-title">Installed {toolchainName}</div>
          <div class="result-sub">Available in the toolchain picker now.</div>
        </div>
      </div>
      <pre class="fix-log" bind:this={logEl}>{output.join('\n')}</pre>

    {:else if complete === 'error'}
      <div class="result-panel result-error">
        <span class="result-mark">✗</span>
        <div class="result-text">
          <div class="result-title">Install failed</div>
          <div class="result-sub">Check the output below for details.</div>
        </div>
      </div>
      <pre class="fix-log" bind:this={logEl}>{output.join('\n')}</pre>

    {:else if installing}
      <div class="installing-row">
        <div class="spinner"></div>
        <span>Installing <code>{toolchainName}</code>…</span>
      </div>
      <pre class="fix-log" bind:this={logEl}>{output.join('\n') || '(starting…)'}</pre>

    {:else}
      <div class="section-label">Choose a toolchain to install</div>

      <div class="radio-group">
        <label class="radio-row">
          <input type="radio" bind:group={selectedChannel} value="stable" />
          <div class="radio-text">
            <span class="radio-label">stable</span>
            <span class="radio-sub">Current stable release channel</span>
          </div>
        </label>

        <label class="radio-row">
          <input type="radio" bind:group={selectedChannel} value="beta" />
          <div class="radio-text">
            <span class="radio-label">beta</span>
            <span class="radio-sub">Preview of the next stable release</span>
          </div>
        </label>

        <label class="radio-row">
          <input type="radio" bind:group={selectedChannel} value="nightly" />
          <div class="radio-text">
            <span class="radio-label">nightly</span>
            <span class="radio-sub">Latest unstable features from master</span>
          </div>
        </label>

        <label class="radio-row">
          <input type="radio" bind:group={selectedChannel} value="specific" />
          <div class="radio-text">
            <span class="radio-label">specific version</span>
            <input
              type="text"
              class="version-input"
              placeholder="1.90.0"
              bind:value={specificVersion}
              onfocus={() => { selectedChannel = 'specific' }}
            />
          </div>
        </label>
      </div>

      {#if preferredVersion}
        <div class="hint-note">
          Suggested from the picker: <code>{preferredVersion}</code>
        </div>
      {/if}
    {/if}
  </div>

  <div class="modal-footer">
    {#if complete === 'success'}
      <button class="btn btn-primary" onclick={requestClose}>Done</button>
    {:else}
      <button class="btn btn-secondary" onclick={requestClose} disabled={installing}>
        {complete === 'error' ? 'Close' : 'Cancel'}
      </button>
      {#if complete !== 'error'}
        <button class="btn btn-primary" onclick={runInstall} disabled={!canInstall}>
          {installing ? 'Installing…' : 'Install'}
        </button>
      {/if}
    {/if}
  </div>
</div>

<style>
  /* Styles mirror ToolchainFixWizard.svelte so the two dialogs look
     consistent. If we later extract shared modal CSS, both should move
     together. */

  .modal-backdrop {
    position: fixed; inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 399;
  }
  .modal {
    position: fixed;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    z-index: 400;
    width: min(680px, calc(100vw - 40px));
    max-height: calc(100vh - 80px);
    display: flex; flex-direction: column;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    box-shadow: 0 24px 80px rgba(0,0,0,0.7), 0 4px 16px rgba(0,0,0,0.4);
    overflow: hidden;
  }
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .header-left { display: flex; align-items: center; gap: 8px; }
  .rs-badge {
    font-size: 8px; font-weight: 800; letter-spacing: 0.04em;
    background: var(--rust-orange, var(--accent)); color: #fff;
    border-radius: 3px; padding: 2px 4px; line-height: 1.3;
  }
  .modal-title { font-size: 14px; font-weight: 600; color: var(--text); }
  .close-btn {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%; color: var(--text-tertiary);
    background: none; border: none; cursor: pointer;
    font-size: 18px; line-height: 1;
    transition: background 0.1s, color 0.1s;
  }
  .close-btn:hover:not(:disabled) { background: var(--bg-hover); color: var(--text); }
  .close-btn:disabled { opacity: 0.4; cursor: default; }

  .modal-body {
    flex: 1; overflow-y: auto;
    padding: 16px 20px 20px;
    display: flex; flex-direction: column; gap: 14px;
  }

  .section-label {
    font-size: 11px; font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase; letter-spacing: 0.5px;
  }

  .radio-group { display: flex; flex-direction: column; gap: 4px; }

  .radio-row {
    display: flex; align-items: flex-start; gap: 10px;
    padding: 8px 10px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.1s;
  }
  .radio-row:hover { background: var(--bg-hover); }
  .radio-row input[type="radio"] { margin-top: 3px; accent-color: var(--accent); }
  .radio-text { display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0; }
  .radio-label { font-size: 13px; color: var(--text); font-weight: 500; }
  .radio-sub { font-size: 11px; color: var(--text-tertiary); }

  .version-input {
    margin-top: 4px;
    padding: 5px 8px;
    font-size: 12px; font-family: var(--font-mono);
    color: var(--text);
    background: var(--bg-input, rgba(0,0,0,0.3));
    border: 1px solid var(--border);
    border-radius: 4px;
    width: 100%; max-width: 180px;
  }
  .version-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .hint-note {
    font-size: 11px;
    color: var(--text-secondary);
    padding: 6px 10px;
    background: color-mix(in srgb, var(--accent) 8%, transparent);
    border-radius: 4px;
  }
  .hint-note code {
    font-family: var(--font-mono);
    color: var(--accent);
  }

  .installing-row {
    display: flex; align-items: center; gap: 10px;
    color: var(--text-secondary); font-size: 13px;
  }
  .installing-row code {
    font-family: var(--font-mono);
    color: var(--text);
    background: rgba(0,0,0,0.3);
    padding: 1px 5px; border-radius: 3px;
  }
  .spinner {
    width: 14px; height: 14px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .result-panel {
    display: flex; gap: 12px; align-items: flex-start;
    padding: 12px 14px;
    border-radius: 6px;
  }
  .result-success {
    background: color-mix(in srgb, #3fbf70 12%, transparent);
    border: 1px solid color-mix(in srgb, #3fbf70 30%, transparent);
  }
  .result-error {
    background: color-mix(in srgb, #f44 12%, transparent);
    border: 1px solid color-mix(in srgb, #f44 30%, transparent);
  }
  .result-mark {
    font-size: 18px; line-height: 1; font-weight: 700;
    color: var(--text);
  }
  .result-success .result-mark { color: #3fbf70; }
  .result-error .result-mark { color: #f44; }
  .result-title { font-size: 13px; font-weight: 600; color: var(--text); }
  .result-sub { font-size: 11px; color: var(--text-secondary); margin-top: 2px; }

  .fix-log {
    margin: 0;
    height: 280px;
    overflow-y: auto;
    background: rgba(0,0,0,0.35);
    padding: 10px 12px;
    font-family: var(--font-mono); font-size: 11px;
    line-height: 1.5;
    color: var(--text-secondary);
    white-space: pre-wrap; word-break: break-all;
    border-radius: 4px;
  }

  .modal-footer {
    display: flex; justify-content: flex-end; gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }
  .btn {
    font-size: 12px; font-weight: 500;
    padding: 7px 14px; border-radius: 6px;
    cursor: pointer; transition: background 0.15s, border-color 0.15s;
    font-family: inherit;
  }
  .btn:disabled { opacity: 0.5; cursor: default; }
  .btn-primary {
    background: var(--accent); color: white;
    border: 1px solid var(--accent);
  }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-secondary {
    background: var(--bg-input, transparent); color: var(--text);
    border: 1px solid var(--border);
  }
  .btn-secondary:hover:not(:disabled) { background: var(--bg-hover); border-color: var(--border-strong); }
</style>
