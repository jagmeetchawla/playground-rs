<script lang="ts">
  import { onMount } from 'svelte'
  import { getVersion } from '@tauri-apps/api/app'
  import { open as shellOpen } from '@tauri-apps/plugin-shell'
  import appIcon from './app-icon.png'
  import type { EditionConfig } from './editions'

  let { onclose, edition }: { onclose: () => void; edition: EditionConfig } = $props()
  let version = $state('')

  onMount(async () => {
    version = await getVersion()
  })

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
    <h1 class="app-name">{edition.displayName}</h1>
    <p class="app-version">Version {version}</p>

    <p class="app-desc">{edition.tagline}</p>

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
        <span class="stack-label">Website</span>
        <a class="stack-link" href="https://rusticplayground.dev" target="_blank" rel="noreferrer">
          rusticplayground.dev
        </a>
      </div>
      <div class="stack-row">
        <span class="stack-label">Source</span>
        <a class="stack-link" href="https://github.com/jagmeetchawla/rustic-playground" target="_blank" rel="noreferrer">
          github.com/jagmeetchawla/rustic-playground
        </a>
      </div>
    </div>

    <div class="divider"></div>

    <!-- Support the project — both actions open the system browser via
         Tauri's shell plugin. Zero backend, zero auth: starring happens
         in GitHub's UI, subscribe lands on the site's signup page. -->
    <div class="support-actions">
      <button class="support-btn" onclick={() => shellOpen('https://github.com/jagmeetchawla/rustic-playground')}>
        <span class="support-icon" aria-hidden="true">⭐</span>
        <span class="support-label">Star on GitHub</span>
      </button>
      <button class="support-btn" onclick={() => shellOpen('https://rusticplayground.dev/subscribe')}>
        <span class="support-icon" aria-hidden="true">📬</span>
        <span class="support-label">Get email updates</span>
      </button>
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
    font-size: 18px; font-weight: 600; color: var(--text);
    margin: 0 0 4px;
  }
  .app-version {
    font-size: 12px; color: var(--text-tertiary); margin: 0 0 12px;
  }
  .app-desc {
    font-size: 13px; color: var(--text-secondary); line-height: 1.6;
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
    color: var(--accent); font-size: 12px;
    text-decoration: none; word-break: break-all;
  }
  .stack-link:hover { text-decoration: underline; }

  /* Support the project — Star + Subscribe. Two equal-width buttons in a
     row on wide viewports; the container gracefully wraps to a stack when
     the modal shrinks. Icon + label centered; hover raises contrast
     without a color swap so it works across all 8 themes. */
  .support-actions {
    width: 100%;
    display: flex; gap: 8px;
    margin-bottom: 16px;
  }
  .support-btn {
    flex: 1;
    display: flex; align-items: center; justify-content: center; gap: 6px;
    padding: 8px 10px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-size: 12px; font-weight: 500;
    cursor: pointer;
    transition: background 0.12s, border-color 0.12s;
  }
  .support-btn:hover {
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }
  .support-icon { font-size: 13px; line-height: 1; }
  .support-label { line-height: 1; }

  .disclaimer {
    font-size: 11px; color: var(--text-tertiary);
    line-height: 1.55; margin: 0; text-align: center;
  }
</style>
