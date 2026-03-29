<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte'
  import * as monaco from 'monaco-editor'
  import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'

  // Monaco needs to know how to load web workers.
  // Rust only needs the base editor worker (no language server worker needed).
  // @ts-ignore
  self.MonacoEnvironment = {
    getWorker() {
      return new editorWorker()
    }
  }

  const dispatch = createEventDispatcher()
  let { code }: { code: string } = $props()

  let container: HTMLDivElement
  let editor: monaco.editor.IStandaloneCodeEditor

  onMount(() => {
    editor = monaco.editor.create(container, {
      value: code,
      language: 'rust',
      theme: 'vs-dark',
      fontSize: 14,
      fontFamily: "'Menlo', 'Monaco', 'Courier New', monospace",
      lineNumbers: 'on',
      lineNumbersMinChars: 3,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize: 4,
      insertSpaces: true,
      wordWrap: 'off',
      padding: { top: 16, bottom: 16 },
      renderLineHighlight: 'line',
      cursorBlinking: 'smooth',
      smoothScrolling: true,
      bracketPairColorization: { enabled: true },
      'semanticHighlighting.enabled': true,
    })

    editor.onDidChangeModelContent(() => {
      dispatch('change', editor.getValue())
    })

    // Prevent Cmd+R / Cmd+S being swallowed by Monaco
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyR, () => {})
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {})
  })

  // Sync when user switches playground
  $effect(() => {
    if (editor && editor.getValue() !== code) {
      editor.setValue(code)
      // Reset undo history so Ctrl+Z doesn't undo back into the previous file
      editor.getModel()?.setValue(code)
    }
  })

  onDestroy(() => {
    editor?.dispose()
  })
</script>

<div class="editor-container" bind:this={container}></div>

<style>
  .editor-container {
    flex: 1;
    overflow: hidden;
  }
</style>
