# rsbkb (Rust blackbag)

## What is it?

`rsbkb` has multiple tools which are designed to be called directly (through
symlinks). This allows various operations on data to be chained easily like
CyberChef but through pipes.

It also includes various practical tools like `entropy` or a timestamp decoder.

## Examples

Read 10 bytes from `/etc/passwd` starting at offset `0x2f`, then `xor` with
`0xF2`, encode it in URL-safe base64 and finally URL encode it:

```
$ slice /etc/passwd 0x2f +10 | xor -x f2 | b64 -u | urlenc
l5%2DdnMjdh4GA3Q%3D%3D
```

Various examples:

```
$ unhex 4141:4141
AA:AA
$ echo -n'4141:4141' | unhex
AA:AA
$ crc32 '41 41 41 32'
e60ce752
$ echo -n '41 41 41 32' | crc32
e60ce752
$ echo test | b64 | urlenc
dGVzdAo%3D
$ tsdec 146424672000234122
2065-01-01T00:00:00.0234122Z
$ tsdec 0
1970-01-01T00:00:00Z
$ rsbkb bofpatt 60
Aa0Aa1Aa2Aa3Aa4Aa5Aa6Aa7Aa8Aa9Ab0Ab1Ab2Ab3Ab4Ab5Ab6Ab7Ab8Ab9
$ rsbkb bofpattoff -b 0x41623841
Decoded pattern: Ab8A (big endian: true)
Offset: 54 (mod 20280) / 0x36
$ echo -n tototutu | rsbkb entropy
0.188
$ bgrep -x 454c460201 /bin/ls
0x1
$ bgrep "\x45\x4c..\x01" /bin/ls
0x1
$ findso -p /lib/x86_64-linux-gnu/ -r memcpy /bin/ls
/lib/x86_64-linux-gnu/libc.so.6
$ findso -l /etc/ld.so.conf -a memcpy
/lib/i386-linux-gnu/libc.so.6
/usr/lib/i386-linux-gnu/libc.so.6
/lib/x86_64-linux-gnu/libasan.so.6
[...]
```

## How to use

### Build it / Get it

```
$ cargo rustc --release
```

or:

* get the binary from the [release page](https://github.com/trou/rsbkb/releases).
* get the latest artifact from the [Actions](https://github.com/trou/rsbkb/actions) page.

### Usage


* All tools take values as an argument on the command line or if not present, read stdin
* Tool name can be specified on the command line `rsbkb TOOL`
* Or can be called busybox-style: `ln -s rsbkb unhex ; unhex 4142`

```
for i in $(rsbkb list) ; do ln -s rsbkb $i ; done
```

## Included tools

* `hex`: hex encode
* `unhex`: decode hex data (either in the middle of arbitrary data, or strictly)
* `b64`: base64 encode (use `-u` or `--URL` for URL-safe b64)
* `d64`: base64 decode (use `-u` or `--URL` for URL-safe b64)
* `urlenc`: url encode
* `urldec`: url decode
* `xor`: xor (use `-x` to specify the key, in hex, `-f` to specify a file)
* `crc`: all CRC algorithms implemented in the [Crc](https://docs.rs/crc/2.1.0/crc/) crate
* `crc16`: CRC-16
* `crc32`: CRC-32
* `bofpatt` / `boffpattoff`: buffer overflow pattern generator / offset calculator
* `tsdec`: decode various timestamps (Epoch with different resolutions, Windows FILETIME)
* `slice`: slice of a file: :
 * `slice input_file 10` will print `input_file` from offset 10 on stdout.
 * `slice input_file 0x10 0x20` will do the same from 0x10 to 0x20 (excluded).
 * `slice input_file 0x10 +0xFF` will copy `0xFF` bytes starting at `0x10`.
 * `slice input_file -0x10` will the last 0x10 bytes from `input_file`
* `entropy`: entropy of a file
* `bgrep`: simple binary grep
* `findso`: find which ELF shared library (.so) exports a given name/function
* `inflate` and `deflate`: raw inflate/deflate compression, fault tolerant and with optional Zlib header support
* `base`: easy radix conversion of big integers

### Getting help

```console
$ rsbkb help
rsbkb 

USAGE:
    rsbkb [APPLET]

OPTIONS:
    -h, --help    Print help information

APPLETS:
    b64           base64 encode
    bgrep         binary grep
    bofpatt       buffer overflow pattern generator
    bofpattoff    buffer overflow pattern offset finder
    crc           flexible CRC computation
    crc16         compute CRC-16
    crc32         compute CRC-32
    d64           base64 decode
    deflate       (raw) deflate compression
    entropy       compute file entropy
    findso        find which .so implements a given function
    help          Print this message or the help of the given subcommand(s)
    hex           hex encode
    inflate       (raw) inflate decompression
    list          list applets
    slice         slice
    tsdec         timestamp decoder
    unhex         hex decode
    urldec        URL decode
    urlenc        URL encode
    xor           xor value

$ rsbkb help slice
slice

USAGE:
    rsbkb slice <file> <start> [end]

ARGS:
    <file>     file to slice
    <start>    start of slice, relative to end of file if negative
    <end>      end of slice: absolute or relative if prefixed with +
```

## Credits and heritage

This is a Rust reimplementation of some tools found in emonti's
[rbkb](https://github.com/emonti/rbkb), itself a Ruby reimplementation of
Matasano's BlackBag.
