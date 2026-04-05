<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
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

  // ── Custom Rust theme — warm, earthy tones inspired by oxidized metal ──────
  monaco.editor.defineTheme('playground-rust', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'comment',            foreground: '7a6b5d', fontStyle: 'italic' },
      { token: 'keyword',            foreground: 'ce422b' },   // Rust red
      { token: 'type',               foreground: 'd4a03c' },   // oxidized gold
      { token: 'type.identifier',    foreground: 'd4a03c' },
      { token: 'string',             foreground: '6b9e3c' },   // patina green
      { token: 'string.escape',      foreground: '8bba5a' },
      { token: 'number',             foreground: 'c87832' },   // copper
      { token: 'number.float',       foreground: 'c87832' },
      { token: 'operator',           foreground: 'e8d5c4' },   // parchment
      { token: 'delimiter',          foreground: '8a7566' },
      { token: 'attribute',          foreground: 'b07840' },   // bronze
      { token: 'macro',              foreground: 'e05a3a' },   // bright rust
      { token: 'identifier',         foreground: 'e8d5c4' },
    ],
    colors: {
      'editor.background':                  '#1a1210',
      'editor.foreground':                  '#e8d5c4',
      'editorLineNumber.foreground':        '#3d3028',
      'editorLineNumber.activeForeground':  '#8a7566',
      'editor.lineHighlightBackground':     '#241c18',
      'editor.lineHighlightBorder':         '#00000000',
      'editor.selectionBackground':         '#ce422b35',
      'editor.inactiveSelectionBackground': '#ce422b20',
      'editorCursor.foreground':            '#ce422b',
      'editorIndentGuide.background':       '#2e2420',
      'editorIndentGuide.activeBackground': '#3d3028',
      'editorBracketMatch.background':      '#ce422b25',
      'editorBracketMatch.border':          '#ce422b60',
      'editorGutter.background':            '#1a1210',
      'editor.findMatchBackground':         '#d4a03c40',
      'editor.findMatchHighlightBackground':'#d4a03c20',
      'editorError.foreground':             '#e05a3a',
      'editorWarning.foreground':           '#d4a03c',
      'editorInfo.foreground':              '#c87832',
      'scrollbarSlider.background':         '#ce825a20',
      'scrollbarSlider.hoverBackground':    '#ce825a35',
      'scrollbarSlider.activeBackground':   '#ce825a50',
    }
  })

  // ── Custom Sea Green theme — cool, oceanic tones inspired by C ─────────────
  monaco.editor.defineTheme('playground-seagreen', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'comment',            foreground: '5d7a74', fontStyle: 'italic' },
      { token: 'keyword',            foreground: '44aa99' },   // sea green
      { token: 'type',               foreground: 'd4b83c' },   // sandy gold
      { token: 'type.identifier',    foreground: 'd4b83c' },
      { token: 'string',             foreground: '6bae8e' },   // moss green
      { token: 'string.escape',      foreground: '8bccaa' },
      { token: 'number',             foreground: '3299aa' },   // ocean blue
      { token: 'number.float',       foreground: '3299aa' },
      { token: 'operator',           foreground: 'd0e8e4' },   // seafoam
      { token: 'delimiter',          foreground: '668a84' },
      { token: 'attribute',          foreground: '40887a' },   // deep teal
      { token: 'macro',              foreground: '5cc4b2' },   // bright sea green
      { token: 'identifier',         foreground: 'd0e8e4' },
    ],
    colors: {
      'editor.background':                  '#0e1a18',
      'editor.foreground':                  '#d0e8e4',
      'editorLineNumber.foreground':        '#283d3a',
      'editorLineNumber.activeForeground':  '#668a84',
      'editor.lineHighlightBackground':     '#152422',
      'editor.lineHighlightBorder':         '#00000000',
      'editor.selectionBackground':         '#44aa9935',
      'editor.inactiveSelectionBackground': '#44aa9920',
      'editorCursor.foreground':            '#44aa99',
      'editorIndentGuide.background':       '#1e2e2c',
      'editorIndentGuide.activeBackground': '#283d3a',
      'editorBracketMatch.background':      '#44aa9925',
      'editorBracketMatch.border':          '#44aa9960',
      'editorGutter.background':            '#0e1a18',
      'editor.findMatchBackground':         '#d4b83c40',
      'editor.findMatchHighlightBackground':'#d4b83c20',
      'editorError.foreground':             '#e05a5a',
      'editorWarning.foreground':           '#d4b83c',
      'editorInfo.foreground':              '#3299aa',
      'scrollbarSlider.background':         '#44aa9920',
      'scrollbarSlider.hoverBackground':    '#44aa9935',
      'scrollbarSlider.activeBackground':   '#44aa9950',
    }
  })

  // ── Custom Zig theme — warm amber tones inspired by Zig's lightning bolt ──
  monaco.editor.defineTheme('playground-zig', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'comment',            foreground: '7a7058', fontStyle: 'italic' },
      { token: 'keyword',            foreground: 'f7a41d' },   // Zig amber
      { token: 'type',               foreground: 'e0c060' },   // warm gold
      { token: 'type.identifier',    foreground: 'e0c060' },
      { token: 'string',             foreground: '8baa3c' },   // olive
      { token: 'string.escape',      foreground: 'a8c858' },
      { token: 'number',             foreground: 'd4903c' },   // copper amber
      { token: 'number.float',       foreground: 'd4903c' },
      { token: 'operator',           foreground: 'e8dcc8' },   // warm cream
      { token: 'delimiter',          foreground: '8a7a60' },
      { token: 'attribute',          foreground: 'c89030' },   // deep gold
      { token: 'macro',              foreground: 'ffbc42' },   // bright amber
      { token: 'identifier',         foreground: 'e8dcc8' },
    ],
    colors: {
      'editor.background':                  '#18150e',
      'editor.foreground':                  '#e8dcc8',
      'editorLineNumber.foreground':        '#3a3420',
      'editorLineNumber.activeForeground':  '#8a7a60',
      'editor.lineHighlightBackground':     '#221e14',
      'editor.lineHighlightBorder':         '#00000000',
      'editor.selectionBackground':         '#f7a41d35',
      'editor.inactiveSelectionBackground': '#f7a41d20',
      'editorCursor.foreground':            '#f7a41d',
      'editorIndentGuide.background':       '#2c271c',
      'editorIndentGuide.activeBackground': '#3a3420',
      'editorBracketMatch.background':      '#f7a41d25',
      'editorBracketMatch.border':          '#f7a41d60',
      'editorGutter.background':            '#18150e',
      'editor.findMatchBackground':         '#f7a41d40',
      'editor.findMatchHighlightBackground':'#f7a41d20',
      'editorError.foreground':             '#e05a3a',
      'editorWarning.foreground':           '#f7a41d',
      'editorInfo.foreground':              '#d4903c',
      'scrollbarSlider.background':         '#f7a41d20',
      'scrollbarSlider.hoverBackground':    '#f7a41d35',
      'scrollbarSlider.activeBackground':   '#f7a41d50',
    }
  })

  // ── Custom Swift theme — warm coral tones inspired by Swift's orange-red ──
  monaco.editor.defineTheme('playground-swift', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'comment',            foreground: '7a5d58', fontStyle: 'italic' },
      { token: 'keyword',            foreground: 'f05138' },   // Swift orange-red
      { token: 'type',               foreground: 'd4903c' },   // warm amber
      { token: 'type.identifier',    foreground: 'd4903c' },
      { token: 'string',             foreground: '5aab6e' },   // garden green
      { token: 'string.escape',      foreground: '78c88a' },
      { token: 'number',             foreground: 'c87050' },   // terra cotta
      { token: 'number.float',       foreground: 'c87050' },
      { token: 'operator',           foreground: 'e8d0c8' },   // warm blush
      { token: 'delimiter',          foreground: '8a6a62' },
      { token: 'attribute',          foreground: 'cc6040' },   // deep coral
      { token: 'macro',              foreground: 'ff6b52' },   // bright coral
      { token: 'identifier',         foreground: 'e8d0c8' },
    ],
    colors: {
      'editor.background':                  '#1a100e',
      'editor.foreground':                  '#e8d0c8',
      'editorLineNumber.foreground':        '#3a2822',
      'editorLineNumber.activeForeground':  '#8a6a62',
      'editor.lineHighlightBackground':     '#241816',
      'editor.lineHighlightBorder':         '#00000000',
      'editor.selectionBackground':         '#f0513835',
      'editor.inactiveSelectionBackground': '#f0513820',
      'editorCursor.foreground':            '#f05138',
      'editorIndentGuide.background':       '#2e201e',
      'editorIndentGuide.activeBackground': '#3a2822',
      'editorBracketMatch.background':      '#f0513825',
      'editorBracketMatch.border':          '#f0513860',
      'editorGutter.background':            '#1a100e',
      'editor.findMatchBackground':         '#d4903c40',
      'editor.findMatchHighlightBackground':'#d4903c20',
      'editorError.foreground':             '#f05138',
      'editorWarning.foreground':           '#d4903c',
      'editorInfo.foreground':              '#c87050',
      'scrollbarSlider.background':         '#f0513820',
      'scrollbarSlider.hoverBackground':    '#f0513835',
      'scrollbarSlider.activeBackground':   '#f0513850',
    }
  })

  // ── Register Zig language (not built into Monaco) ─────────────────────────
  monaco.languages.register({ id: 'zig' })
  monaco.languages.setMonarchTokensProvider('zig', {
    keywords: [
      'addrspace', 'align', 'allowzero', 'and', 'anyframe', 'anytype',
      'asm', 'async', 'await', 'break', 'catch', 'comptime', 'const',
      'continue', 'defer', 'else', 'enum', 'errdefer', 'error', 'export',
      'extern', 'false', 'fn', 'for', 'if', 'inline', 'linksection',
      'noalias', 'nosuspend', 'null', 'opaque', 'or', 'orelse', 'packed',
      'pub', 'resume', 'return', 'struct', 'suspend', 'switch', 'test',
      'threadlocal', 'true', 'try', 'undefined', 'union', 'unreachable',
      'var', 'volatile', 'while',
    ],
    typeKeywords: [
      'i8', 'u8', 'i16', 'u16', 'i32', 'u32', 'i64', 'u64', 'i128', 'u128',
      'isize', 'usize', 'f16', 'f32', 'f64', 'f80', 'f128',
      'bool', 'void', 'noreturn', 'type', 'anyerror', 'comptime_int', 'comptime_float',
    ],
    operators: ['=', '>', '<', '!', '~', '?', ':', '==', '<=', '>=', '!=',
      '&&', '||', '++', '--', '+', '-', '*', '/', '&', '|', '^', '%',
      '<<', '>>', '>>>', '+=', '-=', '*=', '/=', '&=', '|=', '^=',
      '%=', '<<=', '>>=', '>>>='],
    symbols: /[=><!~?:&|+\-*/^%]+/,
    escapes: /\\(?:[abefnrtv\\"']|x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f]+\})/,
    tokenizer: {
      root: [
        [/[a-z_$][\w$]*/, { cases: {
          '@typeKeywords': 'type',
          '@keywords': 'keyword',
          '@default': 'identifier'
        }}],
        [/[A-Z][\w$]*/, 'type.identifier'],
        [/@[a-zA-Z_]\w*/, 'attribute'],
        { include: '@whitespace' },
        [/[{}()[\]]/, '@brackets'],
        [/[<>](?!@symbols)/, '@brackets'],
        [/@symbols/, { cases: { '@operators': 'operator', '@default': '' }}],
        [/\d*\.\d+([eE][-+]?\d+)?/, 'number.float'],
        [/0[xX][0-9a-fA-F_]+/, 'number.hex'],
        [/0[oO][0-7_]+/, 'number.octal'],
        [/0[bB][01_]+/, 'number.binary'],
        [/\d[0-9_]*/, 'number'],
        [/[;,.]/, 'delimiter'],
        [/"([^"\\]|\\.)*$/, 'string.invalid'],
        [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],
        [/'[^\\']'/, 'string'],
        [/'/, 'string.invalid'],
      ],
      string: [
        [/[^\\"]+/, 'string'],
        [/@escapes/, 'string.escape'],
        [/\\./, 'string.escape.invalid'],
        [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
      ],
      whitespace: [
        [/[ \t\r\n]+/, 'white'],
        [/\/\/.*$/, 'comment'],
      ],
    },
  })

  let {
    code,
    language = 'rust',
    fontSize = 13,
    fontFamily = 'Menlo',
    tabSize = 0,
    theme = 'playground-dark',
    diagnostics = [],
    readOnly = false,
    onSave = () => {},
    onRun = () => {},
    onNew = () => {},
    onChange = (_code: string) => {},
  }: {
    code: string
    language?: string
    fontSize?: number
    fontFamily?: string
    tabSize?: number
    theme?: string
    diagnostics?: any[]
    readOnly?: boolean
    onSave?: () => void
    onRun?: () => void
    onNew?: () => void
    onChange?: (code: string) => void
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
      tabSize: tabSize === 0 ? 2 : tabSize,
      detectIndentation: tabSize === 0,
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
      readOnly,
      domReadOnly: readOnly,
      scrollbar: {
        verticalScrollbarSize: 6,
        horizontalScrollbarSize: 6,
        useShadows: false,
      },
    })

    editor.onDidChangeModelContent(() => {
      if (ignoreNextChange) { ignoreNextChange = false; return }
      onChange(editor.getValue())
    })

    // Route keyboard shortcuts through our callbacks so Monaco doesn't swallow them.
    // addCommand() calls stopPropagation internally — empty handlers silently drop events.
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => { if (!readOnly) onSave() })
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
    const effectiveTabSize = tabSize === 0 ? 2 : tabSize
    editor.updateOptions({
      fontSize,
      fontFamily: `'${fontFamily}', monospace`,
      tabSize: effectiveTabSize,
      detectIndentation: tabSize === 0,
    })
    editor.getModel()?.updateOptions({ tabSize: effectiveTabSize })
  })

  // Sync readOnly when it changes (e.g. switching between book and user projects)
  $effect(() => {
    if (!editor) return
    editor.updateOptions({ readOnly, domReadOnly: readOnly })
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
