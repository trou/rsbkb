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

* `unhex`: decode all hex pairs, leave the rest untouched
* `unhex2`: decode hex, ignores spaces. in strict mode, the input must be only hex chars
* `hex`
* `d64`
* `b64`
* `urldec`
* `urlenc`
* `xor`
* `crc16`
* `crc32`
* `slice`

## Examples

```console
$ echo '4141:4141' | unhex 
AA:AA
$ echo '41 41 41 32' | unhex 
A A A 2
$ echo '41 41 41 32' | unhex2
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
