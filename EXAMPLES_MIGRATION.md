# Examples Migration Summary

## Fixed Examples

### 1. `examples/pipe_selection.rs`

**Changes Made:**

- Updated Section 3: Changed from `PipeConfig::PipeNumber(pipe_num)` to `PipeConfig::CustomPath(pipe_path)`
- Updated Section 5: Changed timeout example to use `CustomPath` instead of `PipeNumber`
- Now shows both pipe number and full path in output for clarity

**New Behavior:**

```rust
// Old way (removed):
PipeConfig::PipeNumber(0)

// New way:
let pipes = IpcConnection::discover_pipes();
PipeConfig::CustomPath(pipes[0].path.clone())
```

### 2. `examples/flatpak_discord.rs`

**Changes Made:**

- Updated Section 3: Loop now uses `CustomPath` with discovered pipe paths
- Enhanced output to show both pipe number and path for better debugging

**New Behavior:**

```rust
// Old way (removed):
Some(PipeConfig::PipeNumber(pipe.pipe_number))

// New way:
Some(PipeConfig::CustomPath(pipe.path.clone()))
```

## Why This Is Better

1. **More Explicit**: Users see exactly which socket/pipe path they're connecting to
2. **Better for Flatpak**: Full paths make it obvious when connecting to Flatpak Discord
3. **Easier Debugging**: Path information helps diagnose connection issues
4. **Consistent API**: All connection methods now work uniformly with paths

## Running the Examples

Both examples still work exactly as before, but now use the simplified API:

```bash
# Pipe selection example
cargo run --example pipe_selection

# Flatpak Discord example
cargo run --example flatpak_discord
```

## Example Output Changes

**Before:**

```
3. Connecting to specific pipe number 0...
```

**After:**

```
3. Connecting to specific pipe 0 at /run/user/1000/discord-ipc-0...
```

The output now shows both the logical pipe number (for reference) and the actual path being used (for clarity).
