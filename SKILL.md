## Metadata
name: rsbkb (Rust BlackBag)
description: Use rsbkb for binary data manipulation, encoding/decoding, and security-oriented data analysis via a CLI multi-tool.

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

## Usage Guidelines
- **Piping**: Most applets default to `stdin` if arguments are omitted, enabling chains like:  
  `slice file 0x10 +64 | xor -x deadbeef | b64 -u`
- **Invocation**: Call via `rsbkb <applet> [args]` or via symlinks created by `rsbkb symlink`.
- **Help**: Detailed documentation for any applet is available via `rsbkb help <applet>`.

## When to Apply
- **Reverse Engineering**: Decoding obfuscated strings, extracting payloads, or identifying compression.
- **Exploit Development**: Calculating buffer offsets and formatting strings for shell use.
- **Digital Forensics**: Analyzing binary blobs, searching for signatures, and normalizing timestamps.
- **CTFs**: Solving data manipulation challenges quickly through the command line.
