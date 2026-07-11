<script lang="ts">
  // Embedded subscribe modal — loads rusticplayground.dev/subscribe in an
  // iframe with the current app theme passed as a query param, so the site's
  // matching theme (dark/light/rust) picks it up and renders in sync with the
  // app. Falls back to the system browser if the iframe fails to load.
  import { open as shellOpen } from '@tauri-apps/plugin-shell'

  let {
    onclose,
    theme,
  }: {
    onclose: () => void
    /** Current app resolved theme — passed to the site so it renders
        in the matching color scheme. Site reads this from ?theme=… */
    theme: string
  } = $props()

  const SUBSCRIBE_URL = 'https://rusticplayground.dev/subscribe/'
  // ?theme=<name> — site renders in the matching palette to the app.
  // &embed=1     — site suppresses the "← rusticplayground.dev" back-nav
  //                footer, since the user is inside a modal, not a browser.
  let src = $derived(`${SUBSCRIBE_URL}?theme=${encodeURIComponent(theme)}&embed=1`)
  let loading = $state(true)
  let loadError = $state(false)

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose()
  }

  function openInBrowser() {
    shellOpen(SUBSCRIBE_URL)
    onclose()
  }
</script>

<svelte:window onkeydown={handleKey} />

<div class="backdrop" onclick={onclose} aria-hidden="true"></div>

<div class="modal" role="dialog" aria-modal="true" aria-label="Subscribe to updates">
  <div class="modal-header">
    <span class="modal-title">Get email updates</span>
    <div class="header-actions">
      <button class="header-btn" onclick={openInBrowser} title="Open in browser">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M4.5 2H2v8h8V7.5M7 2h3v3M10 2L5 7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
        </svg>
      </button>
      <button class="header-btn close-btn" onclick={onclose} aria-label="Close">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="modal-body">
    {#if loading && !loadError}
      <div class="loading-overlay">
        <div class="spinner"></div>
        <span class="loading-text">Loading subscribe page…</span>
      </div>
    {/if}
    {#if loadError}
      <div class="error-overlay">
        <p class="error-headline">Couldn't load the page</p>
        <p class="error-sub">rusticplayground.dev may be down, or your network is offline.</p>
        <button class="fallback-btn" onclick={openInBrowser}>Open in system browser</button>
      </div>
    {/if}
    <iframe
      {src}
      title="Subscribe to Rustic Playground updates"
      onload={() => { loading = false }}
      onerror={() => { loadError = true; loading = false }}
    ></iframe>
  </div>

  <!-- Honest attribution: one Buttondown list serves all of Jagmeet's apps,
       so subscribing here also gets updates for any other apps he ships.
       Users should know they're joining a wider newsletter, not an app-
       specific list, before they hand over their email. -->
  <div class="modal-footnote">
    An app by <strong>Jagmeet Chawla</strong> — this list covers updates for all of his apps, not just this one.
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
    width: min(560px, calc(100vw - 40px));
    height: min(640px, calc(100vh - 80px));
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    box-shadow: 0 24px 80px rgba(0,0,0,0.7), 0 4px 16px rgba(0,0,0,0.4);
    display: flex; flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .modal-title { font-size: 13px; font-weight: 600; color: var(--text); }
  .header-actions { display: flex; align-items: center; gap: 4px; }
  .header-btn {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%; color: var(--text-tertiary);
    background: none; border: none; cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }
  .header-btn:hover { background: var(--bg-hover); color: var(--text); }

  .modal-body {
    flex: 1; position: relative;
    background: var(--bg-elevated);
  }
  iframe {
    width: 100%; height: 100%;
    border: none;
    display: block;
    background: transparent;
    color-scheme: light dark;
  }

  /* Sits above the iframe until the first load event fires — hides the
     inevitable flash-of-unstyled-content while Buttondown's JS bootstraps. */
  .loading-overlay,
  .error-overlay {
    position: absolute; inset: 0;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 12px;
    background: var(--bg-elevated);
    padding: 40px 24px;
    text-align: center;
  }
  .error-overlay { gap: 8px; }
  .spinner {
    width: 20px; height: 20px;
    border: 2px solid rgba(255,255,255,0.15);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .loading-text {
    font-size: 12px; color: var(--text-secondary);
  }
  .error-headline { font-size: 13px; font-weight: 600; color: var(--text); margin: 0; }
  .error-sub { font-size: 11px; color: var(--text-secondary); margin: 0; line-height: 1.5; }
  .fallback-btn {
    margin-top: 8px;
    font-size: 12px; font-weight: 500;
    padding: 7px 14px; border-radius: 6px;
    background: var(--accent); color: white;
    border: 1px solid var(--accent);
    cursor: pointer;
    transition: filter 0.1s;
  }
  .fallback-btn:hover { filter: brightness(1.1); }

  /* Attribution footnote — small print acknowledging one Buttondown list
     serves all of Jagmeet's apps. Sits below the iframe, styled quietly so
     it doesn't compete with the site's own signup form. */
  .modal-footnote {
    flex-shrink: 0;
    padding: 10px 14px;
    border-top: 1px solid var(--border);
    background: var(--bg-elevated);
    font-size: 11px;
    color: var(--text-tertiary);
    text-align: center;
    line-height: 1.5;
  }
  .modal-footnote strong {
    color: var(--text-secondary);
    font-weight: 600;
  }
</style>
