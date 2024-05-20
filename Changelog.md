* 2024-XX-XX: v1.4 :
  * `crc` can now compute all known types, alg list updated.
  * Add `--recursive` option to `bgrep`, thanks @marius851000!.
  * `urlenc`:
    * Add `--exclude-chars`
    * Add `-u` to use RFC3986
    * Add `--custom` to specify custom list of chars to encode
* 2024-01-24: v1.3 : `slice` now supports non-seekable files. `tsdec` verbose mode. `bgrep` multiple args. Tests now cover real CLI invocations.
* 2023-09-26: v1.2.1: fix CLI flags parsing, add skipping of invalid files in findso
* 2023-08-13: v1.2: inflate/deflate applet ; base64 update: support custom alphabet ; global: check if given value is potentially a file and warn user
* 2023-01-08: v1.1: better usage info and error reporting
* 2022-08-20: v1.0: `findso` applet, clap update
* 2022-06-25: v0.9: `time` crate update, basic implementation of binary grep in `bgrep`
* 2022-05-01: v0.8: handle most errors gracefully, add more CRC algorithms (`crc` command), negative offset for `slice`
* 2020-09-09: v0.6: add buffer overflow pattern tools, timestamp decoding
* 2020-08-13: v0.5: code rewrite, merge `xorf` into `xor` and `unhex2` into `unhex`.  `slice` now supports relative `end`.
* 2020-08-06: v0.3 : URL-safe support for base64, cleaner argument parsing
* 2020-04-12: v0.2 : `xorf` command to xor files
* 2019-09-30: v0.1 : Initial release
