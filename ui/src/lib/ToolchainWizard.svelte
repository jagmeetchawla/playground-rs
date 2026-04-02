<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'

  let {
    onclose,
  }: {
    onclose: () => void
  } = $props()

  type ToolchainStatus = {
    wizard_completed: boolean
    all_good: boolean
    rustup: { installed: boolean; version: string | null }
    cargo: { installed: boolean; path: string; version: string | null }
    rustc: { installed: boolean; version: string | null }
    active_toolchain: string | null
    installed_toolchains: string[]
    components: { rustfmt: boolean; clippy: boolean }
  }

  let status = $state<ToolchainStatus | null>(null)
  let checking = $state(true)
  let error = $state<string | null>(null)

  async function runCheck() {
    checking = true
    error = null
    try {
      status = await invoke<ToolchainStatus>('check_toolchain')
    } catch (e) {
      error = String(e)
    } finally {
      checking = false
    }
  }

  async function finish() {
    await invoke('complete_wizard')
    onclose()
  }

  // Run check on mount
  runCheck()
</script>

<div class="backdrop" onclick={onclose} aria-hidden="true"></div>

<div class="modal" role="dialog" aria-modal="true" aria-label="Toolchain Setup">
  <div class="modal-header">
    <div class="header-left">
      <span class="rs-badge">RS</span>
      <span class="modal-title">Toolchain Setup</span>
    </div>
    <button class="close-btn" onclick={onclose} aria-label="Close">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  <div class="modal-body">
    {#if checking}
      <div class="checking">
        <div class="spinner"></div>
        <span>Detecting Rust toolchain...</span>
      </div>
    {:else if error}
      <div class="error-box">
        <p>Failed to check toolchain: {error}</p>
        <button class="btn btn-primary" onclick={runCheck}>Retry</button>
      </div>
    {:else if status}
      <!-- Status overview: green = all good, yellow = partial, red = nothing -->
      {@const nothingFound = !status.rustup.installed && !status.cargo.installed && !status.rustc.installed}
      {#if status.all_good}
        <div class="status-banner good">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
            <circle cx="10" cy="10" r="9" stroke="currentColor" stroke-width="1.5"/>
            <path d="M6 10l3 3 5-6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
          </svg>
          <span>Rust toolchain is ready</span>
        </div>
      {:else if nothingFound}
        <div class="status-banner bad">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
            <circle cx="10" cy="10" r="9" stroke="currentColor" stroke-width="1.5"/>
            <path d="M7 7l6 6M13 7l-6 6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
          </svg>
          <span>Rust is not installed</span>
        </div>
      {:else}
        <div class="status-banner warn">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
            <path d="M10 2L18.66 17H1.34L10 2z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" fill="none"/>
            <path d="M10 8v4M10 14v1" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
          </svg>
          <span>Rust toolchain is incomplete</span>
        </div>
      {/if}

      <!-- Detail rows -->
      <div class="detail-grid">
        <div class="detail-row">
          <span class="detail-icon" class:ok={status.rustup.installed} class:missing={!status.rustup.installed}>
            {status.rustup.installed ? '●' : '○'}
          </span>
          <span class="detail-label">rustup</span>
          <span class="detail-value">
            {#if status.rustup.installed}
              {status.rustup.version ?? 'installed'}
            {:else}
              not found
            {/if}
          </span>
        </div>

        <div class="detail-row">
          <span class="detail-icon" class:ok={status.cargo.installed} class:missing={!status.cargo.installed}>
            {status.cargo.installed ? '●' : '○'}
          </span>
          <span class="detail-label">cargo</span>
          <span class="detail-value">
            {#if status.cargo.installed}
              {status.cargo.version ?? 'installed'}
            {:else}
              not found
            {/if}
          </span>
        </div>

        <div class="detail-row">
          <span class="detail-icon" class:ok={status.rustc.installed} class:missing={!status.rustc.installed}>
            {status.rustc.installed ? '●' : '○'}
          </span>
          <span class="detail-label">rustc</span>
          <span class="detail-value">
            {#if status.rustc.installed}
              {status.rustc.version ?? 'installed'}
            {:else}
              not found
            {/if}
          </span>
        </div>

        {#if status.active_toolchain}
          <div class="detail-row">
            <span class="detail-icon ok">●</span>
            <span class="detail-label">toolchain</span>
            <span class="detail-value">{status.active_toolchain}</span>
          </div>
        {/if}

        <div class="detail-row">
          <span class="detail-icon" class:ok={status.components.rustfmt} class:missing={!status.components.rustfmt}>
            {status.components.rustfmt ? '●' : '○'}
          </span>
          <span class="detail-label">rustfmt</span>
          <span class="detail-value">{status.components.rustfmt ? 'installed' : 'not found'}</span>
        </div>

        <div class="detail-row">
          <span class="detail-icon" class:ok={status.components.clippy} class:missing={!status.components.clippy}>
            {status.components.clippy ? '●' : '○'}
          </span>
          <span class="detail-label">clippy</span>
          <span class="detail-value">{status.components.clippy ? 'installed' : 'not found'}</span>
        </div>
      </div>

      {#if status.cargo.installed && status.cargo.path}
        <div class="path-row">
          <span class="path-label">cargo path</span>
          <code class="path-value">{status.cargo.path}</code>
        </div>
      {/if}

      {#if !status.all_good}
        <div class="install-section">
          <p class="install-text">Install Rust using the official installer:</p>
          <code class="install-cmd">curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh</code>
          <p class="install-hint">Run this in Terminal, then click "Re-check" below.</p>
        </div>
      {/if}

      {#if !status.components.rustfmt || !status.components.clippy}
        <div class="install-section">
          <p class="install-text">Install missing components:</p>
          {#if !status.components.rustfmt}
            <code class="install-cmd">rustup component add rustfmt</code>
          {/if}
          {#if !status.components.clippy}
            <code class="install-cmd">rustup component add clippy</code>
          {/if}
        </div>
      {/if}
    {/if}
  </div>

  <div class="modal-footer">
    {#if !checking}
      {#if !status?.all_good}
        <button class="btn btn-secondary" onclick={runCheck}>Re-check</button>
      {/if}
      <button class="btn btn-primary" onclick={finish}>
        {status?.all_good ? 'Done' : 'Continue Anyway'}
      </button>
    {/if}
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
  .modal-title { font-size: 14px; font-weight: 700; color: var(--text); }
  .close-btn {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%; color: var(--text-tertiary);
    transition: background 0.1s, color 0.1s;
  }
  .close-btn:hover { background: var(--bg-hover); color: var(--text); }

  .modal-body {
    flex: 1; overflow-y: auto;
    padding: 20px;
    display: flex; flex-direction: column; gap: 16px;
  }

  .modal-footer {
    display: flex; align-items: center; justify-content: flex-end; gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }

  .btn {
    font-size: 12px; font-weight: 600;
    padding: 5px 14px; border-radius: 6px;
    cursor: pointer; transition: background 0.1s;
  }
  .btn-secondary {
    color: var(--text-secondary);
    background: rgba(255,255,255,0.06); border: 1px solid var(--border);
  }
  .btn-secondary:hover { background: rgba(255,255,255,0.1); border-color: var(--border-strong); }
  .btn-primary { color: #fff; background: var(--accent); border: 1px solid var(--accent); }
  .btn-primary:hover { filter: brightness(1.15); }

  /* Checking spinner */
  .checking {
    display: flex; align-items: center; gap: 12px;
    padding: 20px 0;
    color: var(--text-secondary); font-size: 13px;
    justify-content: center;
  }
  .spinner {
    width: 18px; height: 18px;
    border: 2px solid rgba(255,255,255,0.15);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .error-box {
    text-align: center; padding: 16px;
    color: var(--red); font-size: 13px;
  }
  .error-box .btn { margin-top: 12px; }

  /* Status banner */
  .status-banner {
    display: flex; align-items: center; gap: 10px;
    padding: 12px 14px;
    border-radius: 8px;
    font-size: 14px; font-weight: 600;
  }
  .status-banner.good {
    background: rgba(52, 199, 89, 0.12);
    border: 1px solid rgba(52, 199, 89, 0.3);
    color: var(--green);
  }
  .status-banner.warn {
    background: rgba(255, 159, 10, 0.12);
    border: 1px solid rgba(255, 159, 10, 0.3);
    color: #ff9f0a;
  }
  .status-banner.bad {
    background: rgba(255, 69, 58, 0.12);
    border: 1px solid rgba(255, 69, 58, 0.3);
    color: #ff453a;
  }

  /* Detail grid */
  .detail-grid {
    display: flex; flex-direction: column; gap: 2px;
  }
  .detail-row {
    display: flex; align-items: center; gap: 10px;
    padding: 6px 8px;
    border-radius: 4px;
    font-size: 12px;
  }
  .detail-row:hover { background: rgba(255,255,255,0.03); }

  .detail-icon { font-size: 8px; width: 12px; text-align: center; }
  .detail-icon.ok { color: var(--green); }
  .detail-icon.missing { color: var(--text-tertiary); }

  .detail-label {
    font-family: var(--font-mono); font-weight: 600;
    color: var(--text-secondary);
    width: 80px; flex-shrink: 0;
  }
  .detail-value {
    font-family: var(--font-mono);
    color: var(--text-tertiary);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }

  /* Cargo path */
  .path-row {
    display: flex; align-items: center; gap: 10px;
    padding: 6px 8px;
    font-size: 11px;
  }
  .path-label {
    color: var(--text-tertiary); font-size: 10px; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.05em;
    width: 80px; flex-shrink: 0; padding-left: 22px;
  }
  .path-value {
    color: var(--text-tertiary);
    font-family: var(--font-mono); font-size: 11px;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }

  /* Install section */
  .install-section {
    background: rgba(0,0,0,0.2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
  }
  .install-text {
    font-size: 12px; color: var(--text-secondary);
    margin: 0 0 8px 0;
  }
  .install-cmd {
    display: block;
    font-family: var(--font-mono); font-size: 11px;
    color: var(--text); background: rgba(0,0,0,0.3);
    border: 1px solid var(--border);
    border-radius: 4px; padding: 8px 10px;
    margin: 4px 0;
    user-select: all;
    word-break: break-all;
  }
  .install-hint {
    font-size: 11px; color: var(--text-tertiary);
    margin: 8px 0 0 0;
  }
</style>
