<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { tick } from 'svelte'
  import { getLang, type ProjectType } from './languages'

  let {
    playgroundName,
    code,
    projects,
    projectTypes = {},
    projectSources = {},
    projectType,
    currentProject,
    onclose,
    oncopy,
  }: {
    playgroundName: string
    code: string
    projects: string[]
    projectTypes: Record<string, ProjectType>
    projectSources: Record<string, string>
    projectType: ProjectType
    currentProject: string
    onclose: () => void
    oncopy: (targetProject: string, newName: string) => void
  } = $props()

  // Filter to user projects of the same language type (no book projects), excluding current
  let targetProjects = $derived(
    projects.filter(p =>
      !projectSources[p] &&
      p !== currentProject &&
      (projectTypes[p] ?? 'rust') === projectType
    )
  )

  let selectedProject = $state('')
  let newName = $state(playgroundName)
  let error = $state('')
  let busy = $state(false)
  let nameInput: HTMLInputElement | null = $state(null)

  // Auto-select the first matching project
  $effect(() => {
    if (targetProjects.length > 0 && !selectedProject) {
      selectedProject = targetProjects[0]
    }
  })

  $effect(() => {
    tick().then(() => nameInput?.focus())
  })

  async function confirm() {
    if (!selectedProject || !newName.trim()) return
    error = ''
    busy = true
    try {
      await invoke('copy_playground_to_project', {
        code,
        targetProject: selectedProject,
        playgroundName: newName.trim(),
      })
      oncopy(selectedProject, newName.trim())
    } catch (e) {
      error = String(e)
      busy = false
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); confirm() }
    if (e.key === 'Escape') onclose()
  }

  let lang = $derived(getLang(projectType))
</script>

<div class="backdrop" onclick={onclose} aria-hidden="true"></div>
<div class="modal" role="dialog" aria-modal="true" tabindex="-1" onkeydown={handleKey}>
  <h3 class="title">Copy to Project</h3>

  {#if targetProjects.length === 0}
    <p class="empty">No {lang.label} projects available. Create one first.</p>
    <div class="actions">
      <button class="btn-cancel" onclick={onclose}>Close</button>
    </div>
  {:else}
    <span class="label">Target project</span>
    <div class="project-list">
      {#each targetProjects as name (name)}
        {@const ptype = projectTypes[name] ?? 'rust'}
        {@const pLang = getLang(ptype)}
        <button
          class="project-option"
          class:selected={selectedProject === name}
          onclick={() => selectedProject = name}
        >
          <span class="option-badge" class:clang={ptype === 'clang'} class:zig={ptype === 'zig'} class:swift={ptype === 'swift'}>{pLang.badge}</span>
          <span class="option-name">{name}</span>
          {#if selectedProject === name}
            <svg class="option-check" width="10" height="8" viewBox="0 0 10 8" fill="none">
              <path d="M1 4l3 3 5-6" stroke="currentColor" stroke-width="1.6"
                    stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          {/if}
        </button>
      {/each}
    </div>

    <span class="label">Playground name</span>
    <input
      class="input"
      class:input-error={!!error}
      type="text"
      bind:value={newName}
      bind:this={nameInput}
      spellcheck="false"
      onkeydown={handleKey}
    />
    {#if error}
      <p class="error">{error}</p>
    {/if}

    <div class="actions">
      <button class="btn-cancel" onclick={onclose}>Cancel</button>
      <button class="btn-confirm" onclick={confirm} disabled={busy || !selectedProject || !newName.trim()}>Copy</button>
    </div>
  {/if}
</div>

<style>
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.45);
    z-index: 900;
  }
  .modal {
    position: fixed;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    width: 360px;
    background: var(--bg-sidebar);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 20px;
    z-index: 901;
    box-shadow: 0 8px 30px rgba(0,0,0,0.35);
  }
  .title {
    margin: 0 0 14px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }
  .label {
    display: block;
    font-size: 11px;
    color: var(--text-secondary);
    margin-bottom: 4px;
    margin-top: 10px;
  }
  .project-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    max-height: 160px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-input, rgba(255,255,255,0.06));
  }
  .project-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    font-size: 12px;
    font-family: var(--font-mono, monospace);
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    width: 100%;
  }
  .project-option:hover { background: rgba(255,255,255,0.06); }
  .project-option.selected { background: rgba(var(--accent-rgb, 10,132,255), 0.15); }
  .option-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 9px;
    font-weight: 700;
    min-width: 22px;
    height: 16px;
    border-radius: 4px;
    background: var(--accent);
    color: #fff;
    letter-spacing: 0.04em;
    flex-shrink: 0;
  }
  .option-badge.clang { background: #4a9; }
  .option-badge.zig    { background: #f7a41d; color: #000; }
  .option-badge.swift  { background: #f05138; }
  .option-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .option-check { flex-shrink: 0; color: var(--accent); }
  .input {
    width: 100%;
    box-sizing: border-box;
    padding: 6px 8px;
    font-size: 13px;
    font-family: inherit;
    background: var(--bg-input, rgba(255,255,255,0.06));
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    outline: none;
  }
  .input:focus {
    border-color: var(--accent);
  }
  .input-error {
    border-color: var(--red);
  }
  .error {
    font-size: 11px;
    color: var(--red);
    margin: 4px 0 0;
  }
  .empty {
    font-size: 12px;
    color: var(--text-secondary);
    margin: 8px 0 16px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
  .btn-cancel, .btn-confirm {
    padding: 5px 14px;
    font-size: 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    cursor: pointer;
    font-family: inherit;
  }
  .btn-cancel {
    background: transparent;
    color: var(--text-secondary);
  }
  .btn-cancel:hover { background: rgba(255,255,255,0.06); }
  .btn-confirm {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
  }
  .btn-confirm:hover { filter: brightness(1.1); }
  .btn-confirm:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
