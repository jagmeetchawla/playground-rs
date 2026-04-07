<script lang="ts">
  // Self-contained Rust toolchain status + repair modal.
  // Reachable from: Settings/Wizard "Install/Repair Toolchain…" button, and
  // the Rust menu's "Rust Toolchain…" item.
  // Detects rustup state, lets the user run install / set-default / add-component
  // fixes in-app, and streams the output via a Tauri Channel.
  import { invoke, Channel } from '@tauri-apps/api/core'
  import { open as shellOpen } from '@tauri-apps/plugin-shell'
  import { tick } from 'svelte'

  let { onclose, onfixed }: { onclose: () => void; onfixed?: () => void } = $props()

  type RustState = 'not_installed' | 'no_default' | 'missing_components' | 'healthy'
  type ToolchainStatus = {
    rust_state: RustState
    missing_components: string[]
    rustup: { installed: boolean; version: string | null }
    cargo: { installed: boolean; path: string; version: string | null }
    rustc: { installed: boolean; version: string | null }
    components: { rustfmt: boolean; clippy: boolean }
  }
  type FixAction =
    | { type: 'InstallRustup' }
    | { type: 'SetDefaultStable' }
    | { type: 'AddComponent'; name: string }

  let status = $state<ToolchainStatus | null>(null)
  let checking = $state(false)
  let fixState = $state<'idle' | 'running' | 'success' | 'error'>('idle')
  let fixOutput = $state<string[]>([])
  let fixActionLabel = $state('')
  let logEl = $state<HTMLPreElement | null>(null)
  let bodyEl = $state<HTMLDivElement | null>(null)

  // Auto-scroll log to bottom as new lines arrive.
  $effect(() => {
    fixOutput // dep
    if (logEl) logEl.scrollTop = logEl.scrollHeight
  })

  // When the output panel first appears (fixState idle → running), scroll the
  // modal body so the log is fully in view without the user having to scroll.
  $effect(() => {
    if (fixState === 'running' && bodyEl) {
      tick().then(() => {
        // Two rAFs: one for layout commit, one for paint, so scrollHeight is final.
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            bodyEl?.scrollTo({ top: bodyEl.scrollHeight, behavior: 'smooth' })
          })
        })
      })
    }
  })

  async function runCheck() {
    checking = true
    try {
      status = await invoke<ToolchainStatus>('check_toolchain')
    } catch { /* ignore */ }
    finally { checking = false }
  }

  $effect(() => {
    if (!status && !checking) runCheck()
  })

  async function runFix(action: FixAction, label: string) {
    fixState = 'running'
    fixOutput = []
    fixActionLabel = label
    const channel = new Channel<{ stream: string; line?: string; code?: number }>()
    channel.onmessage = (msg) => {
      if (msg.stream === 'complete') {
        fixState = (msg.code ?? -1) === 0 ? 'success' : 'error'
        if (fixState === 'success') {
          runCheck()
          // Notify parent so any underlying settings/wizard panel can refresh.
          onfixed?.()
        }
      } else if (msg.line !== undefined) {
        fixOutput = [...fixOutput, msg.line]
      }
    }
    try {
      await invoke('run_toolchain_fix', { action, onOutput: channel })
    } catch (e) {
      fixState = 'error'
      fixOutput = [...fixOutput, `Error: ${e}`]
    }
  }

  // Install multiple components sequentially, streaming output into one log.
  async function runFixComponents(components: string[]) {
    fixState = 'running'
    fixOutput = []
    fixActionLabel = `Installing ${components.join(' and ')}`
    let allOk = true
    for (const comp of components) {
      fixOutput = [...fixOutput, `\n── Installing ${comp} ──`]
      const ok = await new Promise<boolean>((resolve) => {
        const channel = new Channel<{ stream: string; line?: string; code?: number }>()
        channel.onmessage = (msg) => {
          if (msg.stream === 'complete') {
            resolve((msg.code ?? -1) === 0)
          } else if (msg.line !== undefined) {
            fixOutput = [...fixOutput, msg.line]
          }
        }
        invoke('run_toolchain_fix', {
          action: { type: 'AddComponent', name: comp },
          onOutput: channel,
        }).catch((e) => {
          fixOutput = [...fixOutput, `Error: ${e}`]
          resolve(false)
        })
      })
      if (!ok) { allOk = false; break }
    }
    fixState = allOk ? 'success' : 'error'
    if (allOk) {
      runCheck()
      onfixed?.()
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape' && fixState !== 'running') onclose()
  }
</script>

<svelte:window onkeydown={handleKey} />

<div class="backdrop" onclick={() => fixState !== 'running' && onclose()} aria-hidden="true"></div>

<div class="modal" role="dialog" aria-modal="true" aria-label="Rust Toolchain">
  <div class="modal-header">
    <div class="header-left">
      <span class="rs-badge">RS</span>
      <span class="modal-title">Rust Toolchain</span>
    </div>
    <button class="close-btn" onclick={onclose} disabled={fixState === 'running'} aria-label="Close">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  <div class="modal-body" bind:this={bodyEl}>
    {#if checking && !status}
      <div class="checking">
        <div class="spinner"></div>
        <span>Detecting toolchain…</span>
      </div>
    {:else if status}
      <!-- Status overview -->
      <div class="status-card" class:healthy={status.rust_state === 'healthy'} class:unhealthy={status.rust_state !== 'healthy'}>
        {#if status.rust_state === 'healthy'}
          <div class="status-headline ok">● Rust toolchain is healthy</div>
          <p class="status-sub">Everything is installed and ready to use.</p>
        {:else if status.rust_state === 'not_installed'}
          <div class="status-headline missing">○ Rust is not installed</div>
          <p class="status-sub">Install rustup, cargo, and the stable toolchain to start writing Rust.</p>
        {:else if status.rust_state === 'no_default'}
          <div class="status-headline missing">○ No default toolchain</div>
          <p class="status-sub">rustup is installed, but no default toolchain is selected. This often happens after moving <code>~/.rustup</code>.</p>
        {:else if status.rust_state === 'missing_components'}
          <div class="status-headline warn">◐ Missing components</div>
          <p class="status-sub">Rust is installed, but {status.missing_components.join(' and ')} {status.missing_components.length === 1 ? 'is' : 'are'} missing. {status.missing_components.includes('rustfmt') ? 'rustfmt enables auto-formatting.' : ''} {status.missing_components.includes('clippy') ? 'clippy powers live error checking.' : ''}</p>
        {/if}
      </div>

      <!-- Detail grid — hidden during/after a fix so the log gets the room -->
      {#if fixState === 'idle'}
      <div class="detail-grid">
        <div class="detail-row">
          <span class="detail-icon" class:ok={status.rustup.installed} class:missing={!status.rustup.installed}>{status.rustup.installed ? '●' : '○'}</span>
          <span class="detail-label">rustup</span>
          <span class="detail-value">{status.rustup.installed ? (status.rustup.version ?? 'installed') : 'not found'}</span>
        </div>
        <div class="detail-row">
          <span class="detail-icon" class:ok={status.cargo.installed} class:missing={!status.cargo.installed}>{status.cargo.installed ? '●' : '○'}</span>
          <span class="detail-label">cargo</span>
          <span class="detail-value">{status.cargo.installed ? (status.cargo.version ?? 'installed') : 'not found'}</span>
        </div>
        <div class="detail-row">
          <span class="detail-icon" class:ok={status.rustc.installed} class:missing={!status.rustc.installed}>{status.rustc.installed ? '●' : '○'}</span>
          <span class="detail-label">rustc</span>
          <span class="detail-value">{status.rustc.installed ? (status.rustc.version ?? 'installed') : 'not found'}</span>
        </div>
        <div class="detail-row">
          <span class="detail-icon" class:ok={status.components.rustfmt} class:missing={!status.components.rustfmt}>{status.components.rustfmt ? '●' : '○'}</span>
          <span class="detail-label">rustfmt</span>
          <span class="detail-value">{status.components.rustfmt ? 'installed' : 'not found'}</span>
        </div>
        <div class="detail-row">
          <span class="detail-icon" class:ok={status.components.clippy} class:missing={!status.components.clippy}>{status.components.clippy ? '●' : '○'}</span>
          <span class="detail-label">clippy</span>
          <span class="detail-value">{status.components.clippy ? 'installed' : 'not found'}</span>
        </div>
      </div>
      {/if}

      <!-- Fix actions — hidden during/after a fix -->
      {#if fixState === 'idle' && status.rust_state !== 'healthy'}
        <div class="fix-actions">
          {#if status.rust_state === 'not_installed'}
            <button class="fix-btn primary" disabled={fixState === 'running'} onclick={() => runFix({ type: 'InstallRustup' }, 'Installing Rust toolchain')}>
              {fixState === 'running' ? 'Installing…' : 'Install Rust'}
            </button>
            <p class="fix-hint">Runs the official installer from <button class="link-btn" onclick={() => shellOpen('https://rustup.rs')}>rustup.rs</button>. May take a few minutes.</p>
          {:else if status.rust_state === 'no_default'}
            <button class="fix-btn primary" disabled={fixState === 'running'} onclick={() => runFix({ type: 'SetDefaultStable' }, 'Setting default toolchain')}>
              {fixState === 'running' ? 'Setting default…' : 'Set default to stable'}
            </button>
          {:else if status.rust_state === 'missing_components'}
            <div class="fix-row">
              {#if status.missing_components.length > 1}
                <button class="fix-btn primary" disabled={fixState === 'running'} onclick={() => runFixComponents(status.missing_components)}>
                  Install {status.missing_components.join(' & ')}
                </button>
              {/if}
              {#each status.missing_components as comp}
                <button class="fix-btn" class:primary={status.missing_components.length === 1} disabled={fixState === 'running'} onclick={() => runFix({ type: 'AddComponent', name: comp }, `Installing ${comp}`)}>
                  Install {comp}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Output panel — always visible while running/done so user can watch progress -->
      {#if fixState !== 'idle'}
        <div class="fix-output">
          <div class="fix-header">
            {#if fixState === 'running'}
              <span class="fix-status running">{fixActionLabel}…</span>
            {:else if fixState === 'success'}
              <span class="fix-status ok">✓ {fixActionLabel} — done</span>
            {:else}
              <span class="fix-status err">✗ {fixActionLabel} — failed</span>
            {/if}
          </div>
          <pre class="fix-log" bind:this={logEl}>{fixOutput.join('\n') || '(starting…)'}</pre>
        </div>
      {/if}
    {/if}
  </div>

  <div class="modal-footer">
    <button class="btn btn-secondary" onclick={runCheck} disabled={checking || fixState === 'running'}>Re-check</button>
    <button class="btn btn-primary" onclick={onclose} disabled={fixState === 'running'}>Done</button>
  </div>
</div>

<style>
  /* Sits above ToolchainWizard (settings) so it can be opened from there. */
  .backdrop {
    position: fixed; inset: 0; z-index: 399;
    background: rgba(0,0,0,0.55);
    backdrop-filter: blur(2px);
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
    background: var(--rust-orange); color: #fff;
    border-radius: 3px; padding: 2px 4px; line-height: 1.3;
  }
  .modal-title { font-size: 14px; font-weight: 600; color: var(--text); }
  .close-btn {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%; color: var(--text-tertiary);
    background: none; border: none; cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }
  .close-btn:hover:not(:disabled) { background: var(--bg-hover); color: var(--text); }
  .close-btn:disabled { opacity: 0.4; cursor: default; }

  .modal-body {
    flex: 1; overflow-y: auto;
    padding: 16px 20px 20px;
    display: flex; flex-direction: column; gap: 14px;
  }

  .checking {
    display: flex; align-items: center; gap: 12px;
    padding: 30px 0;
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

  .status-card {
    padding: 12px 14px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg-input);
    display: flex; flex-direction: column; gap: 4px;
  }
  .status-card.healthy { border-color: rgba(46, 160, 67, 0.4); }
  .status-card.unhealthy { border-color: rgba(212, 32, 32, 0.4); }
  .status-headline { font-size: 13px; font-weight: 600; }
  .status-headline.ok { color: var(--green); }
  .status-headline.missing { color: var(--red, #d42020); }
  .status-headline.warn { color: #e8a820; }
  .status-sub {
    font-size: 11px; color: var(--text-secondary);
    margin: 2px 0 0 0; line-height: 1.4;
  }
  .status-sub code {
    font-family: var(--font-mono); font-size: 10px;
    background: rgba(0,0,0,0.25);
    padding: 1px 4px; border-radius: 3px;
  }

  .detail-grid {
    display: flex; flex-direction: column; gap: 2px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 10px;
  }
  .detail-row {
    display: flex; align-items: center; gap: 10px;
    padding: 4px 6px; border-radius: 4px; font-size: 12px;
  }
  .detail-icon { font-size: 8px; width: 12px; text-align: center; }
  .detail-icon.ok { color: var(--green); }
  .detail-icon.missing { color: var(--red, #d42020); }
  .detail-label {
    font-family: var(--font-mono); font-weight: 600;
    color: var(--text-secondary); width: 70px; flex-shrink: 0;
  }
  .detail-value {
    font-family: var(--font-mono); color: var(--text-tertiary);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }

  .fix-actions {
    display: flex; flex-direction: column; gap: 8px;
    align-items: flex-start;
  }
  .fix-row {
    display: flex; flex-direction: row; flex-wrap: wrap; gap: 8px;
    align-items: center;
  }
  .fix-btn {
    font-size: 12px; font-weight: 500;
    padding: 8px 16px; border-radius: 6px;
    border: 1px solid var(--border-strong);
    background: var(--bg-input); color: var(--text);
    cursor: pointer; transition: filter 0.1s, background 0.1s;
  }
  .fix-btn.primary {
    border-color: var(--accent);
    background: var(--accent); color: white;
  }
  .fix-btn:hover:not(:disabled) { filter: brightness(1.1); }
  .fix-btn:not(.primary):hover:not(:disabled) { background: var(--bg-hover); }
  .fix-btn:disabled { opacity: 0.6; cursor: default; }
  .fix-hint {
    font-size: 10px; color: var(--text-tertiary); margin: 0;
  }
  .link-btn {
    background: none; border: none; padding: 0;
    color: var(--accent); font-size: 10px;
    text-decoration: underline; cursor: pointer;
  }
  .link-btn:hover { filter: brightness(1.2); }

  .fix-output {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-input);
    overflow: hidden;
    display: flex; flex-direction: column;
  }
  .fix-header {
    padding: 8px 12px;
    font-size: 12px;
    border-bottom: 1px solid var(--border);
  }
  .fix-status.running { color: var(--text-secondary); }
  .fix-status.ok { color: var(--green); }
  .fix-status.err { color: var(--red, #d44); }
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
  }
  .btn:disabled { opacity: 0.5; cursor: default; }
  .btn-primary {
    background: var(--accent); color: white;
    border: 1px solid var(--accent);
  }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-secondary {
    background: var(--bg-input); color: var(--text);
    border: 1px solid var(--border);
  }
  .btn-secondary:hover:not(:disabled) { background: var(--bg-hover); border-color: var(--border-strong); }
</style>
