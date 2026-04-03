<script lang="ts">
  import { tick } from 'svelte'
  import { TEMPLATES, C_TEMPLATES, CPP_TEMPLATES, type Template } from './templates'

  let {
    existingNames,
    projectType = 'rust',
    onclose,
    oncreate,
  }: {
    existingNames: string[]
    projectType?: 'rust' | 'native'
    onclose: () => void
    oncreate: (name: string, template: Template) => void
  } = $props()

  let isNative = $derived(projectType === 'native')

  let selectedId = $state('blank')
  let nativeSelectedId = $state('c_blank')
  let name = $state('')
  let nameError = $state('')
  let nameInput = $state<HTMLInputElement | null>(null)
  let nativeLang: 'c' | 'cpp' = $state('c')

  let nativeTemplates = $derived(nativeLang === 'cpp' ? CPP_TEMPLATES : C_TEMPLATES)
  let selected = $derived(
    isNative
      ? (nativeTemplates.find(t => t.id === nativeSelectedId) ?? nativeTemplates[0])
      : (TEMPLATES.find(t => t.id === selectedId) ?? TEMPLATES[0])
  )

  // Reset native template selection when switching language
  $effect(() => {
    nativeSelectedId = nativeLang === 'cpp' ? 'cpp_blank' : 'c_blank'
  })

  $effect(() => {
    if (nameInput) {
      tick().then(() => nameInput?.focus())
    }
  })

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose()
  }

  function validateAndCreate() {
    const trimmed = name.trim()
    if (!trimmed) { nameError = 'Name is required'; return }
    if (!/^[a-z][a-z0-9_]*$/.test(trimmed)) {
      nameError = 'Lowercase letters, digits, underscores. Must start with a letter.'
      return
    }
    if (trimmed.length > 64) { nameError = 'Max 64 characters'; return }

    if (isNative) {
      // Native: append extension, check for collision with full filename
      const ext = nativeLang === 'cpp' ? '.cpp' : '.c'
      const fullName = `${trimmed}${ext}`
      if (existingNames.includes(fullName)) { nameError = 'Already exists'; return }
      nameError = ''
      oncreate(fullName, selected)
    } else {
      if (existingNames.includes(trimmed)) { nameError = 'Already exists'; return }
      nameError = ''
      oncreate(trimmed, selected)
    }
  }
</script>

<svelte:window onkeydown={handleKey} />

<div class="backdrop" onclick={onclose} aria-hidden="true"></div>

<div class="modal" role="dialog" aria-modal="true" aria-label="New Playground">
  <div class="modal-header">
    <div class="header-left">
      {#if isNative}
        <span class="native-badge">&#63743;</span>
      {:else}
        <span class="rs-badge">RS</span>
      {/if}
      <span class="modal-title">New Playground</span>
    </div>
    <button class="close-btn" onclick={onclose} aria-label="Close">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  <div class="modal-body">
    {#if isNative}
      <!-- Language tabs -->
      <div class="lang-tabs">
        <button class="lang-tab" class:active={nativeLang === 'c'} onclick={() => nativeLang = 'c'}>
          <span class="lang-tab-badge">C</span> C
        </button>
        <button class="lang-tab" class:active={nativeLang === 'cpp'} onclick={() => nativeLang = 'cpp'}>
          <span class="lang-tab-badge">C++</span> C++
        </button>
      </div>
    {/if}

    <!-- Name input -->
    <div class="name-section">
      <label for="pg-name">Name</label>
      <div class="name-input-row">
        <input
          id="pg-name"
          type="text"
          bind:this={nameInput}
          bind:value={name}
          placeholder="my_playground"
          spellcheck="false"
          autocomplete="off"
          onkeydown={(e) => { if (e.key === 'Enter') validateAndCreate() }}
        />
        {#if isNative}
          <span class="ext-suffix">.{nativeLang}</span>
        {/if}
      </div>
      {#if nameError}
        <span class="name-error">{nameError}</span>
      {/if}
    </div>

    {#if isNative}
      <!-- Template grid for native -->
      <div class="template-section">
        <label>Template</label>
        <div class="template-grid">
          {#each nativeTemplates as tmpl (tmpl.id)}
            <button
              class="template-card"
              class:selected={nativeSelectedId === tmpl.id}
              onclick={() => nativeSelectedId = tmpl.id}
            >
              <span class="tmpl-name">{tmpl.name}</span>
              <span class="tmpl-desc">{tmpl.description}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Preview -->
      <div class="preview-section">
        <label>Preview</label>
        <pre class="preview-code">{selected.code.slice(0, 300)}{selected.code.length > 300 ? '...' : ''}</pre>
      </div>
    {:else}
      <!-- Template grid for Rust projects -->
      <div class="template-section">
        <label>Template</label>
        <div class="template-grid">
          {#each TEMPLATES as tmpl (tmpl.id)}
            <button
              class="template-card"
              class:selected={selectedId === tmpl.id}
              onclick={() => selectedId = tmpl.id}
            >
              <span class="tmpl-name">{tmpl.name}</span>
              <span class="tmpl-desc">{tmpl.description}</span>
              {#if tmpl.deps?.length}
                <span class="tmpl-deps">
                  {tmpl.deps.map(d => d.name).join(', ')}
                </span>
              {/if}
            </button>
          {/each}
        </div>
      </div>

      <!-- Preview -->
      <div class="preview-section">
        <label>Preview</label>
        <pre class="preview-code">{selected.code.slice(0, 300)}{selected.code.length > 300 ? '...' : ''}</pre>
      </div>
    {/if}
  </div>

  <div class="modal-footer">
    <button class="btn btn-secondary" onclick={onclose}>Cancel</button>
    <button class="btn btn-primary" onclick={validateAndCreate}>Create</button>
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
    width: min(720px, calc(100vw - 40px));
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
  .native-badge {
    font-size: 11px; font-weight: 400;
    background: #4a9; color: #fff;
    border-radius: 3px; padding: 0 4px; line-height: 1.3;
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
    padding: 16px 20px;
    display: flex; flex-direction: column; gap: 16px;
  }

  label {
    font-size: 11px; font-weight: 700; letter-spacing: 0.06em;
    text-transform: uppercase; color: var(--rust-orange);
    display: block; margin-bottom: 6px;
  }

  /* Name input */
  .name-input-row {
    display: flex; align-items: center; gap: 0;
  }
  .name-input-row input {
    flex: 1; box-sizing: border-box;
    font-family: var(--font-mono); font-size: 13px;
    background: rgba(0,0,0,0.25); color: var(--text);
    border: 1px solid var(--border); border-radius: var(--radius-xs);
    padding: 6px 10px; outline: none;
  }
  .name-input-row input:focus { border-color: var(--accent); }
  .name-input-row input::placeholder { color: var(--text-tertiary); opacity: 0.5; }
  .ext-suffix {
    font-family: var(--font-mono); font-size: 13px;
    color: var(--text-tertiary); padding: 6px 8px;
    background: rgba(0,0,0,0.15); border: 1px solid var(--border);
    border-left: none; border-radius: 0 var(--radius-xs) var(--radius-xs) 0;
  }
  .ext-suffix ~ input,
  .name-input-row:has(.ext-suffix) input {
    border-radius: var(--radius-xs) 0 0 var(--radius-xs);
  }
  .name-error { font-size: 11px; color: var(--red); margin-top: 4px; display: block; }

  /* Language tabs for native projects */
  .lang-tabs {
    display: flex; gap: 0;
    border: 1px solid var(--border); border-radius: 6px;
    overflow: hidden;
  }
  .lang-tab {
    flex: 1;
    display: flex; align-items: center; justify-content: center; gap: 5px;
    padding: 6px 10px;
    font-size: 12px; font-weight: 600;
    color: var(--text-tertiary);
    background: none; border: none;
    cursor: pointer;
    transition: color 0.15s, background 0.15s;
  }
  .lang-tab:first-child { border-right: 1px solid var(--border); }
  .lang-tab:hover { color: var(--text-secondary); background: rgba(255,255,255,0.03); }
  .lang-tab.active { color: var(--text); background: rgba(68,170,153,0.12); }
  .lang-tab-badge {
    font-size: 7px; font-weight: 800;
    background: #4a9; color: #fff;
    border-radius: 3px; padding: 1.5px 3px; line-height: 1.3;
  }

  /* Template grid */
  .template-grid {
    display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px;
  }
  .template-card {
    display: flex; flex-direction: column; gap: 2px;
    padding: 8px 10px; text-align: left;
    background: rgba(255,255,255,0.04);
    border: 1px solid var(--border); border-radius: 6px;
    cursor: pointer; transition: background 0.1s, border-color 0.1s;
  }
  .template-card:hover { background: rgba(255,255,255,0.08); border-color: var(--border-strong); }
  .template-card.selected {
    background: rgba(var(--accent-rgb, 59,130,246), 0.15);
    border-color: var(--accent);
  }
  .tmpl-name { font-size: 12px; font-weight: 600; color: var(--text); }
  .tmpl-desc { font-size: 10px; color: var(--text-tertiary); line-height: 1.4; }
  .tmpl-deps {
    font-size: 9px; color: var(--accent); margin-top: 2px;
    font-family: var(--font-mono);
  }

  /* Preview */
  .preview-section {
    background: rgba(0,0,0,0.2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px 12px;
  }
  .preview-code {
    font-family: var(--font-mono); font-size: 11px;
    color: var(--text-secondary); line-height: 1.6;
    margin: 0; white-space: pre; overflow-x: auto;
    max-height: 150px; overflow-y: auto;
  }

  /* Footer */
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
</style>
