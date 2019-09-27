# rsbkb

A Rust reimplementation of some tools found in emonti's
[rbkb](https://github.com/emonti/rbkb), itself a Ruby reimplementation of
Matasano's BlackBag.

## Why reimplement it ?

* rbkb is unmaintained
* Ruby is slow
* I wanted to learn Rust

## How to use

* All tools take values as an argument on the command line or if not present, read stdin
* Tool name can be specified on the command line `rsbkb -t TOOL`
* Or can be called busybox-style: `ln -s rsbkb unhex ; unhex 4142`

```
for i in slice unhex hex d64 b64 urldec urlenc xor crc16 crc32 unhex2 ; do ln -s rsbkb $i ; done
```

## Included tools

* `unhex`
* `unhex2`: decode all hex pairs, leave the rest untouched
* `hex`
* `d64`
* `b64`
* `urldec`
* `urlenc`
* `xor`
* `crc16`
* `crc32`
* `slice`
