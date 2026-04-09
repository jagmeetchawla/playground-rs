# Code Review — Rustic Playground v0.3.4

> Fresh senior staff engineer review, 2026-04-09.
> Covers all backend Rust, frontend Svelte, config, and scripts.

---

## CRITICAL ISSUES

### 1. Unsafe Unwrap on Child Process Pipes
**File:** `playground_commands.rs:484-485, 657`

Using `.unwrap()` on `child.stdout.take()` and `child.stderr.take()` can panic at runtime if pipes are unexpectedly None. The code configures `Stdio::piped()` at spawn, but edge cases (e.g., process termination race) could cause these to become None.

**Impact:** App crash in production during concurrent process management.

**Fix:**
```rust
let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
```

### 2. Unsafe Path Construction Without Canonicalization
**File:** `playground_commands.rs:457`

`binary_path.to_str().unwrap()` is called on a computed path without validation. While `playground_path()` validates earlier, the binary path for `CompileThenRun` is constructed dynamically. Intermediate steps don't canonicalize before execution.

**Impact:** Potential path injection via malicious compilation artifacts (low risk in single-user context, but architecturally unsound).

**Fix:**
```rust
let binary_str = binary_path
    .canonicalize()
    .and_then(|p| p.to_str().map(|s| s.to_string()))
    .ok_or_else(|| "Invalid binary path".to_string())?;
```

### 3. Command Injection via Shell Interpreter
**File:** `lib.rs:601-604`

Rust installer is invoked via `sh -c` with a hardcoded curl command piped to shell. While the URL is hardcoded (not user-controlled), the pattern is unsafe. If any part of the execution path is user-influenced (e.g., via environment variables like `HOME`), this becomes a vector.

**Impact:** Medium risk; mitigated by hardcoded URL, but demonstrates unsafe pattern.

**Fix:** Use direct process invocation instead of shell piping, or download to temp file first, then execute.

### 4. Mutex Lock Held Across Await Boundary
**File:** `playground_commands.rs:482`

`app.state::<RunningProcess>().0.lock().unwrap()` is a blocking sync Mutex held while awaiting on line 484+. This can cause async runtime stalls.

**Impact:** Potential deadlock or starvation under high concurrency.

**Fix:** Move sync lock outside the async await path or convert to async mutex.

### 5. Xcode CLT Dialog on Vanilla Macs
**File:** `cargo_commands.rs:200-204`, `lib.rs` (WKWebView)

On vanilla Macs, macOS shows an "Install Command Line Developer Tools" dialog at app launch. This is triggered by Apple's WKWebView framework (used by Tauri) probing for developer tools during WebView initialization — NOT by our code. Confirmed via log stream instrumentation 2026-04-09.

**Impact:** Confusing UX on first launch on vanilla Macs. Documented in README and HelpModal.

---

## HIGH SEVERITY ISSUES

### 6. Race Condition in `check_playground` Process Cancellation
**File:** `playground_commands.rs:618-625`

The code kills a previous `cargo check` process before starting a new one, but there's a race: if the process exits naturally between the `take()` call and the `kill -TERM` command, sending SIGTERM to a stale PID could kill an unrelated process with the same PID (PID reuse).

**Impact:** Could accidentally terminate an unrelated user process.

**Fix:** Use a generation counter or process group handle instead of raw PID. Double-check process is still alive before killing.

### 7. No Timeout on Live `cargo check` Process
**File:** `playground_commands.rs:635-656`

`cargo check` runs with no timeout. On a slow machine or with a large project, the process could hang indefinitely, blocking the output channel and consuming memory.

**Impact:** Memory exhaustion, UI freeze.

**Fix:** Add `tokio::time::timeout(Duration::from_secs(30), child.wait())` wrapper.

### 8. Unsafe JSON Parsing in `check_playground`
**File:** `playground_commands.rs:662-694`

Compiler message JSON is parsed with multiple `.get()` chains. A malformed or unexpected JSON structure will silently skip diagnostics instead of logging the issue.

**Impact:** Silent data loss if cargo changes its JSON format in newer Rust versions.

**Fix:** Log parsing errors instead of silently dropping them.

### 9. Hardcoded Default C Flags
**File:** `rustic_manifest.rs:68-72`

Default C/C++ build flags may include dependencies that assume system-wide libraries. The Clang project type is supposed to be minimal but may have hidden dependencies.

**Impact:** Silent build failures for C/C++ playgrounds on minimal systems.

**Fix:** Start with empty default flags; let users add what they need.

### 10. Missing Source Validation in `import_content_file`
**File:** `content_commands.rs:75-117`

The source path `src_path` is taken as a String without validation. While `safe_content_path()` validates the destination, the source path is never checked for traversal or symlinks. A user could pass `/etc/passwd` and import it into the project.

**Impact:** Information disclosure; user can read arbitrary files on the system.

**Fix:** Validate source path, reject symlinks, optionally restrict to safe directories.

### 11. Unbounded Output Buffer in `stream_pipe`
**File:** `playground_commands.rs:516-561`

The `leftover` string accumulates bytes indefinitely. If a program outputs a single huge line (e.g., 1GB of data with no newline), memory usage will grow without bound.

**Impact:** Denial of service; malicious code can crash the app.

**Fix:** Add a `MAX_LINE_LEN` (e.g., 1 MB) limit; truncate or emit partial lines when exceeded.

---

## MEDIUM SEVERITY ISSUES

### 12. Path Traversal in `copy_playground_to_project`
**File:** `playground_commands.rs:725-752`

The `target_project` parameter is only checked for existence. A malicious frontend could pass `../../` to write to parent directories. The validation happens in `target_lang.playground_path()`, but the direct check is minimal.

**Impact:** Low in single-user context, but violates defense-in-depth.

**Fix:** Validate target project name through `validate_name()`.

### 13. No Concurrent Edit Handling
**Files:** `playground_commands.rs`, `App.svelte`

Multiple frontend instances can edit the same playground file without conflict resolution. The last write wins, silently losing intermediate edits.

**Impact:** Data loss if user has multiple tabs/windows open (rare for single-edition, relevant for side-by-side edition testing).

**Fix:** Implement optimistic locking: add a version field, validate on save, return conflict error if versions don't match.

### 14. Manifest Auto-Generation Race Condition
**File:** `lib.rs:221-228`

`ensure_project()` auto-generates a manifest if missing, but if two concurrent calls race, one might generate incorrectly while another is checking.

**Impact:** Project type confusion if app is launched in unusual conditions.

**Fix:** Use a lock file or atomic write.

### 15. Missing Cleanup on Failed Export
**File:** `export.rs:437-523, 529-609`

File I/O errors during export leave partial files behind. No cleanup occurs on failure.

**Impact:** Corrupted exports; disk littered with partial files.

**Fix:** Use a temporary directory and atomic rename on success; clean up on failure.

### 16. Frontend Validation Doesn't Know Active Language
**File:** `App.svelte` (new_playground handler)

Frontend validates names before sending to backend, but the backend's validation is language-specific. The frontend doesn't always know the active language when the user hits Cmd+N.

**Impact:** User gets validation errors after typing if language context is mismatched.

**Fix:** Include `projectType` in the frontend validation or query from backend first.

---

## MEDIUM-LOW SEVERITY ISSUES

### 17. Zombie Process Potential in SIGKILL Cleanup
**File:** `playground_commands.rs:563-581`

After SIGTERM, if the process doesn't die in 300ms, SIGKILL is sent. But if the process is already dead but hasn't been waited on, the PID becomes a zombie.

**Impact:** Zombie processes accumulate if user runs many short-lived programs.

**Fix:** Call `.wait()` after SIGKILL to reap the child.

### 18. Memory Leak Possibility in `send_stdin`
**File:** `playground_commands.rs:756-775`

If `send_stdin` is called while the process is being killed, there's a race where we might write to a closed pipe (SIGPIPE). Error handling catches it but isn't clean.

**Impact:** Minor; SIGPIPE is caught but process state is inconsistent.

**Fix:** Check if child is still alive before writing to stdin.

### 19. No Limits on Dependency Version Specs
**File:** `cargo_commands.rs:61-107`

`add_dependency()` accepts a `version` parameter with loose validation. Invalid versions won't be rejected until `cargo add` runs and fails.

**Impact:** Poor UX; confusing error messages.

**Fix:** Validate version strings more strictly before invoking cargo.

### 20. Insufficient Bounds Checking on Diagnostic Line Numbers
**File:** `playground_commands.rs:686-689`

Diagnostic line numbers default to 1 when missing from cargo's JSON output. This masks bugs in cargo's output parsing.

**Impact:** Diagnostics shown on wrong lines, confusing users.

**Fix:** Log and reject malformed diagnostics.

---

## LOW SEVERITY ISSUES

### 21. Hardcoded Rust Edition
**File:** `lib.rs:167-171`

New projects are created with `edition = "2024"`. This should be dynamically determined or configurable for compatibility.

### 22. Dead Parameters in `build_menu`
**File:** `menu.rs:23, 26`

Parameters `_playground_count` and `_project_type` are prefixed with `_` — unused dead code. Should be removed.

### 23. Inconsistent Error Message Formatting
**Files:** Various (cargo_commands.rs, playground_commands.rs, export.rs)

Error messages mix lowercase and Title Case inconsistently.

### 24. No Config Caching
**File:** `lib.rs:135-149`

`load_config()` reads config.json from disk on every call without caching. Called multiple times during startup.

### 25. No Symlink Check on Project Directories
**File:** `playground_commands.rs:14-26`

`list_projects()` enumerates projects without checking for symlinks. A user could symlink arbitrary directories into the projects folder.

---

## ARCHITECTURAL OBSERVATIONS

### Good Patterns
1. **Language dispatch via enum** — `Lang` enum in `languages/mod.rs` ensures exhaustive match coverage; adding a new language causes compilation errors at every dispatch point.
2. **Manifest versioning** — `rustic.toml` allows future evolution without breaking existing projects.
3. **Snapshot system** — `.saved/` snapshots for revert are clean and don't interfere with VCS.
4. **Edition isolation** — separate bundle IDs and storage paths per edition; tested side-by-side successfully.
5. **CLT-shim guard** — xcrun/clang/swiftc calls correctly gated behind `xcode_clt_installed`.

### Concerns
1. **Global app state with Mutex locks** — Using `Mutex<Option<u32>>` for process PIDs is fine for small state but doesn't scale. Consider a single `AppState` struct.
2. **Frontend/backend coupling** — The frontend knows too many backend details (manifest structure, build flags, etc.). A versioned API contract would be more maintainable.
3. **No event sourcing** — Changes to playgrounds and projects aren't logged. If the app crashes mid-save, there's no recovery beyond the `.saved/` snapshot.

---

## SECURITY SUMMARY

| Category | Risk | Notes |
|----------|------|-------|
| Command Injection | Low | Hardcoded URLs, but shell pattern is unsafe |
| Path Traversal | Medium | `import_content_file` lacks source validation |
| Process Safety | Medium | Race conditions in PID handling, PID reuse |
| Memory Safety | High | Unbounded output buffer in `stream_pipe` |
| Concurrent Edits | Medium | No conflict resolution for multi-tab editing |
| **Overall** | **Medium** | **Sandboxing disabled; relies on macOS permissions** |

---

## TOP 5 FIXES (by impact)

1. **Fix unbounded output buffer** (issue #11) — Prevents OOM from malicious/runaway code
2. **Fix source path validation in import_content_file** (issue #10) — Closes information disclosure
3. **Fix unwrap on child pipes** (issue #1) — Prevents app crash during process management
4. **Add timeout to cargo check** (issue #7) — Prevents UI freeze on slow machines
5. **Fix race in process cancellation** (issue #6) — Prevents accidentally killing wrong process
