# Sidecar Migration

This document describes the migration from executing `amp-executor.ts` via `bun run` to using it as a compiled Tauri sidecar binary.

## Overview

The `amp-executor.ts` script is now compiled into a platform-specific binary and bundled with the Tauri app as a sidecar. This eliminates the runtime dependency on `bun` and provides better performance and reliability.

## Key Changes

### Build Process

- **Makefile**: Added `sidecar` target that compiles `amp-executor.ts` using `bun build --compile`
- **Platform Detection**: Automatically detects platform (macOS arm64/x86_64, Linux) and builds the appropriate binary
- **Output**: `src-tauri/binaries/amp-executor-{target}` (e.g., `amp-executor-aarch64-apple-darwin`)
- **Integration**: `make build` now automatically runs `make sidecar` before building the Tauri app

### Tauri Configuration

**tauri.conf.json**:

- Removed `resources` array (no longer bundling TypeScript source or node_modules)
- Added `externalBin: ["binaries/amp-executor"]` to register the sidecar
- Updated `beforeDevCommand` and `beforeBuildCommand` to run `make sidecar`

### Rust Implementation

**Type Changes**:

```rust
// Old: std::process::Child with pipes
static ref ACTIVE_CHILDREN: Mutex<HashMap<String, Arc<Mutex<std::process::Child>>>>

// New: CommandChild in Option for safe consumption
type ChildHandle = Arc<Mutex<Option<CommandChild>>>;
static ref ACTIVE_CHILDREN: Mutex<HashMap<String, ChildHandle>>
```

**Execution Pattern**:

```rust
// Old: spawn + pipes + wait
let child = Command::new("bun").args(&args).spawn()?;
let stdout = child.stdout.take()?;
let status = child.wait()?;

// New: spawn + event stream + terminated event
let (mut rx, child) = app.shell().sidecar("amp-executor")?.spawn()?;
while let Some(event) = rx.recv().await {
    match event {
        CommandEvent::Stdout(bytes) => { /* accumulate */ }
        CommandEvent::Stderr(bytes) => { /* parse session_id */ }
        CommandEvent::Terminated(payload) => { break; }
        _ => {}
    }
}
```

**Cancellation Pattern**:

```rust
// Old: Kill directly
child.lock().unwrap().kill();

// New: Take from Option and kill
if let Some(child) = handle.lock().unwrap().take() {
    child.kill();
}
```

### Files Modified

1. **src-tauri/src/commands/executor.rs**:
   - Refactored `execute_with_amp()` to use `CommandEvent` stream
   - Updated cancellation logic in `cancel_execution()` and `cancel_validation()`
   - Changed `ACTIVE_CHILDREN` type to store `ChildHandle`

2. **src-tauri/src/commands/analysis.rs**:
   - Refactored `execute_analysis_with_amp()` to use `CommandEvent` stream

3. **Makefile**:
   - Added platform detection logic
   - Added `sidecar` target
   - Updated `build` target to depend on `sidecar`

4. **src-tauri/tauri.conf.json**:
   - Replaced `resources` with `externalBin`
   - Updated build commands

## Benefits

1. **No Runtime Dependency**: Users don't need `bun` installed
2. **Better Performance**: Compiled binary vs interpreted TypeScript
3. **Smaller Bundle**: No need to bundle `node_modules` or TypeScript source
4. **Cleaner Architecture**: Proper sidecar pattern vs ad-hoc script execution
5. **Consistent Behavior**: Same binary across dev and production

## Building

```bash
# Build sidecar binary only
make sidecar

# Build full app (includes sidecar)
make build

# Development mode (sidecar built automatically)
make dev
```

## Platform Support

- **macOS**: arm64 (Apple Silicon), x86_64 (Intel)
- **Linux**: x86_64
- **Windows**: Not yet implemented (would need x86_64-pc-windows-msvc target)

## Migration Notes

- The `CommandChild` API requires async/await for event processing
- Stdout/stderr are no longer pipes but event streams
- Cancellation must use `Option::take()` since `kill()` consumes `self`
- All `execute_with_amp` callers must be async (already were)

## References

- [Tauri Sidecar Documentation](https://v2.tauri.app/develop/sidecar/)
- [tauri-plugin-shell Documentation](https://github.com/tauri-apps/tauri-plugin-shell)
- Oracle consultation in thread T-33818c60-3feb-4774-a404-bcd097f657be
