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

  // ── Custom light theme — matches macOS light appearance ────────────────────
  monaco.editor.defineTheme('playground-light', {
    base: 'vs',
    inherit: true,
    rules: [
      { token: 'comment',            foreground: '8e8e93', fontStyle: 'italic' },
      { token: 'keyword',            foreground: 'ad3da4' },
      { token: 'type',               foreground: '0b4f79' },
      { token: 'type.identifier',    foreground: '0b4f79' },
      { token: 'string',             foreground: 'c41a16' },
      { token: 'string.escape',      foreground: 'c41a16' },
      { token: 'number',             foreground: '1c00cf' },
      { token: 'number.float',       foreground: '1c00cf' },
      { token: 'operator',           foreground: '1c1c1e' },
      { token: 'delimiter',          foreground: '636366' },
      { token: 'attribute',          foreground: '703daa' },
      { token: 'macro',              foreground: '703daa' },
      { token: 'identifier',         foreground: '1c1c1e' },
    ],
    colors: {
      'editor.background':                  '#ffffff',
      'editor.foreground':                  '#1c1c1e',
      'editorLineNumber.foreground':        '#c7c7cc',
      'editorLineNumber.activeForeground':  '#8e8e93',
      'editor.lineHighlightBackground':     '#f2f2f7',
      'editor.lineHighlightBorder':         '#00000000',
      'editor.selectionBackground':         '#0a84ff30',
      'editor.inactiveSelectionBackground': '#0a84ff18',
      'editorCursor.foreground':            '#0a84ff',
      'editorIndentGuide.background':       '#e5e5ea',
      'editorIndentGuide.activeBackground': '#d1d1d6',
      'editorBracketMatch.background':      '#0a84ff20',
      'editorBracketMatch.border':          '#0a84ff60',
      'editorGutter.background':            '#ffffff',
      'editor.findMatchBackground':         '#ffd60a50',
      'editor.findMatchHighlightBackground':'#ffd60a25',
      'editorError.foreground':             '#ff3b30',
      'editorWarning.foreground':           '#ff9500',
      'editorInfo.foreground':              '#0a84ff',
      'scrollbarSlider.background':         '#00000015',
      'scrollbarSlider.hoverBackground':    '#00000025',
      'scrollbarSlider.activeBackground':   '#00000035',
    }
  })

  const dispatch = createEventDispatcher()
  let {
    code,
    language = 'rust',
    fontSize = 13,
    fontFamily = 'Menlo',
    tabSize = 4,
    theme = 'playground-dark',
    diagnostics = [],
    onSave = () => {},
    onRun = () => {},
    onNew = () => {},
  }: {
    code: string
    language?: string
    fontSize?: number
    fontFamily?: string
    tabSize?: number
    theme?: string
    diagnostics?: any[]
    onSave?: () => void
    onRun?: () => void
    onNew?: () => void
  } = $props()

  let container: HTMLDivElement
  let editor: monaco.editor.IStandaloneCodeEditor
  // Guard: prevent programmatic setValue() calls from triggering the change dispatch.
  // Monaco fires onDidChangeModelContent even when we call setValue ourselves (e.g. on
  // tab switch). Without this flag every tab open immediately marks the tab dirty.
  let ignoreNextChange = false

  onMount(() => {
    editor = monaco.editor.create(container, {
      value: code,
      language,
      theme,
      fontSize,
      fontFamily: `'${fontFamily}', monospace`,
      fontLigatures: false,
      lineNumbers: 'on',
      lineNumbersMinChars: 3,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize,
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
      if (ignoreNextChange) { ignoreNextChange = false; return }
      dispatch('change', editor.getValue())
    })

    // Route keyboard shortcuts through our callbacks so Monaco doesn't swallow them.
    // addCommand() calls stopPropagation internally — empty handlers silently drop events.
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => onSave())
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyR, () => onRun())
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyN, () => onNew())
  })

  // Sync when user switches playground (code prop changes from outside).
  // Set ignoreNextChange BEFORE setValue so the onDidChangeModelContent listener
  // skips the programmatic update and doesn't mark the tab dirty.
  $effect(() => {
    if (editor && editor.getValue() !== code) {
      ignoreNextChange = true
      editor.getModel()?.setValue(code)
    }
  })

  // Sync language when tab type changes (e.g. switching to Cargo.toml)
  $effect(() => {
    const lang = language
    const model = editor?.getModel()
    if (model && model.getLanguageId() !== lang) {
      monaco.editor.setModelLanguage(model, lang)
    }
  })

  // Sync editor settings when they change (e.g. from Settings panel)
  $effect(() => {
    if (!editor) return
    editor.updateOptions({
      fontSize,
      fontFamily: `'${fontFamily}', monospace`,
      tabSize,
    })
    editor.getModel()?.updateOptions({ tabSize })
  })

  // Sync theme when it changes
  $effect(() => {
    if (!editor) return
    monaco.editor.setTheme(theme)
  })

  // Set Monaco markers from cargo check diagnostics
  $effect(() => {
    if (!editor) return
    const model = editor.getModel()
    if (!model) return
    const markers: monaco.editor.IMarkerData[] = diagnostics.map(d => ({
      severity: d.severity === 'warning'
        ? monaco.MarkerSeverity.Warning
        : monaco.MarkerSeverity.Error,
      message: d.message,
      startLineNumber: d.line,
      startColumn: d.col,
      endLineNumber: d.end_line,
      endColumn: d.end_col,
    }))
    monaco.editor.setModelMarkers(model, 'cargo-check', markers)
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
