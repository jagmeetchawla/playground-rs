<script lang="ts">
  let { onclose }: { onclose: () => void } = $props()

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose()
  }
</script>

<svelte:window onkeydown={handleKey} />

<!-- Backdrop -->
<div class="backdrop" onclick={onclose} aria-hidden="true"></div>

<div class="modal" role="dialog" aria-modal="true" aria-label="Playground Help">
  <div class="modal-header">
    <div class="header-left">
      <span class="rs-badge">RS</span>
      <span class="modal-title">Rustic Playground Help</span>
    </div>
    <button class="close-btn" onclick={onclose} aria-label="Close">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  <div class="modal-body">

    <!-- Getting started -->
    <section>
      <h2>Getting Started</h2>
      <p>
        Rustic Playground is a macOS desktop app for running Rust experiments — inspired by Swift Playgrounds.
        Each <strong>playground</strong> is a <code>.rs</code> file with a <code>fn main()</code>.
        Hit <kbd>⌘R</kbd> to compile and run it; output streams live to the Console panel below.
      </p>
    </section>

    <!-- Keyboard shortcuts -->
    <section>
      <h2>Keyboard Shortcuts</h2>
      <table>
        <tbody>
          <tr><td><kbd>⌘R</kbd></td><td>Run the active playground</td></tr>
          <tr><td><kbd>⌘.</kbd></td><td>Stop the running process</td></tr>
          <tr><td><kbd>⌘S</kbd></td><td>Save the active file</td></tr>
          <tr><td><kbd>⌘N</kbd></td><td>New playground</td></tr>
          <tr><td><kbd>⌘W</kbd></td><td>Close the active tab</td></tr>
          <tr><td><kbd>⌘⇧N</kbd></td><td>New project</td></tr>
          <tr><td><kbd>⌘⇧/</kbd></td><td>This help</td></tr>
        </tbody>
      </table>
    </section>

    <!-- Playgrounds -->
    <section>
      <h2>Working with Playgrounds</h2>
      <p>
        Playgrounds live in the left sidebar. Each one is a <code>.rs</code> file inside the active
        project's <code>src/bin/</code> folder — a standard Cargo binary target.
      </p>
      <ul>
        <li><strong>New</strong> — click <code>+</code> in the sidebar or press <kbd>⌘N</kbd></li>
        <li><strong>Rename / Delete / Duplicate</strong> — right-click the playground in the sidebar</li>
        <li><strong>Run</strong> — <kbd>⌘R</kbd> or the ▶ Run button in the toolbar</li>
        <li><strong>Stop</strong> — <kbd>⌘.</kbd> or ■ Stop — kills the entire cargo process tree</li>
        <li><strong>Save</strong> — <kbd>⌘S</kbd> or the Save button; unsaved changes show a dot on the tab</li>
      </ul>
    </section>

    <!-- Content files -->
    <section>
      <h2>Content Files</h2>
      <p>
        Each playground has its own <strong>content folder</strong> — the same concept as Swift
        Playgrounds' assets bundle. Drop any file you need at runtime there: CSV data, JSON configs,
        text fixtures, images, whatever.
      </p>
      <p>Access content files from your playground code via the injected environment variable:</p>
      <div class="code-block">
        <pre>use std::&#123;env, fs&#125;;

fn main() &#123;
    let dir = env::var("PLAYGROUND_CONTENT").unwrap_or_default();
    let data = fs::read_to_string(format!("&#123;dir&#125;/data.csv")).unwrap();
    println!("&#123;data&#125;");
&#125;</pre>
      </div>
      <ul>
        <li><strong>Add a file</strong> — click <code>[+ Add File]</code> in the expanded content section</li>
        <li><strong>Import from Finder</strong> — drag a file onto the playground row in the sidebar</li>
        <li><strong>Edit a text file</strong> — click it; it opens as an editor tab</li>
        <li><strong>Open a binary / image</strong> — click it; opens with the default macOS app</li>
        <li><strong>Rename / Delete / Reveal</strong> — right-click the file</li>
      </ul>
    </section>

    <!-- Projects -->
    <section>
      <h2>Projects</h2>
      <p>
        A <strong>project</strong> is a self-contained Cargo workspace — its own <code>Cargo.toml</code>
        and set of playgrounds. Use projects to keep unrelated experiments separate
        (e.g. one for algorithms, one for async experiments).
      </p>
      <p>
        Switch, create, rename, or delete projects from the project pill button in the top-left corner,
        or from the <strong>Project</strong> menu.
      </p>
      <p>
        Projects are stored at:<br/>
        <code>~/Library/Application Support/com.rustic-playground.app/projects/</code>
      </p>
    </section>

    <!-- Dependencies -->
    <section>
      <h2>Adding Dependencies</h2>
      <p>
        All playgrounds in a project share one <code>Cargo.toml</code>. Click the
        <strong>Cargo.toml</strong> entry at the bottom of the sidebar to edit it directly in the app.
        Changes take effect on the next run.
      </p>
      <div class="code-block">
        <pre>[dependencies]
serde = &#123; version = "1", features = ["derive"] &#125;
rand = "0.8"
tokio = &#123; version = "1", features = ["full"] &#125;</pre>
      </div>
    </section>

    <!-- Console -->
    <section>
      <h2>Console Output</h2>
      <p>
        Each run appears as a collapsible block in the Console panel. The block shows:
      </p>
      <ul>
        <li><strong>Compiler</strong> — rustc/cargo warnings and errors (dark background box)</li>
        <li><strong>Output</strong> — your program's stdout/stderr</li>
        <li><strong>Interactive input</strong> — if your playground reads from <code>stdin</code>,
          a text field appears while the program is running. Type your input and press
          <kbd>Enter</kbd> to send it to the process</li>
      </ul>
      <p>Click the block header to collapse/expand previous runs. Click <strong>Clear</strong> to remove all history.</p>
    </section>

    <!-- Rust Book attribution -->
    <section class="attribution-section">
      <h2>📖 Rust Book Examples</h2>
      <p>
        The chapter examples loaded via <strong>Help → Load Rust Book Examples…</strong>
        are based on the curriculum of
        <strong>"The Rust Programming Language"</strong>
        by Steve Klabnik and Carol Nichols, with contributions from the Rust Community.
      </p>
      <table class="attr-table">
        <tbody>
          <tr><td>Book (free online)</td><td><a href="https://doc.rust-lang.org/book/" target="_blank" rel="noreferrer">doc.rust-lang.org/book</a></td></tr>
          <tr><td>Source</td><td><a href="https://github.com/rust-lang/book" target="_blank" rel="noreferrer">github.com/rust-lang/book</a></td></tr>
          <tr><td>License</td><td>MIT / Apache-2.0</td></tr>
          <tr><td>Copyright</td><td>Rust Project Developers (2010)</td></tr>
        </tbody>
      </table>
      <p>
        The playground code is original educational Rust written to illustrate each
        chapter's concepts — not verbatim from the book. Rustic Playground is not
        affiliated with or endorsed by the Rust Project.
      </p>
      <p class="attr-note">
        An <code>attribution.md</code> file is placed in every chapter's Files folder for
        offline reference.
      </p>
    </section>

    <!-- Security -->
    <section class="warn-section">
      <h2>⚠ Security Notice</h2>
      <p>
        This app is <strong>intentionally not sandboxed</strong>. Like Xcode or Terminal, it compiles
        and executes arbitrary Rust code using your local <code>cargo</code> and <code>rustc</code> toolchain.
        Any playground you run has full access to your filesystem, network, and processes.
        Only run code you wrote or fully understand.
      </p>
    </section>

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
    width: min(680px, calc(100vw - 40px));
    max-height: calc(100vh - 80px);
    display: flex; flex-direction: column;
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    box-shadow: 0 24px 80px rgba(0,0,0,0.7), 0 4px 16px rgba(0,0,0,0.4);
    overflow: hidden;
  }

  /* ── Header ── */
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
  .modal-title {
    font-size: 14px; font-weight: 700; color: var(--text);
  }
  .close-btn {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%;
    color: var(--text-tertiary);
    transition: background 0.1s, color 0.1s;
  }
  .close-btn:hover { background: var(--bg-hover); color: var(--text); }

  /* ── Body ── */
  .modal-body {
    flex: 1; overflow-y: auto;
    padding: 20px 24px 28px;
    display: flex; flex-direction: column; gap: 24px;
  }

  section { display: flex; flex-direction: column; gap: 8px; }

  h2 {
    font-size: 11px; font-weight: 700; letter-spacing: 0.06em;
    text-transform: uppercase; color: var(--rust-orange);
    margin: 0;
  }

  p {
    font-size: 13px; color: var(--text-secondary); line-height: 1.65;
    margin: 0;
  }

  strong { color: var(--text); font-weight: 600; }

  ul {
    margin: 0; padding-left: 18px;
    display: flex; flex-direction: column; gap: 4px;
  }
  li { font-size: 13px; color: var(--text-secondary); line-height: 1.55; }

  /* Keyboard shortcut table */
  table {
    border-collapse: collapse;
    font-size: 12px;
  }
  td {
    padding: 4px 12px 4px 0;
    color: var(--text-secondary);
    vertical-align: middle;
  }
  td:first-child { white-space: nowrap; }

  kbd {
    display: inline-block;
    font-family: var(--font-ui);
    font-size: 10.5px; font-weight: 600;
    color: var(--text);
    background: rgba(255,255,255,0.08);
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    padding: 2px 6px;
    line-height: 1.5;
    box-shadow: 0 1px 0 rgba(0,0,0,0.4);
  }

  code {
    font-family: var(--font-mono); font-size: 11.5px;
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
  }
  .code-block pre {
    margin: 0;
    font-family: var(--font-mono); font-size: 11.5px;
    color: var(--text); line-height: 1.7;
    white-space: pre;
  }

  /* Rust Book attribution section */
  .attribution-section {
    background: rgba(80, 140, 255, 0.05);
    border: 1px solid rgba(80, 140, 255, 0.18);
    border-radius: 6px;
    padding: 12px 14px;
    gap: 10px;
  }
  .attribution-section h2 { color: #7aadff; }
  .attr-table {
    border-collapse: collapse;
    font-size: 12px;
    width: 100%;
  }
  .attr-table td {
    padding: 3px 12px 3px 0;
    color: var(--text-secondary);
    vertical-align: middle;
  }
  .attr-table td:first-child {
    color: var(--text-tertiary);
    white-space: nowrap;
    width: 120px;
  }
  .attr-table a {
    color: #7aadff;
    text-decoration: none;
  }
  .attr-table a:hover { text-decoration: underline; }
  .attr-note {
    font-size: 11.5px !important;
    color: var(--text-tertiary) !important;
    font-style: italic;
  }

  /* Security warning section */
  .warn-section {
    background: rgba(255, 80, 60, 0.06);
    border: 1px solid rgba(255, 80, 60, 0.2);
    border-radius: 6px;
    padding: 12px 14px;
  }
  .warn-section h2 { color: var(--red); }
</style>
