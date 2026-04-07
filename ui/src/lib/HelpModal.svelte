<script lang="ts">
  import { open as shellOpen } from '@tauri-apps/plugin-shell'
  import appIcon from './app-icon.png'
  import { getLang, allLanguages, type ProjectType } from './languages'
  import type { EditionConfig } from './editions'

  let { onclose, enabledLanguages = ['rust', 'clang', 'zig', 'swift'], edition }: {
    onclose: () => void
    enabledLanguages?: string[]
    edition: EditionConfig
  } = $props()

  let activeSection: string = $state('overview')

  // Derive book label from actual book count for enabled languages
  const enabledBooks = allLanguages().filter(l => enabledLanguages.includes(l.type) && l.book)
  const bookCount = enabledBooks.length
  const bookShortNames: Record<string, string> = {
    rust: 'The Rust Book',
    clang: 'K&R C Book',
    swift: 'The Swift Book',
  }
  const singleBookName = bookCount === 1 ? (bookShortNames[enabledBooks[0].type] ?? enabledBooks[0].book!.commandLabel) : null
  const booksLabel = singleBookName ?? 'Book Examples'

  const allSections = [
    { id: 'overview', label: 'Overview' },
    { id: 'languages', label: edition.isSingleLanguage ? ({
      rust: 'Rust / Cargo', clang: 'C / Clang', zig: 'Zig', swift: 'Swift'
    }[edition.defaultProjectType] ?? 'Language') : 'Languages' },
    { id: 'playgrounds', label: 'Playgrounds' },
    { id: 'projects', label: 'Projects' },
    { id: 'console', label: 'Console' },
    { id: 'content', label: 'Content Files' },
    { id: 'shortcuts', label: 'Shortcuts' },
    ...(bookCount > 0 ? [{ id: 'books', label: booksLabel }] : []),
    { id: 'security', label: 'Security' },
  ]
  const sections = allSections

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose()
  }
</script>

<svelte:window onkeydown={handleKey} />

<!-- Backdrop -->
<div class="backdrop" onclick={onclose} aria-hidden="true"></div>

<div class="modal" role="dialog" aria-modal="true" aria-label="Rustic Playground Help">

  <!-- Sidebar navigation -->
  <nav class="help-nav">
    <div class="nav-header">
      <img class="nav-icon" src={appIcon} alt="" width="32" height="32" />
      <span class="nav-title">{edition.displayName}</span>
    </div>
    <ul class="nav-list">
      {#each sections as s (s.id)}
        <li>
          <button
            class="nav-item"
            class:active={activeSection === s.id}
            onclick={() => activeSection = s.id}
          >{s.label}</button>
        </li>
      {/each}
    </ul>
  </nav>

  <!-- Content area -->
  <div class="help-content">
    <button class="close-btn" onclick={onclose} aria-label="Close">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
      </svg>
    </button>

    {#if activeSection === 'overview'}
      <h1>Getting Started</h1>
      <p class="lead">
        {edition.displayName} is a macOS desktop app for running code experiments — inspired
        by Swift Playgrounds. Write code, press <kbd>⌘R</kbd>, see output stream live.
      </p>
      <p class="lead-meta">
        Requires <strong>macOS 14 Sonoma or later</strong>.
      </p>

      <div class="card-grid">
        <div class="card">
          <span class="card-icon">⌘R</span>
          <h3>Run</h3>
          <p>Compile and execute your playground. Output streams to the Console panel in real time.</p>
        </div>
        <div class="card">
          <span class="card-icon">⌘,</span>
          <h3>Settings</h3>
          <p>{edition.isSingleLanguage ? 'Choose themes, fonts, and manage your toolchain.' : 'Choose languages, themes, fonts, and manage toolchains.'} Open from the gear button or menu.</p>
        </div>
        <div class="card">
          <span class="card-icon">📖</span>
          <h3>Learn</h3>
          <p>Load book examples from the Learn menu. Read-only reference material you can copy and modify.</p>
        </div>
      </div>

      <h2>How it works</h2>
      <ol class="steps">
        <li>Choose or create a <strong>project</strong> — an isolated workspace for your experiments.</li>
        <li>Create a <strong>playground</strong> — a source file with a <code>main</code> function.</li>
        <li>Write code in the editor. Press <kbd>⌘R</kbd> to run.</li>
        <li>See compiler output and program output stream live in the <strong>Console</strong>.</li>
        <li>If your program reads from stdin, a text field appears automatically.</li>
      </ol>

    {:else if activeSection === 'languages'}
      <h1>{edition.isSingleLanguage ? sections.find(s => s.id === 'languages')?.label : 'Supported Languages'}</h1>
      <p class="lead">
        {#if edition.isSingleLanguage}
          Your locally installed toolchain powers all compilation and execution.
        {:else}
          {edition.displayName} supports four languages. Enable or disable them in Settings.
          Each language uses your locally installed toolchain.
        {/if}
      </p>

      <div class="lang-cards">
        {#if enabledLanguages.includes('rust')}
        <div class="lang-card">
          <div class="lang-header">
            <span class="lang-dot" style="background: var(--rust-orange)"></span>
            <h3>Rust</h3>
          </div>
          <p>
            First-class support. Each project is a Cargo package.
            Playgrounds are binary targets in <code>src/bin/</code>.
            Live error checking via <code>cargo check</code>.
            Dependencies managed via <code>Cargo.toml</code>.
          </p>
          <div class="lang-meta">
            <span>Toolchain: <code>cargo</code> + <code>rustc</code></span>
            <span>Install: <button class="link-btn" onclick={() => shellOpen('https://rustup.rs')}>rustup.rs</button></span>
          </div>
        </div>
        {/if}

        {#if enabledLanguages.includes('clang')}
        <div class="lang-card">
          <div class="lang-header">
            <span class="lang-dot" style="background: #4a9"></span>
            <h3>C / C++</h3>
          </div>
          <p>
            Compile and run with Clang. Separate C and C++ flags per project.
            Each playground is a standalone <code>.c</code> or <code>.cpp</code> file.
          </p>
          <div class="lang-meta">
            <span>Toolchain: <code>clang</code> / <code>clang++</code></span>
            <span>Install: <code>xcode-select --install</code></span>
          </div>
        </div>
        {/if}

        {#if enabledLanguages.includes('zig')}
        <div class="lang-card">
          <div class="lang-header">
            <span class="lang-dot" style="background: #f7a41d"></span>
            <h3>Zig (v0.15)</h3>
            <span class="exp-badge">experimental</span>
          </div>
          <p>
            Single-step execution via <code>zig run</code>.
            Targets Zig <strong>0.15.x</strong> — other versions may have breaking
            stdlib changes and are flagged with a warning.
          </p>
          <div class="lang-meta">
            <span>Toolchain: <code>zig</code></span>
            <span>Install: <code>brew install zig</code></span>
          </div>
        </div>
        {/if}

        {#if enabledLanguages.includes('swift')}
        <div class="lang-card">
          <div class="lang-header">
            <span class="lang-dot" style="background: #f05138"></span>
            <h3>Swift</h3>
          </div>
          <p>
            Two-step compile and run via <code>swiftc</code>.
            Ships free with Xcode Command Line Tools (same install as Clang).
          </p>
          <div class="lang-meta">
            <span>Toolchain: <code>swiftc</code></span>
            <span>Install: <code>xcode-select --install</code></span>
          </div>
        </div>
        {/if}
      </div>

    {:else if activeSection === 'playgrounds'}
      <h1>Working with Playgrounds</h1>
      <p class="lead">
        A playground is a single source file with a <code>main</code> function.
        Each one compiles and runs independently.
      </p>

      <h2>Create</h2>
      <p>Click <strong>+</strong> in the sidebar or press <kbd>⌘N</kbd>. Choose from starter templates or start blank.</p>

      <h2>Run</h2>
      <p>Press <kbd>⌘R</kbd> or click the <strong>Run</strong> button. The file is saved automatically, then compiled and executed. The Console shows each stage: <strong>Saving → Compiling → Running → Done</strong>.</p>

      <h2>Stop</h2>
      <p>Press <kbd>⌘.</kbd> or click <strong>Stop</strong>. This kills the entire process tree.</p>

      <h2>Save</h2>
      <p>Press <kbd>⌘S</kbd>. Unsaved changes show a dot on the tab. Files are also saved automatically before each run.</p>

      <h2>Rename / Delete / Duplicate</h2>
      <p>Right-click any playground in the sidebar for these actions. Book playgrounds are read-only — use <strong>Copy to Project</strong> to create an editable copy.</p>

      <h2>Templates</h2>
      <p>Each language includes starter templates: Hello World, CLI Input, Structs, Error Handling, and more. Templates auto-add required dependencies for Rust projects.</p>

    {:else if activeSection === 'projects'}
      <h1>Projects</h1>
      <p class="lead">
        A project is an isolated workspace — its own config, dependencies, and set of playgrounds.
        Use projects to keep unrelated experiments separate.
      </p>

      <h2>Managing Projects</h2>
      <p>
        Click the project name in the toolbar to open the project switcher. From here you can
        switch, create, rename, duplicate, or delete projects. The same actions are available
        in the <strong>Project</strong> menu.
      </p>

      <h2>Project Types</h2>
      <p>
        Each project has a language type set at creation. Rust projects use <code>Cargo.toml</code>
        for dependency management. Other languages use <code>rustic.toml</code> for project config
        and compiler flags.
      </p>

      <h2>Dependencies (Rust)</h2>
      <p>
        All Rust playgrounds in a project share one <code>Cargo.toml</code>. Click it at the bottom
        of the sidebar to edit directly, or use the dependency manager in the toolbar.
      </p>
      <div class="code-block">
        <pre>[dependencies]
serde = &#123; version = "1", features = ["derive"] &#125;
rand = "0.8"
tokio = &#123; version = "1", features = ["full"] &#125;</pre>
      </div>

      <h2>Compiler Flags (C/C++, Zig, Swift)</h2>
      <p>
        Non-Rust projects have per-project compiler flags visible in the sidebar.
        These are saved to <code>rustic.toml</code> and applied on every run.
      </p>

      <h2>Storage</h2>
      <p>
        Projects are stored at:<br/>
        <code>~/Library/Application Support/com.rustic-playground.app/projects/</code>
      </p>

    {:else if activeSection === 'console'}
      <h1>Console</h1>
      <p class="lead">
        The Console panel shows compiler output and program output for each run.
      </p>

      <h2>Run Blocks</h2>
      <p>
        Each run creates a collapsible block that progresses through stages:
        <strong>Saving</strong> (file written to disk) → <strong>Compiling</strong> (compiler output) → <strong>Running</strong> (program output).
        Click the block header to collapse or expand previous runs.
      </p>

      <h2>Interactive Input (stdin)</h2>
      <p>
        If your playground reads from <code>stdin</code>, a text field appears automatically
        while the program is running. Type your input and press <kbd>Enter</kbd>.
        Prompts without a trailing newline (like <code>print!("name? ")</code>) appear immediately.
      </p>

      <h2>Features</h2>
      <ul>
        <li><strong>ANSI colors</strong> — colored output renders correctly</li>
        <li><strong>Timestamps</strong> — hover any line to see its timestamp</li>
        <li><strong>Copy</strong> — copy a run's output to clipboard</li>
        <li><strong>Clear</strong> — remove all run history</li>
      </ul>

    {:else if activeSection === 'content'}
      <h1>Content Files</h1>
      <p class="lead">
        Each project has a content folder for runtime assets — CSV data, JSON configs, text
        fixtures, images, anything your playground needs at runtime.
      </p>

      <h2>Accessing Files in Code</h2>
      <p>Use the <code>PLAYGROUND_CONTENT</code> environment variable:</p>
      <div class="code-block">
        <pre>use std::&#123;env, fs&#125;;

fn main() &#123;
    let dir = env::var("PLAYGROUND_CONTENT").unwrap_or_default();
    let data = fs::read_to_string(format!("&#123;dir&#125;/data.csv")).unwrap();
    println!("&#123;data&#125;");
&#125;</pre>
      </div>

      <h2>Managing Files</h2>
      <ul>
        <li><strong>Add</strong> — click <code>[+ Add File]</code> in the sidebar content section</li>
        <li><strong>Import</strong> — drag a file from Finder onto the content area in the sidebar</li>
        <li><strong>Edit text files</strong> — click to open as an editor tab</li>
        <li><strong>Open binary / image</strong> — click to open with the default macOS app</li>
        <li><strong>Rename / Delete / Reveal</strong> — right-click the file</li>
        <li><strong>Size limit</strong> — imported files must be under 10 MB</li>
      </ul>

    {:else if activeSection === 'shortcuts'}
      <h1>Keyboard Shortcuts</h1>

      <h2>Run & Edit</h2>
      <table>
        <tbody>
          <tr><td><kbd>⌘R</kbd></td><td>Run the active playground</td></tr>
          <tr><td><kbd>⌘.</kbd></td><td>Stop the running process</td></tr>
          <tr><td><kbd>⌘S</kbd></td><td>Save the active file</td></tr>
          <tr><td><kbd>⌘N</kbd></td><td>New playground</td></tr>
          <tr><td><kbd>⌘W</kbd></td><td>Close the active tab</td></tr>
        </tbody>
      </table>

      <h2>Project & Window</h2>
      <table>
        <tbody>
          <tr><td><kbd>⌘⇧N</kbd></td><td>New project</td></tr>
          <tr><td><kbd>⌘,</kbd></td><td>Settings</td></tr>
          <tr><td><kbd>⌘⇧/</kbd></td><td>This help</td></tr>
        </tbody>
      </table>

    {:else if activeSection === 'books'}
      <h1>{booksLabel}</h1>
      <p class="lead">
        {edition.isSingleLanguage
          ? `Learn from curated example projects based on ${singleBookName}. Load from the Learn menu or the Welcome Wizard.`
          : 'Learn from curated example projects based on popular programming books. Load them from the Learn menu or the Welcome Wizard.'}
      </p>

      <div class="book-list">
        {#if enabledLanguages.includes('rust')}
        <div class="book-item">
          <h3><button class="link-btn" onclick={() => shellOpen('https://doc.rust-lang.org/book/')}>The Rust Programming Language (2024 Edition)</button></h3>
          <p>21 chapters covering the full curriculum of <em>The Rust Programming Language</em>
            by Steve Klabnik and Carol Nichols.</p>
          <div class="book-meta">
            <span>License: MIT / Apache-2.0</span>
            <span><button class="link-btn" onclick={() => shellOpen('https://doc.rust-lang.org/book/')}>Read online</button></span>
          </div>
        </div>
        {/if}
        {#if enabledLanguages.includes('clang')}
        <div class="book-item">
          <h3>The C Programming Language (K&R, 2nd Ed.)</h3>
          <p>8 chapters based on <em>The C Programming Language</em> (2nd Edition, 1988) by Brian W. Kernighan and Dennis M. Ritchie.</p>
        </div>
        {/if}
        {#if enabledLanguages.includes('swift')}
        <div class="book-item">
          <h3><button class="link-btn" onclick={() => shellOpen('https://docs.swift.org/swift-book/')}>The Swift Programming Language (Swift 6.1)</button></h3>
          <p>8 chapters based on <em>The Swift Programming Language</em> by Apple Inc.</p>
          <div class="book-meta">
            <span><button class="link-btn" onclick={() => shellOpen('https://docs.swift.org/swift-book/')}>Read online</button></span>
          </div>
        </div>
        {/if}
      </div>

      <h2>Read-Only</h2>
      <p>
        Book projects are non-editable reference material. The editor is read-only and
        rename/delete are disabled. To experiment with an example, right-click and choose
        <strong>Copy to Project</strong> to create an editable copy in your own project.
      </p>

      <h2>{bookCount === 1 ? 'Managing' : 'Managing Books'}</h2>
      <p>
        Load or remove {bookCount === 1 ? 'the book' : 'books'} from <strong>Settings → {booksLabel}</strong>. Removing {bookCount === 1 ? 'it' : 'a book'} deletes
        the projects from disk. You can re-load at any time.
      </p>

      <p class="footnote">
        Playground code is original educational material — not verbatim from the books.
        Each chapter includes an <code>attribution.md</code> in its Files folder.
        {edition.displayName} is not affiliated with or endorsed by the original authors.
      </p>

    {:else if activeSection === 'security'}
      <h1>Security</h1>
      <div class="warn-banner">
        <h2>This app is intentionally not sandboxed</h2>
        <p>
          Like Xcode, VS Code, and Terminal, {edition.displayName} compiles and executes arbitrary
          code using your local {edition.isSingleLanguage ? 'toolchain' : 'toolchains'} ({#if enabledLanguages.includes('rust')}<code>cargo</code>{/if}{#if enabledLanguages.includes('clang')}{#if enabledLanguages.includes('rust')}, {/if}<code>clang</code>{/if}{#if enabledLanguages.includes('zig')}{#if enabledLanguages.includes('rust') || enabledLanguages.includes('clang')}, {/if}<code>zig</code>{/if}{#if enabledLanguages.includes('swift')}{#if enabledLanguages.length > 1}, {/if}<code>swiftc</code>{/if}).
        </p>
        <p>
          Any playground you run has <strong>full access</strong> to your filesystem, network,
          and processes — the same as code you'd run in Terminal.
        </p>
      </div>

      <h2>Safety Measures</h2>
      <ul>
        <li>Playground names are validated — path traversal is blocked at the API layer</li>
        <li>Tauri IPC only accepts calls from the app's own WebView origin</li>
        <li>Toolchains are invoked via absolute paths, not shell string interpolation</li>
        <li>Playground runs use a separate target directory to avoid lock conflicts</li>
      </ul>

      <h2>Your Responsibility</h2>
      <p>Only run code you wrote or fully understand. Do not run code or binaries from untrusted sources.</p>
    {/if}
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
    width: min(820px, calc(100vw - 40px));
    height: min(600px, calc(100vh - 80px));
    display: flex; flex-direction: row;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    box-shadow: 0 24px 80px rgba(0,0,0,0.7), 0 4px 16px rgba(0,0,0,0.4);
    overflow: hidden;
  }

  /* ── Sidebar nav (Apple User Guide style) ── */
  .help-nav {
    width: 180px; flex-shrink: 0;
    display: flex; flex-direction: column;
    background: var(--bg-sidebar, rgba(0,0,0,0.15));
    border-right: 1px solid var(--border);
    padding: 16px 0;
    overflow-y: auto;
  }
  .nav-header {
    display: flex; align-items: center; gap: 8px;
    padding: 0 14px 14px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 8px;
  }
  .nav-icon {
    width: 32px; height: 32px;
    border-radius: 7px;
    box-shadow: 0 1px 4px rgba(0,0,0,0.3);
  }
  .nav-title {
    font-size: 11px; font-weight: 600; color: var(--text);
    line-height: 1.3;
  }
  .nav-list {
    list-style: none; margin: 0; padding: 0;
    display: flex; flex-direction: column; gap: 1px;
  }
  .nav-item {
    display: block; width: 100%;
    padding: 6px 14px;
    font-size: 12px; font-weight: 500; color: var(--text-secondary);
    text-align: left;
    background: none; border: none; cursor: pointer;
    font-family: inherit;
    border-radius: 0;
    transition: background 0.1s, color 0.1s;
  }
  .nav-item:hover { background: var(--bg-hover); color: var(--text); }
  .nav-item.active {
    background: rgba(var(--accent-rgb), 0.15);
    color: var(--accent);
    font-weight: 600;
  }

  /* ── Content area ── */
  .help-content {
    flex: 1; overflow-y: auto;
    padding: 28px 32px 32px;
    position: relative;
  }
  .close-btn {
    position: absolute; top: 12px; right: 12px;
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%; color: var(--text-tertiary);
    transition: background 0.1s, color 0.1s;
    z-index: 1;
  }
  .close-btn:hover { background: var(--bg-hover); color: var(--text); }

  /* ── Typography ── */
  h1 {
    font-size: 20px; font-weight: 600; color: var(--text);
    margin: 0 0 8px; letter-spacing: -0.01em;
  }
  h2 {
    font-size: 13px; font-weight: 600; color: var(--text);
    margin: 20px 0 6px; letter-spacing: 0;
    text-transform: none;
  }
  h2:first-of-type { margin-top: 16px; }
  h3 { font-size: 13px; font-weight: 600; color: var(--text); margin: 0; }

  p { font-size: 13px; color: var(--text-secondary); line-height: 1.65; margin: 0 0 4px; }
  .lead { font-size: 13px; color: var(--text-secondary); line-height: 1.7; margin-bottom: 16px; }
  .lead-meta { font-size: 11px; color: var(--text-tertiary); margin: -8px 0 16px 0; }
  .footnote { font-size: 12px; color: var(--text-tertiary); font-style: italic; margin-top: 16px; }

  strong { color: var(--text); font-weight: 600; }
  em { font-style: italic; }

  ul, ol { margin: 4px 0; padding-left: 20px; display: flex; flex-direction: column; gap: 4px; }
  li { font-size: 13px; color: var(--text-secondary); line-height: 1.55; }

  .steps { gap: 8px; }
  .steps li { padding-left: 4px; }

  /* ── Card grid (overview) ── */
  .card-grid {
    display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px;
    margin-bottom: 8px;
  }
  .card {
    background: rgba(0,0,0,0.12);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
    display: flex; flex-direction: column; gap: 6px;
  }
  .card-icon { font-size: 18px; }
  .card h3 { font-size: 12px; }
  .card p { font-size: 11px; line-height: 1.5; margin: 0; }

  /* ── Language cards ── */
  .lang-cards { display: flex; flex-direction: column; gap: 12px; }
  .lang-card {
    background: rgba(0,0,0,0.1);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
    display: flex; flex-direction: column; gap: 8px;
  }
  .lang-header { display: flex; align-items: center; gap: 8px; }
  .lang-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
  .lang-card p { font-size: 12px; line-height: 1.6; margin: 0; }
  .lang-meta {
    display: flex; gap: 16px; flex-wrap: wrap;
    font-size: 11px; color: var(--text-tertiary);
  }
  .exp-badge {
    font-size: 9px; font-weight: 700; letter-spacing: 0.03em;
    text-transform: uppercase;
    background: rgba(247, 164, 29, 0.15);
    color: #f7a41d;
    border: 1px solid rgba(247, 164, 29, 0.3);
    border-radius: 3px;
    padding: 1px 5px;
  }

  /* ── Book list ── */
  .book-list { display: flex; flex-direction: column; gap: 12px; margin-bottom: 8px; }
  .book-item {
    background: rgba(0,0,0,0.1);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px 16px;
    display: flex; flex-direction: column; gap: 4px;
  }
  .book-item p { font-size: 12px; line-height: 1.55; margin: 0; }
  .book-meta {
    display: flex; gap: 16px; flex-wrap: wrap;
    font-size: 11px; color: var(--text-tertiary);
    margin-top: 2px;
  }

  /* ── Keyboard shortcut table ── */
  table { border-collapse: collapse; font-size: 12px; width: 100%; }
  td {
    padding: 5px 12px 5px 0;
    color: var(--text-secondary);
    vertical-align: middle;
    border-bottom: 1px solid rgba(255,255,255,0.04);
  }
  td:first-child { white-space: nowrap; width: 80px; }

  kbd {
    display: inline-block;
    font-family: var(--font-ui);
    font-size: 11px; font-weight: 600;
    color: var(--text);
    background: rgba(255,255,255,0.08);
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    padding: 2px 6px;
    line-height: 1.5;
    box-shadow: 0 1px 0 rgba(0,0,0,0.4);
  }

  code {
    font-family: var(--font-mono); font-size: 12px;
    color: var(--text);
    background: rgba(255,255,255,0.06);
    border: 1px solid rgba(255,255,255,0.08);
    border-radius: 3px;
    padding: 1px 5px;
  }

  .code-block {
    background: rgba(0,0,0,0.3);
    border: 1px solid rgba(255,255,255,0.07);
    border-radius: 6px;
    padding: 12px 14px;
    overflow-x: auto;
    margin: 4px 0;
  }
  .code-block pre {
    margin: 0;
    font-family: var(--font-mono); font-size: 12px;
    color: var(--text); line-height: 1.7;
    white-space: pre;
  }

  /* ── Security warning ── */
  .warn-banner {
    background: rgba(255, 80, 60, 0.06);
    border: 1px solid rgba(255, 80, 60, 0.2);
    border-radius: 8px;
    padding: 14px 16px;
    margin-bottom: 8px;
  }
  .warn-banner h2 {
    font-size: 13px; font-weight: 600;
    color: var(--red);
    margin: 0 0 8px;
  }
  .warn-banner p { font-size: 12px; }

  /* ── Link button ── */
  .link-btn {
    color: var(--accent); font-size: inherit;
    text-decoration: none; cursor: pointer;
    background: none; border: none; padding: 0;
    font-family: inherit; font-weight: inherit;
  }
  .link-btn:hover { text-decoration: underline; }
</style>
