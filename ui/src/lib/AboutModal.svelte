<script lang="ts">
  import appIcon from './app-icon.png'

  let { onclose }: { onclose: () => void } = $props()

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose()
  }
</script>

<svelte:window onkeydown={handleKey} />

<!-- Backdrop -->
<div class="backdrop" onclick={onclose} aria-hidden="true"></div>

<div class="modal" role="dialog" aria-modal="true" aria-label="About Rustic Playground">
  <!-- Close -->
  <button class="close-btn" onclick={onclose} aria-label="Close">
    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
      <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
    </svg>
  </button>

  <!-- Logo -->
  <div class="logo-wrap">
    <img class="logo" src={appIcon} alt="Rustic Playground" width="80" height="80" />
  </div>

  <div class="about-body">
    <h1 class="app-name">Rustic Playground</h1>
    <p class="app-version">Version 0.1.0</p>

    <p class="app-desc">
      A macOS desktop app for Rust experiments,<br/>
      inspired by Swift Playgrounds.
    </p>

    <div class="divider"></div>

    <div class="stack-info">
      <div class="stack-row">
        <span class="stack-label">Backend</span>
        <span class="stack-value">Tauri 2 + Rust</span>
      </div>
      <div class="stack-row">
        <span class="stack-label">Frontend</span>
        <span class="stack-value">Svelte 5</span>
      </div>
      <div class="stack-row">
        <span class="stack-label">Editor</span>
        <span class="stack-value">Monaco (VS Code engine)</span>
      </div>
      <div class="stack-row">
        <span class="stack-label">License</span>
        <span class="stack-value">MIT</span>
      </div>
      <div class="stack-row">
        <span class="stack-label">Source</span>
        <a class="stack-link" href="https://github.com/jagmeetchawla/rustic-playground" target="_blank" rel="noreferrer">
          github.com/jagmeetchawla/rustic-playground
        </a>
      </div>
    </div>

    <div class="divider"></div>

    <p class="disclaimer">
      This app is not sandboxed. It compiles and executes code using your local
      Rust toolchain. Only run code you trust.
    </p>
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
    width: 340px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: 12px;
    box-shadow: 0 24px 80px rgba(0,0,0,0.7), 0 4px 16px rgba(0,0,0,0.4);
    padding: 28px 28px 24px;
    display: flex; flex-direction: column; align-items: center;
    gap: 0;
  }

  .close-btn {
    position: absolute; top: 12px; right: 12px;
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%; color: var(--text-tertiary);
    transition: background 0.1s, color 0.1s;
  }
  .close-btn:hover { background: var(--bg-hover); color: var(--text); }

  /* ── Logo ── */
  .logo-wrap { margin-bottom: 16px; }
  .logo {
    width: 80px; height: 80px;
    border-radius: 18px;
    box-shadow: 0 4px 24px rgba(0,0,0,0.6);
  }

  /* ── Body text ── */
  .about-body {
    width: 100%; display: flex; flex-direction: column; align-items: center; gap: 0;
    text-align: center;
  }

  .app-name {
    font-size: 18px; font-weight: 700; color: var(--text);
    margin: 0 0 4px;
  }
  .app-version {
    font-size: 12px; color: var(--text-tertiary); margin: 0 0 12px;
  }
  .app-desc {
    font-size: 12.5px; color: var(--text-secondary); line-height: 1.6;
    margin: 0 0 16px;
  }

  .divider {
    width: 100%; border-top: 1px solid var(--border);
    margin-bottom: 16px;
  }

  /* ── Stack info ── */
  .stack-info {
    width: 100%; display: flex; flex-direction: column; gap: 6px;
    margin-bottom: 16px;
    text-align: left;
  }
  .stack-row {
    display: flex; justify-content: space-between; align-items: baseline;
    font-size: 12px; gap: 12px;
  }
  .stack-label { color: var(--text-tertiary); flex-shrink: 0; }
  .stack-value { color: var(--text-secondary); }
  .stack-link {
    color: var(--accent); font-size: 11.5px;
    text-decoration: none; word-break: break-all;
  }
  .stack-link:hover { text-decoration: underline; }

  .disclaimer {
    font-size: 10.5px; color: var(--text-tertiary);
    line-height: 1.55; margin: 0; text-align: center;
  }
</style>
