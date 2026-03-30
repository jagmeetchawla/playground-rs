<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte'
  import * as monaco from 'monaco-editor'
  import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'

  // Monaco needs to know how to load web workers.
  // @ts-ignore
  self.MonacoEnvironment = {
    getWorker() { return new editorWorker() }
  }

  // ── Custom dark theme — matches our #1c1c1e macOS dark background ─────────
  monaco.editor.defineTheme('playground-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      // Comments — muted gray
      { token: 'comment',            foreground: '636770', fontStyle: 'italic' },
      // Keywords: fn, let, mut, if, else, match, return, use, pub, …
      { token: 'keyword',            foreground: 'fc5fa3' },
      // Types and type parameters
      { token: 'type',               foreground: '5dd8ff' },
      { token: 'type.identifier',    foreground: '5dd8ff' },
      // String literals
      { token: 'string',             foreground: 'fc6a5d' },
      { token: 'string.escape',      foreground: 'ff8170' },
      // Number literals
      { token: 'number',             foreground: 'd9c97c' },
      { token: 'number.float',       foreground: 'd9c97c' },
      // Operators, punctuation
      { token: 'operator',           foreground: 'cdd6f4' },
      { token: 'delimiter',          foreground: '8a8f98' },
      // Attributes #[derive(...)]
      { token: 'attribute',          foreground: 'a8c7fa' },
      // Macros: println!, vec!, etc.
      { token: 'macro',              foreground: 'a8c7fa' },
      // Identifiers / functions
      { token: 'identifier',         foreground: 'cdd6f4' },
    ],
    colors: {
      // Core backgrounds
      'editor.background':                  '#1c1c1e',
      'editor.foreground':                  '#cdd6f4',
      'editorLineNumber.foreground':        '#3f3f46',
      'editorLineNumber.activeForeground':  '#a1a1aa',
      'editor.lineHighlightBackground':     '#2c2c2e',
      'editor.lineHighlightBorder':         '#00000000',

      // Selection
      'editor.selectionBackground':         '#0a84ff40',
      'editor.inactiveSelectionBackground': '#0a84ff25',

      // Cursor
      'editorCursor.foreground':            '#0a84ff',

      // Indent guides
      'editorIndentGuide.background':       '#2c2c2e',
      'editorIndentGuide.activeBackground': '#3a3a3c',

      // Bracket match
      'editorBracketMatch.background':      '#0a84ff30',
      'editorBracketMatch.border':          '#0a84ff80',

      // Gutter / margins
      'editorGutter.background':            '#1c1c1e',

      // Find/highlight
      'editor.findMatchBackground':         '#ffd60a50',
      'editor.findMatchHighlightBackground':'#ffd60a25',

      // Error / warning squiggles
      'editorError.foreground':             '#ff453a',
      'editorWarning.foreground':           '#ffd60a',
      'editorInfo.foreground':              '#0a84ff',

      // Scrollbar
      'scrollbarSlider.background':         '#ffffff20',
      'scrollbarSlider.hoverBackground':    '#ffffff35',
      'scrollbarSlider.activeBackground':   '#ffffff50',
    }
  })

  const dispatch = createEventDispatcher()
  let { code }: { code: string } = $props()

  let container: HTMLDivElement
  let editor: monaco.editor.IStandaloneCodeEditor

  onMount(() => {
    editor = monaco.editor.create(container, {
      value: code,
      language: 'rust',
      theme: 'playground-dark',
      fontSize: 13,
      fontFamily: "'Menlo', 'Monaco', 'SF Mono', 'Courier New', monospace",
      fontLigatures: false,
      lineNumbers: 'on',
      lineNumbersMinChars: 3,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize: 4,
      insertSpaces: true,
      wordWrap: 'off',
      padding: { top: 20, bottom: 20 },
      renderLineHighlight: 'line',
      cursorBlinking: 'smooth',
      cursorSmoothCaretAnimation: 'on',
      smoothScrolling: true,
      bracketPairColorization: { enabled: true },
      'semanticHighlighting.enabled': true,
      overviewRulerBorder: false,
      hideCursorInOverviewRuler: true,
      renderLineHighlightOnlyWhenFocus: false,
      scrollbar: {
        verticalScrollbarSize: 6,
        horizontalScrollbarSize: 6,
        useShadows: false,
      },
    })

    editor.onDidChangeModelContent(() => {
      dispatch('change', editor.getValue())
    })

    // Let our global keyboard shortcuts handle these — don't let Monaco consume them
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyR, () => {})
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {})
  })

  // Sync when user switches playground (code prop changes from outside)
  $effect(() => {
    if (editor && editor.getValue() !== code) {
      // Push value + clear undo history so Ctrl+Z doesn't bleed across files
      editor.getModel()?.setValue(code)
    }
  })

  onDestroy(() => { editor?.dispose() })
</script>

<div class="editor-container" bind:this={container}></div>

<style>
  .editor-container {
    flex: 1;
    overflow: hidden;
  }
</style>
