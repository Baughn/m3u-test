# m3u-emu Rust Port Design

**Date:** 2026-01-01
**Status:** Approved

## Goals

- **Distribution:** Single binary, no dependencies on bash/coreutils
- **Maintainability:** Better code structure than the original bash script
- **Cross-platform:** Linux, macOS, and Windows support
- **UX improvements:** Progress output, colored errors, better help text

## Project Structure

```
m3u-emu/
├── Cargo.toml
└── src/
    ├── main.rs      # CLI parsing, orchestration
    ├── cli.rs       # Clap argument definitions
    ├── scanner.rs   # Directory traversal, file discovery
    ├── parser.rs    # Filename parsing (disc/floppy detection)
    ├── m3u.rs       # M3U file creation and validation
    └── types.rs     # Shared types (MediaFile, GameSet, etc.)
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` (derive) | CLI argument parsing |
| `walkdir` | Recursive directory traversal |
| `regex` | Pattern matching for disc identifiers |
| `infer` | File type detection (replaces `file` command) |
| `indicatif` | Progress bars and spinners |
| `console` | Colored output and terminal handling |
| `anyhow` | Ergonomic error handling |

## CLI Interface

```
m3u-emu [OPTIONS] <TARGET> [DESTINATION]

Arguments:
  <TARGET>       Directory to search for ROM media files
  [DESTINATION]  Where to write m3u files (default: alongside ROMs)

Options:
  -r, --relative   Use relative paths in m3u files (default: absolute)
  -c, --children   Create subdirectories in DESTINATION mirroring TARGET's
                   top-level folders (requires DESTINATION)
  -f, --force      Use disc-style grouping for floppy formats too
  -q, --quiet      Suppress progress output, only show errors
  -v, --verbose    Show detailed information about each file processed
  -h, --help       Print help
  -V, --version    Print version
```

### Default Output

```
Scanning /roms/psx...
  Found 156 media files in 23 directories
Creating m3u files...
  [████████████████████████] 23/23 directories
Done: Created 31 m3u files
```

### Error Output (colored, to stderr)

```
Warning: /roms/psx/Myst III/extras.m3u may not be a text file, skipping directory
Error: Could not access /roms/restricted - permission denied
```

## Core Types

```rust
struct MediaFile {
    path: PathBuf,          // Full path to file
    filename: String,       // Just the filename
    base_name: String,      // Name with disc/side markers removed
    disc_number: f32,       // e.g., 2.0, or 2.1 for "Disc 2 Side 1"
    is_floppy: bool,        // Detected from extension
}
```

## Filename Parsing

1. Detect media type from extension (floppy vs disc)
2. Extract disc identifier from parentheses: `(Disc 2)`, `(CD Two)`, `(Floppy B)`
3. Convert words/letters to numbers: "Two" → 2, "B" → 2
4. Extract side number if present: `(Side A)` → 0.1 suffix
5. Strip these markers to get `base_name` for grouping

### Extension Groups

**Floppy formats:**
`.ipf`, `.adf`, `.adz`, `.dms`, `.dim`, `.d64`, `.d71`, `.d81`, `.d88`, `.dsk`, `.ima`, `.fdi`, `.qd`, `.fds`, `.tap`, `.tzx`, `.cas`

**Disc index formats (priority):**
`.cue`, `.toc`, `.ccd`, `.gdi`

**Disc image formats:**
`.mds`, `.cdi`, `.img`, `.iso`, `.chd`, `.rvz`

## Grouping Logic

### Floppy Mode (default for floppy extensions)

All files in a directory go into one m3u. Floppy dumps often have inconsistent naming (boot disks, save disks, scenario disks, etc.), so grouping by name doesn't work reliably.

### Disc Mode (default for CD/DVD, or with `--force`)

Files are grouped by `base_name`. This allows multiple games to share a directory.

### Sorting

- Within each game set, files are sorted by disc number (version sort semantics)
- Side numbers act as sub-sort: Disc 1 Side A < Disc 1 Side B < Disc 2

## Directory Modes

### Normal Mode (no `--children`)

- Recursively scans TARGET and all subdirectories
- Creates m3u files in the same directory as the ROM files found
- If DESTINATION is provided, all m3u files go there (flat)

### Children Mode (`--children`)

- Looks at top-level directories in TARGET
- Creates matching directories in DESTINATION
- Scans each child recursively, placing all its m3u files in the corresponding output folder
- Empty output directories (no games found) are removed
- TARGET and DESTINATION themselves are not scanned for games

## Safety Checks

Before writing to any directory, existing `.m3u` files are checked using the `infer` crate. If any appear to be binary files, that directory is skipped with a warning rather than risk deleting something important.

## Error Handling

- **Recoverable errors** (permission denied on one directory, invalid file): Log warning, continue processing
- **Fatal errors** (target doesn't exist, can't create destination): Exit with clear error message

### Exit Codes

- `0` - Success (even if some directories were skipped with warnings)
- `1` - Fatal error (bad arguments, target doesn't exist, etc.)

## Testing Strategy

- Unit tests for parsing logic (filename → base_name + disc_number)
- Unit tests for word-to-number conversion ("Twenty-Three" → 23, "B" → 2)
- Integration tests with temporary directories containing mock ROM files
- Validation against original bash script output on sample directories
