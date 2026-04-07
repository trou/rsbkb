---
name: rsbkb (Rust BlackBag)

description: Use rsbkb for binary data manipulation, CLI tools: hex unhex urlenc urldec crc16 crc32 crc b64 d64 bofpattoff bofpatt xor entropy slice bgrep findso tsdec tsenc deflate inflate base escape unescape
---

## Overview
rsbkb is a versatile collection of command-line tools (applets) designed for rapid data transformation and analysis. It functions similarly to `busybox`, where a single binary provides multiple utilities that can be chained together via pipes to perform complex operations, effectively serving as a high-performance CLI alternative to CyberChef.

## Key Capabilities

### Encoding & Decoding
- **Hex**: `hex` (encode), `unhex` (flexible decoding of hex strings and mixed data).
- **Base64**: `b64` (encode), `d64` (decode), with URL-safe support via `-u`.
- **URL**: `urlenc` (encode), `urldec` (decode) with advanced escaping options.

### Binary Analysis & Hacking
- **Search**: `bgrep` for binary pattern matching using hex or regex.
- **Entropy**: `entropy` to calculate Shannon entropy for identifying packed or encrypted data.
- **Exploitation**: `bofpatt` and `bofpattoff` for cyclic pattern generation and offset calculation.
- **Timestamps**: `tsdec` for decoding Unix epochs (various precisions) and Windows FILETIME.
- **Library Analysis**: `findso` to locate which ELF shared object exports a specific symbol.

### Data Transformation
- **Slicing**: `slice` extracts byte ranges using absolute, relative, or end-relative offsets.
- **Logic**: `xor` applies XOR operations using hex keys or key files.
- **Checksums**: `crc`, `crc16`, and `crc32` supporting numerous standard algorithms.
- **Compression**: `inflate` and `deflate` for raw or Zlib-wrapped streams.
- **String Handling**: `escape` and `unescape` for various shell and programming string formats.
- **Base Conversion**: `base` for arbitrary radix conversion of large integers.

## Input Model

Applets fall into three categories based on how they receive input:

### Stdin / value applets (do NOT take a file path argument)
These read binary data from **stdin** (piped) or as an optional inline `[value]` argument.
**Do not pass a filename to these — pipe the data in or provide the value directly.**

| Applet(s) | Notes |
|---|---|
| `entropy` | pipe data in: `cat file \| entropy` |
| `hex`, `unhex` | encode/decode hex |
| `b64`, `d64` | encode/decode base64 |
| `urlenc`, `urldec` | encode/decode URL |
| `escape`, `unescape` | string escaping |
| `inflate`, `deflate` | compression |
| `crc16`, `crc32` | checksums (data via stdin) |
| `crc` | requires algorithm type arg; data via stdin/value |
| `xor` | requires `-x KEY` or `-f keyfile`; data via stdin/value |
| `base` | integer base conversion; value via stdin/arg |
| `tsdec`, `tsenc` | timestamp value via stdin/arg |

### File applets (require a file path argument)
These take an explicit **file path** as a positional argument — they do not read the target data from stdin.

| Applet | Signature |
|---|---|
| `slice` | `slice <file> <start> [end]` — use `-` for stdin |
| `bgrep` | `bgrep [opts] <pattern> <path>...` |
| `findso` | `findso [opts] <function> [files]...` |

### Value-argument applets (take a non-file, non-binary value)
These take a specific **numeric or string value** as a positional argument, not data to process.

| Applet | Argument |
|---|---|
| `bofpatt` | `<length>` — numeric pattern length |
| `bofpattoff` | `<extract>` — pattern string or `0xAABBCCDD` register value |

## Usage Guidelines
- **Piping**: Stdin applets chain naturally: 
  `slice file 0x10 +64 | xor -x deadbeef | b64 -u` 
  `cat firmware.bin | entropy`
- **Invocation**: Call via `rsbkb <applet> [args]` or via symlinks created by `rsbkb symlink`.
- **Help**: Detailed documentation for any applet is available via `rsbkb help <applet>`.

## When to Apply
- **Reverse Engineering**: Decoding obfuscated strings, extracting payloads, or identifying compression.
- **Exploit Development**: Calculating buffer offsets and formatting strings for shell use.
- **Digital Forensics**: Analyzing binary blobs, searching for signatures, and normalizing timestamps.
- **CTFs**: Solving data manipulation challenges quickly through the command line.
