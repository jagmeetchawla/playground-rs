<script lang="ts">
  // Self-contained Rust toolchain status + repair modal.
  // Reachable from: Wizard "Install Rust Toolchain…" button, Settings
  // "Repair Rust Toolchain…" button, and the Rust menu's "Rust Toolchain…" item.
  // Detects rustup state, lets the user run install / set-default / add-component
  // fixes in-app, and streams the output via a Tauri Channel.
  import { invoke, Channel } from '@tauri-apps/api/core'
  import { open as shellOpen } from '@tauri-apps/plugin-shell'
  import { tick } from 'svelte'

  let { onclose, onfixed }: { onclose: () => void; onfixed?: () => void } = $props()

  type RustState = 'clt_missing' | 'not_installed' | 'no_default' | 'outdated' | 'missing_components' | 'healthy'
  type ToolchainStatus = {
    rust_state: RustState
    missing_components: string[]
    xcode_clt: { installed: boolean; path: string | null }
    rustup: { installed: boolean; version: string | null }
    cargo: { installed: boolean; path: string; version: string | null }
    rustc: { installed: boolean; version: string | null; version_ok?: boolean; min_version?: string }
    components: { rustfmt: boolean; clippy: boolean }
  }
  type FixAction =
    | { type: 'InstallXcodeCLT' }
    | { type: 'InstallRustup' }
    | { type: 'SetDefaultStable' }
    | { type: 'AddComponent'; name: string }
    | { type: 'UpdateRust' }

  let status = $state<ToolchainStatus | null>(null)
  let checking = $state(false)
  let fixState = $state<'idle' | 'running' | 'success' | 'error'>('idle')
  let fixOutput = $state<string[]>([])
  let fixActionLabel = $state('')
  /** Track the action type so success UI can render action-specific guidance
   *  (e.g. a "find Apple's installer" callout for InstallXcodeCLT). */
  let fixActionType = $state<FixAction['type'] | null>(null)
  let logEl = $state<HTMLPreElement | null>(null)
  let bodyEl = $state<HTMLDivElement | null>(null)

  /** Two setup paths, chosen from the left sidebar when state ≠ healthy:
   *   guided — the existing in-app installer with streamed output
   *   manual — show raw Terminal commands for the user to run themselves
   *
   * Defaults to manual: most users on a fresh Mac who download a dev tool
   * are comfortable with Terminal, and showing commands first respects user
   * agency. Beginners who prefer guided can click "Help Me Install".
   */
  let setupMode = $state<'guided' | 'manual'>('manual')
  /** Track which command was most recently copied, to render a "Copied" flash. */
  let copiedCommand = $state<string | null>(null)
  /** Canonical rustup one-liner, shared by guided + manual flows. */
  const RUSTUP_INSTALL_CMD =
    `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable`
  const XCODE_CLT_CMD = 'xcode-select --install'

  async function copyCommand(cmd: string) {
    try {
      await navigator.clipboard.writeText(cmd)
      copiedCommand = cmd
      setTimeout(() => {
        if (copiedCommand === cmd) copiedCommand = null
      }, 1600)
    } catch {
      // Clipboard API can fail under insecure contexts; ignore silently.
    }
  }

  // ── Auto-polling for Xcode CLT install completion ────────────────────────
  // After we trigger `xcode-select --install`, the command itself exits
  // immediately. Apple's GUI installer takes 10-15 minutes in the background.
  // Rather than make the user click Re-check repeatedly, we poll the toolchain
  // status every few seconds and automatically advance the cascade when CLT
  // becomes available.
  let pollingForCLT = $state(false)
  let pollIntervalId: ReturnType<typeof setInterval> | null = null
  let pollSecondsElapsed = $state(0)
  const POLL_INTERVAL_MS = 5000          // 5 seconds — light overhead, snappy detection
  const MAX_POLL_SECONDS = 30 * 60       // 30 min cap — safety net if installer hangs

  function startCLTPolling() {
    if (pollIntervalId !== null) return
    pollingForCLT = true
    pollSecondsElapsed = 0
    pollIntervalId = setInterval(async () => {
      pollSecondsElapsed += POLL_INTERVAL_MS / 1000
      try {
        const newStatus = await invoke<ToolchainStatus>('check_toolchain')
        status = newStatus
        if (newStatus.xcode_clt.installed) {
          stopCLTPolling()
          // Reset fix UI so the next state's actions render naturally.
          // The cascade will pick up rust_state's new value (likely
          // not_installed) and surface the next fix.
          fixState = 'idle'
          fixActionType = null
          fixOutput = []
          fixActionLabel = ''
          onfixed?.()
        } else if (pollSecondsElapsed >= MAX_POLL_SECONDS) {
          stopCLTPolling()
        }
      } catch {
        // Ignore individual poll errors; keep polling. The user can also
        // manually click Re-check if they suspect a problem.
      }
    }, POLL_INTERVAL_MS)
  }

  function stopCLTPolling() {
    if (pollIntervalId !== null) {
      clearInterval(pollIntervalId)
      pollIntervalId = null
    }
    pollingForCLT = false
  }

  function formatElapsed(seconds: number): string {
    const m = Math.floor(seconds / 60)
    const s = seconds % 60
    return `${m}:${s.toString().padStart(2, '0')}`
  }

  // Cleanup polling when the modal unmounts (parent unmounts via {#if})
  $effect(() => {
    return () => stopCLTPolling()
  })

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
    fixActionType = action.type
    const channel = new Channel<{ stream: string; line?: string; code?: number }>()
    channel.onmessage = (msg) => {
      if (msg.stream === 'complete') {
        fixState = (msg.code ?? -1) === 0 ? 'success' : 'error'
        if (fixState === 'success') {
          if (action.type === 'InstallXcodeCLT') {
            // CLT install command exits in seconds but Apple's installer runs
            // for 10-15 min. Poll check_toolchain in the background and
            // auto-advance when CLT is detected — no need for the user to
            // click Re-check repeatedly.
            startCLTPolling()
          } else {
            runCheck()
            // Notify parent so any underlying settings/wizard panel can refresh.
            onfixed?.()
          }
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
    fixActionType = 'AddComponent'
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

  <!-- Reusable "Need help with the Rust toolchain?" footer, rendered in
       every FixWizard state (healthy, guided unhealthy, manual unhealthy)
       so users can jump to external resources regardless of toolchain
       status. Same content as the settings/wizard toolchain panel. -->
  {#snippet helpLinks()}
    <div class="help-links">
      <div class="help-links-title">Need help with the Rust toolchain?</div>
      <ul class="help-links-list">
        <li><button class="link-btn" onclick={() => shellOpen('https://rustup.rs')}>rustup.rs</button> — official installer</li>
        <li><button class="link-btn" onclick={() => shellOpen('https://www.rust-lang.org/learn/get-started')}>rust-lang.org</button> — getting started guide</li>
        <li><button class="link-btn" onclick={() => shellOpen('https://users.rust-lang.org')}>users.rust-lang.org</button> — friendly Q&A forum</li>
        <li><button class="link-btn" onclick={() => shellOpen('https://www.reddit.com/r/rust')}>r/rust</button> — Reddit community</li>
        <li><button class="link-btn" onclick={() => shellOpen('https://github.com/rust-lang/rustup/issues')}>rustup issues</button> — for installer bugs</li>
        <li><button class="link-btn" onclick={() => shellOpen('https://github.com/jagmeetchawla/rustic-playground/issues')}>Report a bug</button> — for issues with this app</li>
      </ul>
    </div>
  {/snippet}

  <div class="modal-body" bind:this={bodyEl}>
    {#if checking && !status}
      <div class="checking">
        <div class="spinner"></div>
        <span>Detecting toolchain…</span>
      </div>
    {:else if status && status.rust_state === 'healthy'}
      <!-- Centered healthy view — no split layout needed, nothing to fix -->
      <div class="status-card healthy">
        <div class="status-headline ok">● Rust toolchain is healthy</div>
        <p class="status-sub">Everything is installed and ready to use.</p>
      </div>
      <div class="detail-grid">
        <div class="detail-row">
          <span class="detail-icon ok">●</span>
          <span class="detail-label">Xcode CLT</span>
          <span class="detail-value">{status.xcode_clt.path ?? 'installed'}</span>
        </div>
        <div class="detail-row">
          <span class="detail-icon ok">●</span>
          <span class="detail-label">rustup</span>
          <span class="detail-value">{status.rustup.version ?? 'installed'}</span>
        </div>
        <div class="detail-row">
          <span class="detail-icon ok">●</span>
          <span class="detail-label">cargo</span>
          <span class="detail-value">{status.cargo.version ?? 'installed'}</span>
        </div>
        <div class="detail-row">
          <span class="detail-icon ok">●</span>
          <span class="detail-label">rustc</span>
          <span class="detail-value">{status.rustc.version ?? 'installed'}</span>
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
      {@render helpLinks()}
    {:else if status}
      <!-- Split layout: left sidebar (path chooser), right content (status + mode) -->
      <div class="split-layout">
        <!-- ── Left sidebar: Guided vs Manual ────────────────────────────── -->
        <div class="setup-sidebar">
          <button
            class="setup-tab"
            class:active={setupMode === 'guided'}
            disabled={fixState === 'running'}
            onclick={() => setupMode = 'guided'}
          >
            <div class="tab-title">Help Me Install</div>
            <div class="tab-sub">Guided, in-app</div>
          </button>
          <button
            class="setup-tab"
            class:active={setupMode === 'manual'}
            disabled={fixState === 'running'}
            onclick={() => setupMode = 'manual'}
          >
            <div class="tab-title">I'll Do It Myself</div>
            <div class="tab-sub">Show me the commands</div>
          </button>
        </div>

        <!-- ── Right content: shared status + mode-specific body ─────────── -->
        <div class="setup-content">
          <!-- Status overview (shared across both modes) -->
          <div class="status-card unhealthy">
            {#if status.rust_state === 'clt_missing'}
              <div class="status-headline missing">○ Xcode Command Line Tools required</div>
              <p class="status-sub">Rust needs Apple's Command Line Tools to compile and link on macOS. Install them first — this is a one-time setup that also enables C/C++ and Swift. <strong>Apple's installer takes about 10–15 minutes</strong>, so grab a coffee.</p>
            {:else if status.rust_state === 'not_installed'}
              <div class="status-headline missing">○ Rust is not installed</div>
              <p class="status-sub">Install rustup, cargo, and the stable toolchain to start writing Rust.</p>
            {:else if status.rust_state === 'no_default'}
              <div class="status-headline missing">○ No default toolchain</div>
              <p class="status-sub">rustup is installed, but no default toolchain is selected. This often happens after moving <code>~/.rustup</code>.</p>
            {:else if status.rust_state === 'outdated'}
              <div class="status-headline warn">◐ Rust toolchain is outdated</div>
              <p class="status-sub">Rust is installed but below <strong>{status.rustc.min_version ?? '1.85.0'}</strong>, which is required for <code>edition = "2024"</code>. New playgrounds won't compile until you update.</p>
            {:else if status.rust_state === 'missing_components'}
              <div class="status-headline warn">◐ Missing components</div>
              <p class="status-sub">Rust is installed, but {status.missing_components.join(' and ')} {status.missing_components.length === 1 ? 'is' : 'are'} missing. {status.missing_components.includes('rustfmt') ? 'rustfmt enables auto-formatting.' : ''} {status.missing_components.includes('clippy') ? 'clippy powers live error checking.' : ''}</p>
            {/if}
          </div>

          {#if setupMode === 'guided'}
            <!-- ── Guided mode: existing in-app installer flow ─────────── -->

            <!-- Detail grid — hidden during/after a fix so the log gets the room -->
            {#if fixState === 'idle'}
              <div class="detail-grid">
                <div class="detail-row">
                  <span class="detail-icon" class:ok={status.xcode_clt.installed} class:missing={!status.xcode_clt.installed}>{status.xcode_clt.installed ? '●' : '○'}</span>
                  <span class="detail-label">Xcode CLT</span>
                  <span class="detail-value">{status.xcode_clt.installed ? (status.xcode_clt.path ?? 'installed') : 'not found'}</span>
                </div>
                {#if status.rust_state !== 'clt_missing'}
                  {@const toolchainOutdated = status.rustc.installed && status.rustc.version_ok === false}
                  {@const minVer = status.rustc.min_version ?? '1.85.0'}
                  <div class="detail-row">
                    <span class="detail-icon" class:ok={status.rustup.installed} class:missing={!status.rustup.installed}>{status.rustup.installed ? '●' : '○'}</span>
                    <span class="detail-label">rustup</span>
                    <span class="detail-value">{status.rustup.installed ? (status.rustup.version ?? 'installed') : 'not found'}</span>
                  </div>
                  {@const cargoOk = status.cargo.installed && !toolchainOutdated}
                  <div class="detail-row">
                    <span class="detail-icon" class:ok={cargoOk} class:warn={toolchainOutdated && status.cargo.installed} class:missing={!status.cargo.installed}>{cargoOk ? '●' : toolchainOutdated ? '◐' : '○'}</span>
                    <span class="detail-label">cargo</span>
                    <span class="detail-value">{status.cargo.installed ? (status.cargo.version ?? 'installed') : 'not found'}{toolchainOutdated && status.cargo.installed ? ` · needs ${minVer}+` : ''}</span>
                  </div>
                  {@const rustcOk = status.rustc.installed && !toolchainOutdated}
                  <div class="detail-row">
                    <span class="detail-icon" class:ok={rustcOk} class:warn={toolchainOutdated} class:missing={!status.rustc.installed}>{rustcOk ? '●' : toolchainOutdated ? '◐' : '○'}</span>
                    <span class="detail-label">rustc</span>
                    <span class="detail-value">{status.rustc.installed ? (status.rustc.version ?? 'installed') : 'not found'}{toolchainOutdated ? ` · needs ${minVer}+` : ''}</span>
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
                {/if}
              </div>
            {/if}

            <!-- Fix actions — hidden during/after a fix -->
            {#if fixState === 'idle'}
              <div class="fix-actions">
                {#if status.rust_state === 'clt_missing'}
                  <button class="fix-btn primary" disabled={fixState === 'running'} onclick={() => runFix({ type: 'InstallXcodeCLT' }, 'Launching Apple installer')}>
                    Install Xcode Command Line Tools
                  </button>
                  <p class="fix-hint">
                    Opens <strong>Apple's installer dialog</strong>. The dialog may appear <strong>behind this window</strong> —
                    use <kbd>⌘Tab</kbd> or Mission Control to find it. Click <strong>Install</strong> in Apple's dialog and
                    wait 10–15 minutes for it to finish. Then come back here and click <strong>Re-check</strong>.
                  </p>
                {:else if status.rust_state === 'not_installed'}
                  <button class="fix-btn primary" disabled={fixState === 'running'} onclick={() => runFix({ type: 'InstallRustup' }, 'Installing Rust toolchain')}>
                    {fixState === 'running' ? 'Installing…' : 'Install Rust'}
                  </button>
                  <p class="fix-hint">Runs the official installer from <button class="link-btn" onclick={() => shellOpen('https://rustup.rs')}>rustup.rs</button>. May take a few minutes.</p>
                {:else if status.rust_state === 'no_default'}
                  <button class="fix-btn primary" disabled={fixState === 'running'} onclick={() => runFix({ type: 'SetDefaultStable' }, 'Setting default toolchain')}>
                    {fixState === 'running' ? 'Setting default…' : 'Set default to stable'}
                  </button>
                {:else if status.rust_state === 'outdated'}
                  {#if status.rustup.installed}
                    <button class="fix-btn primary" disabled={fixState === 'running'} onclick={() => runFix({ type: 'UpdateRust' }, 'Updating Rust toolchain')}>
                      {fixState === 'running' ? 'Updating…' : 'Update Rust'}
                    </button>
                    <p class="fix-hint">Runs <code>rustup update stable</code>. Takes 1–3 minutes.</p>
                  {:else}
                    <p class="fix-hint">Your rustc is below {status.rustc.min_version ?? '1.85.0'}, but <strong>rustup isn't installed</strong>, so we can't auto-update. Install rustup first (runs the official installer from <button class="link-btn" onclick={() => shellOpen('https://rustup.rs')}>rustup.rs</button>), then come back and click <strong>Update Rust</strong>.</p>
                    <button class="fix-btn primary" disabled={fixState === 'running'} onclick={() => runFix({ type: 'InstallRustup' }, 'Installing rustup')}>
                      Install rustup
                    </button>
                  {/if}
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

            <!-- Output panel (guided mode only) — visible while a fix is running or after it completes -->
            {#if fixState !== 'idle'}
              <div class="fix-output">
                <div class="fix-header">
                  {#if fixState === 'running'}
                    <span class="fix-status running">{fixActionLabel}…</span>
                  {:else if fixState === 'success'}
                    <span class="fix-status ok">
                      {#if fixActionType === 'InstallXcodeCLT'}
                        ✓ Apple installer launched
                      {:else}
                        ✓ {fixActionLabel} — done
                      {/if}
                    </span>
                  {:else}
                    <span class="fix-status err">✗ {fixActionLabel} — failed</span>
                  {/if}
                </div>
                <pre class="fix-log" bind:this={logEl}>{fixOutput.join('\n') || '(starting…)'}</pre>

                {#if fixState === 'success' && fixActionType === 'InstallXcodeCLT'}
                  <div class="fix-callout">
                    <div class="callout-title">Apple's installer dialog is now open</div>
                    <ol class="callout-steps">
                      <li>
                        If you don't see it, it's probably <strong>behind this window</strong>.
                        Use <kbd>⌘Tab</kbd>, Mission Control (<kbd>F3</kbd>), or check your Dock for the Install dialog.
                      </li>
                      <li>Click <strong>Install</strong> in Apple's dialog and accept the license.</li>
                      <li>Wait 10–15 minutes for Apple to download and install the Command Line Tools.</li>
                      <li>
                        {#if pollingForCLT}
                          We'll continue automatically when the installer finishes — <strong>no need to click anything</strong>.
                        {:else}
                          When it's done, click <strong>Re-check</strong> below to continue with Rust installation.
                        {/if}
                      </li>
                    </ol>
                    {#if pollingForCLT}
                      <div class="poll-indicator">
                        <div class="poll-spinner"></div>
                        <span class="poll-text">
                          Watching for installer to finish… <span class="poll-elapsed">{formatElapsed(pollSecondsElapsed)}</span>
                        </span>
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            {/if}
          {:else}
            <!-- ── Manual mode: state-aware Terminal commands ──────────── -->
            <div class="manual-panel">
              <p class="manual-intro">
                Open <strong>Terminal.app</strong> and run the commands below.
                Come back and click <strong>Re-check</strong> when you're done to verify your setup.
              </p>

              {#if status.rust_state === 'clt_missing'}
                <div class="manual-step">
                  <div class="step-title">1. Install Xcode Command Line Tools</div>
                  <div class="command-box">
                    <code>{XCODE_CLT_CMD}</code>
                    <button class="copy-btn" onclick={() => copyCommand(XCODE_CLT_CMD)}>
                      {copiedCommand === XCODE_CLT_CMD ? 'Copied' : 'Copy'}
                    </button>
                  </div>
                  <p class="step-hint">Opens Apple's installer dialog. Click Install and wait 10–15 minutes.</p>
                </div>
                <div class="manual-step">
                  <div class="step-title">2. Install Rust via rustup</div>
                  <div class="command-box">
                    <code>{RUSTUP_INSTALL_CMD}</code>
                    <button class="copy-btn" onclick={() => copyCommand(RUSTUP_INSTALL_CMD)}>
                      {copiedCommand === RUSTUP_INSTALL_CMD ? 'Copied' : 'Copy'}
                    </button>
                  </div>
                  <p class="step-hint">Installs rustup, cargo, rustc, rustfmt, and clippy. Takes a few minutes.</p>
                </div>
              {:else if status.rust_state === 'not_installed'}
                <div class="manual-step">
                  <div class="step-title">Install Rust via rustup</div>
                  <div class="command-box">
                    <code>{RUSTUP_INSTALL_CMD}</code>
                    <button class="copy-btn" onclick={() => copyCommand(RUSTUP_INSTALL_CMD)}>
                      {copiedCommand === RUSTUP_INSTALL_CMD ? 'Copied' : 'Copy'}
                    </button>
                  </div>
                  <p class="step-hint">Installs rustup, cargo, rustc, rustfmt, and clippy. Takes a few minutes.</p>
                </div>
              {:else if status.rust_state === 'no_default'}
                <div class="manual-step">
                  <div class="step-title">Set stable as the default toolchain</div>
                  <div class="command-box">
                    <code>rustup default stable</code>
                    <button class="copy-btn" onclick={() => copyCommand('rustup default stable')}>
                      {copiedCommand === 'rustup default stable' ? 'Copied' : 'Copy'}
                    </button>
                  </div>
                  <p class="step-hint">Downloads the stable toolchain if needed and sets it as the default.</p>
                </div>
              {:else if status.rust_state === 'outdated'}
                {@const updateCmd = 'rustup update stable && rustup default stable'}
                <div class="manual-step">
                  <div class="step-title">Update Rust to {status.rustc.min_version ?? '1.85.0'}+</div>
                  <div class="command-box">
                    <code>{updateCmd}</code>
                    <button class="copy-btn" onclick={() => copyCommand(updateCmd)}>
                      {copiedCommand === updateCmd ? 'Copied' : 'Copy'}
                    </button>
                  </div>
                  <p class="step-hint">Pulls the latest stable toolchain and re-points your default to it — needed for <code>edition = "2024"</code>. The second step matters if you pinned a specific version before (e.g. <code>rustup default 1.74.0</code>).</p>
                </div>
              {:else if status.rust_state === 'missing_components'}
                {@const componentsCmd = `rustup component add ${status.missing_components.join(' ')}`}
                <div class="manual-step">
                  <div class="step-title">Install missing component{status.missing_components.length === 1 ? '' : 's'}</div>
                  <div class="command-box">
                    <code>{componentsCmd}</code>
                    <button class="copy-btn" onclick={() => copyCommand(componentsCmd)}>
                      {copiedCommand === componentsCmd ? 'Copied' : 'Copy'}
                    </button>
                  </div>
                </div>
              {/if}

              <p class="manual-footer">
                <strong>All done?</strong> Click <strong>Re-check</strong> below and we'll verify everything is set up.
              </p>
            </div>
          {/if}

          <!-- External help links footer — same content regardless of
               guided/manual mode, sits at the bottom of the right pane. -->
          {@render helpLinks()}
        </div>
      </div>
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

  /* Prominent "find Apple's installer" callout, shown in the CLT success state.
     Replaces the normally-terse "done" message with explicit next-step guidance,
     because Apple's installer dialog often opens behind our modal and is easy
     to miss. Styled as a full-width callout under the log, not a toast. */
  .fix-callout {
    background: color-mix(in srgb, var(--accent) 12%, transparent);
    border-top: 1px solid var(--border);
    padding: 12px 14px;
    font-size: 12px;
    color: var(--text);
    line-height: 1.5;
  }
  .callout-title {
    font-weight: 600;
    font-size: 12px;
    margin-bottom: 6px;
    color: var(--text);
  }
  .callout-steps {
    margin: 0;
    padding-left: 20px;
    color: var(--text-secondary);
  }
  .callout-steps li {
    margin-bottom: 4px;
  }
  .callout-steps li:last-child {
    margin-bottom: 0;
  }
  kbd {
    display: inline-block;
    padding: 1px 6px;
    font-family: var(--font-mono);
    font-size: 10px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text);
    vertical-align: 1px;
  }

  /* Auto-polling indicator (visible during background CLT install detection).
     Shows a pulsing dot + elapsed time, communicating that the app is actively
     watching so the user knows they don't need to do anything. */
  .poll-indicator {
    margin-top: 10px;
    padding: 8px 10px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: 6px;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .poll-spinner {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    animation: poll-pulse 1.4s ease-in-out infinite;
    flex-shrink: 0;
  }
  @keyframes poll-pulse {
    0%, 100% { opacity: 0.35; transform: scale(0.85); }
    50%      { opacity: 1;    transform: scale(1.15); }
  }
  .poll-text {
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.4;
  }
  .poll-elapsed {
    font-family: var(--font-mono);
    color: var(--accent);
    font-weight: 600;
  }

  /* External help links footer — rendered in every FixWizard state
     (healthy + unhealthy split layout) so users can jump to external
     Rust resources regardless of toolchain status. */
  .help-links {
    margin-top: 14px;
    padding: 12px 14px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .help-links-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }
  .help-links-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px 16px;
  }
  .help-links-list li {
    font-size: 11px;
    color: var(--text-tertiary);
    line-height: 1.6;
  }
  .help-links .link-btn {
    color: var(--accent);
    font-size: 11px;
    font-family: inherit;
    text-decoration: none;
  }
  .help-links .link-btn:hover { text-decoration: underline; }

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

  /* ── Split layout: left sidebar (path chooser) + right content area ───── */
  .split-layout {
    display: flex;
    gap: 0;
    /* Bleed to modal edges so the sidebar can have its own background flush
       against the modal border, like Apple's Settings app. Negative margins
       must match .modal-body padding exactly. */
    margin: -16px -20px -20px;
    border-radius: 0;
    min-height: 0;
  }
  .setup-sidebar {
    width: 180px;
    flex-shrink: 0;
    padding: 12px 8px;
    background: var(--bg-input);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .setup-tab {
    display: flex; flex-direction: column; align-items: flex-start;
    gap: 2px;
    padding: 10px 12px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s, border-color 0.1s;
  }
  .setup-tab:hover:not(:disabled):not(.active) {
    background: var(--bg-hover);
  }
  .setup-tab.active {
    background: color-mix(in srgb, var(--accent) 15%, transparent);
    border-color: color-mix(in srgb, var(--accent) 40%, transparent);
  }
  .setup-tab:disabled { opacity: 0.45; cursor: not-allowed; }
  .setup-tab .tab-title {
    font-size: 12px; font-weight: 600; color: var(--text);
    line-height: 1.3;
  }
  .setup-tab .tab-sub {
    font-size: 10px; color: var(--text-tertiary);
    line-height: 1.3;
  }
  .setup-content {
    flex: 1; min-width: 0;
    padding: 14px 16px;
    display: flex; flex-direction: column; gap: 12px;
    overflow-y: auto;
  }
  /* Shrink status card a touch to fit the narrower right pane */
  .setup-content .status-card {
    margin: 0;
  }

  /* ── Manual mode: terminal command boxes ──────────────────────────────── */
  .manual-panel {
    display: flex; flex-direction: column; gap: 14px;
  }
  .manual-intro {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
  }
  .manual-step {
    display: flex; flex-direction: column; gap: 6px;
  }
  .step-title {
    font-size: 12px; font-weight: 600; color: var(--text);
  }
  .command-box {
    display: flex; align-items: stretch; gap: 0;
    background: rgba(0,0,0,0.35);
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }
  .command-box code {
    flex: 1;
    padding: 8px 10px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text);
    line-height: 1.4;
    white-space: pre-wrap;
    word-break: break-all;
    overflow-x: auto;
  }
  .copy-btn {
    flex-shrink: 0;
    padding: 0 12px;
    background: var(--bg-input);
    border: none;
    border-left: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }
  .copy-btn:hover { background: var(--bg-hover); color: var(--text); }
  .step-hint {
    margin: 0;
    font-size: 10px;
    color: var(--text-tertiary);
    line-height: 1.4;
  }
  .manual-footer {
    margin: 4px 0 0;
    padding-top: 10px;
    border-top: 1px dashed var(--border);
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.5;
  }
</style>
