# rsbkb

A Rust reimplementation of some tools found in emonti's
[rbkb](https://github.com/emonti/rbkb), itself a Ruby reimplementation of
Matasano's BlackBag.

## Why reimplement it ?

* rbkb is unmaintained
* Ruby is slow
* I wanted to learn Rust


## How to use

### Build

```
$ cargo rustc --release
```
or get the binary from the [release page](https://github.com/trou/rsbkb/releases).

### Usage


* All tools take values as an argument on the command line or if not present, read stdin
* Tool name can be specified on the command line `rsbkb -t TOOL`
* Or can be called busybox-style: `ln -s rsbkb unhex ; unhex 4142`

```
for i in slice unhex hex d64 b64 urldec urlenc xor crc16 crc32 ; do ln -s rsbkb $i ; done
```

## Included tools

* `unhex`: decode hex data (either in the middle of arbitrary data, or strictly)
* `hex`: hex encode
* `d64`: base64 decode (use `-u` or `--URL` for URL-safe b64)
* `b64`: base64 encode (use `-u` or `--URL` for URL-safe b64)
* `urldec`: url decode
* `urlenc`: url encode
* `xor`: xor (use `-x` to specify the key, in hex, `-f` to specify a file)
* `crc16`: CRC-16
* `crc32`: CRC-32
* `slice`: slice of a file: :
 * `slice input_file 10` will print `input_file` from offset 10 on stdout.
 * `slice input_file 0x10 0x20` will do the same from 0x10 to 0x20 (excluded).
 * `slice input_file 0x10 +0xFF` will copy `0xFF` bytes starting at `0x10`.

### Getting help

```
$ rsbkb help
USAGE:
    rsbkb [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    b64       base64 encode
    crc16     compute CRC-16
    crc32     compute CRC-32
    d64       base64 decode
    help      Prints this message or the help of the given subcommand(s)
    hex       hex encode
    slice     slice
    unhex     Decode hex data
    urldec    URL decode
    urlenc    URL encode
    xor       xor value

$ rsbkb help slice
rsbkb-slice 
slice

USAGE:
    rsbkb slice <file> <start> [end]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <file>     file to slice
    <start>    start of slice
    <end>      end of slice: absolute or relative if prefixed with +

## Examples

```console
$ unhex 4141:4141
AA:AA
$ echo '4141:4141' | unhex
AA:AA
$ echo '41 41 41 32' | unhex
A A A 2
$ echo '41 41 41 32' | unhex
AAA2
$ echo '41 41 41 32' | unhex2 | xor -x 41 | hex
00000073
$ crc32 '41 41 41 32'
e60ce752
$ echo -n '41 41 41 32' | crc32
e60ce752
$ echo test | b64 | urlenc
dGVzdAo%3D%0A
```
