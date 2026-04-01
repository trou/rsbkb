use assert_cmd::Command;
use predicates::str::contains;
use std::fs::File;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

// BaseIntApplet CLI tests

#[test]
fn test_base_cli_no_radix() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["base", "10"])
        .assert()
        .stdout("0xa")
        .success();
}

#[test]
fn test_base_cli_arg() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["base", "0x10"])
        .assert()
        .stdout("16")
        .success();
}

#[test]
fn test_base_cli_arg_from_to() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["base", "-f", "2", "-t", "16", "10000"])
        .assert()
        .stdout("10")
        .success();
}

#[test]
fn test_base_cli_stdin() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["base"])
        .write_stdin("0xA\n")
        .assert()
        .stdout("10")
        .success();
}

#[test]
fn test_base_cli_arg_to() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["base", "-t", "32", "0o7675"])
        .assert()
        .stdout("3tt")
        .success();
}

// Hex/UnHex CLI tests

#[test]
fn test_hex_cli_arg() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["hex", "aAé!"])
        .assert()
        .stdout("6141c3a921")
        .success();
}

#[test]
fn test_hex_cli_stdin() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["hex"])
        .write_stdin("aAé!\n")
        .assert()
        .stdout("6141c3a9210a")
        .success();
}

#[test]
fn test_unhex_cli_arg() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["unhex", "6141210a00ff"])
        .assert()
        .stdout(&b"aA!\n\x00\xff"[..])
        .success();
}

#[test]
fn test_unhex_cli_stdin() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["unhex"])
        .write_stdin("41ff\n00FF")
        .assert()
        .stdout(&[0x41, 0xFF, 0x0A, 0x00, 0xFF][..])
        .success();
}

#[test]
fn test_unhex_cli_stdin_hexonly() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["unhex", "-o"])
        .write_stdin("41ff\n00FF")
        .assert()
        .stdout(&b"A\xFF\n00FF"[..])
        .success();
}

#[test]
fn test_unhex_cli_stdin_strict() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["unhex", "-s"])
        .write_stdin("41l")
        .assert()
        .stdout(&b""[..])
        .stderr(contains("Odd number of digits"))
        .failure();
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["unhex", "-s"])
        .write_stdin("41ll")
        .assert()
        .stdout(&b""[..])
        .stderr(contains("Invalid character"))
        .failure();
}

// UrlEnc/UrlDec CLI tests

#[test]
fn test_urlenc_cli_arg() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["urlenc", "aAé!,"])
        .assert()
        .stdout("aA%c3%a9%21%2c")
        .success();
}

#[test]
fn test_urlenc_cli_arg_exclude() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["urlenc", "-e", "!,", "aAé!,"])
        .assert()
        .stdout("aA%c3%a9!,")
        .success();
}

#[test]
fn test_urlenc_cli_arg_custom() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["urlenc", "-e", "!,", "-c", "aA,", "aAé!,"])
        .assert()
        .stdout("%61%41é!,")
        .success();
}

#[test]
fn test_urlenc_stdin() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["urlenc"])
        .write_stdin("aAé!,")
        .assert()
        .stdout("aA%c3%a9%21%2c")
        .success();
}

// SliceApplet CLI tests

#[test]
fn test_slice_cli_file() {
    let mut data: [u8; 10] = [0; 10];
    for i in 0..10 {
        data[i] = i as u8;
    }

    let mut tmpfile = NamedTempFile::new().unwrap();
    tmpfile.write_all(&data).unwrap();

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["slice", tmpfile.path().to_str().unwrap(), "2", "+0x3"])
        .assert()
        .stdout(&b"\x02\x03\x04"[..])
        .success();

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["slice", tmpfile.path().to_str().unwrap(), "2"])
        .assert()
        .stdout(&b"\x02\x03\x04\x05\x06\x07\x08\x09"[..])
        .success();

    // Should fail because "start" is before beginning of file
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["slice", tmpfile.path().to_str().unwrap(), "-200"])
        .assert()
        .failure();

    // Should fail because "end" is before "start"
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["slice", tmpfile.path().to_str().unwrap(), "0", "-300"])
        .assert()
        .failure();

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["slice", "--", tmpfile.path().to_str().unwrap(), "-2"])
        .assert()
        .stdout(&b"\x08\x09"[..])
        .success();

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args([
            "slice",
            "--",
            tmpfile.path().to_str().unwrap(),
            "-0x2",
            "+1",
        ])
        .assert()
        .stdout(&b"\x08"[..])
        .success();
}

#[test]
fn test_slice_cli_stdin() {
    let mut data: [u8; 10] = [0; 10];
    for i in 0..10 {
        data[i] = i as u8;
    }

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["slice", "-", "2", "+3"])
        .write_stdin(&data[..])
        .assert()
        .stdout(&b"\x02\x03\x04"[..])
        .success();

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["slice", "-", "2"])
        .write_stdin(&data[..])
        .assert()
        .stdout(&b"\x02\x03\x04\x05\x06\x07\x08\x09"[..])
        .success();

    // Should fail because stdin is not seekable
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["slice", "-", "-2"])
        .write_stdin(&data[..])
        .assert()
        .stdout("")
        .failure();

    // Should fail because stdin is not seekable
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["slice", "-", "0", "-10"])
        .write_stdin(&data[..])
        .assert()
        .stdout("")
        .failure();

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["slice", "-", "0", "0"])
        .write_stdin(&data[..])
        .assert()
        .stdout(&b""[..])
        .success();
}

// Escape/UnEscape CLI tests

#[test]
fn test_base_escape_arg_auto() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["escape", r"te'st"])
        .assert()
        .stdout(r#""te\'st""#)
        .success();
}

#[test]
fn test_base_escape_stdin_auto() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["escape"])
        // by default, trim input so '\n' will be removed
        .write_stdin("'te'st'\n")
        .assert()
        .stdout(r"'te\'st'")
        .success();
}

#[test]
fn test_base_escape_stdin_no_detect() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["escape", "-d"])
        // no detect mode will not try to determine enclosing quote type,
        // just escape them
        .write_stdin(r"'test'")
        .assert()
        .stdout(r#""\'test\'""#)
        .success();
}

#[test]
fn test_base_escape_stdin_auto_multiline() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["escape", "-m"])
        // multiline mode will not trim '\n', escaping them instead
        .write_stdin("te'st\nte\"st\n")
        .assert()
        .stdout(r#""te\'st\nte\"st\n""#)
        .success();
}

#[test]
fn test_base_escape_stdin_bash_single() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["escape", "-t", "bash-single"])
        .write_stdin("te'st")
        .assert()
        .stdout(r#"'te'"'"'st'"#)
        .success();
}

#[test]
fn test_base_escape_stdin_bash() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["escape", "-t", "bash"])
        .write_stdin(r#""!t"e`s$t""#)
        .assert()
        .stdout(r#""\!t\"e\`s\$t""#)
        .success();
}

#[test]
fn test_base_escape_stdin_posix_shell() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["escape", "-t", "shell"])
        .write_stdin(r#""!t"e`s$t""#)
        .assert()
        .stdout(r#""!t\"e\`s\$t""#)
        .success();
}

#[test]
fn test_base_escape_stdin_single() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["escape", "-t", "single"])
        .write_stdin(r#"sin'gle"#)
        .assert()
        .stdout(r#"'sin\'gle'"#)
        .success();
}

#[test]
fn test_base_escape_stdin_single_noquote() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["escape", "-t", "single", "-n"])
        .write_stdin(r#"sin'gle"#)
        .assert()
        .stdout(r#"sin\'gle"#)
        .success();
}

// Time applet CLI tests

#[test]
fn test_tsdec_verbose_cli_stdin() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["tsdec", "-v"])
        .write_stdin("1")
        .assert()
        .stdout("1970-01-01T00:00:01Z")
        .stderr("Used format: Seconds since Epoch\n")
        .success();
}

#[test]
fn test_tsenc_cli_stdin() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["tsenc"])
        .write_stdin("1970-01-01T00:00:01Z")
        .assert()
        .stdout("1")
        .success();
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["tsenc", "-t", "filetime"])
        .write_stdin("1601-01-01T00:00:01Z")
        .assert()
        .stdout("10000000")
        .success();
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["tsenc", "-i", "iso8601"])
        .write_stdin("1970-01-01T00:00:01Z")
        .assert()
        .stdout("1")
        .success();
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["tsenc", "-i", "rfc2822"])
        .write_stdin("Sat, 12 Jun 1993 13:25:19 GMT")
        .assert()
        .stdout("739891519")
        .success();
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["tsenc", "-i", "rfc3339"])
        .write_stdin("1985-04-12T23:20:50.52Z")
        .assert()
        .stdout("482196050")
        .success();
}

// XorApplet CLI tests

#[test]
fn test_hex_key_cli() {
    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["xor", "-x", "41", "AAAA"])
        .assert()
        .stdout(&b"\0\0\0\0"[..])
        .success();
}

#[test]
fn test_key_file_cli_stdin() {
    let rand_key: [u8; 17] = [
        0x7f, 0x5a, 0x88, 0x7b, 0xe8, 0x81, 0xd6, 0x5e, 0x39, 0xf4, 0x7e, 0x25, 0xf2, 0x05, 0xdc,
        0x22, 0x86,
    ];
    let zero_data = [0u8; 32];

    let mut tmpkey = NamedTempFile::new().unwrap();
    tmpkey.write_all(&rand_key).unwrap();

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["xor", "-f", tmpkey.path().to_str().expect("Could not get path as str")])
        .write_stdin(&zero_data[..])
        .assert()
        .stdout(
            &b"\x7fZ\x88{\xE8\x81\xD6^9\xF4~%\xF2\x05\xDC\"\x86\x7fZ\x88{\xE8\x81\xD6^9\xF4~%\xF2\x05\xDC"
                [..],
        )
        .success();

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args([
            "xor",
            "-f",
            tmpkey.path().to_str().expect("Could not get path as str"),
        ])
        .write_stdin(&rand_key[..])
        .assert()
        .stdout(&b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"[..])
        .success();
}

// BgrepApplet CLI tests

#[test]
fn test_bgrep_cli() {
    let mut data: [u8; 10] = [0; 10];
    for i in 0..10 {
        data[i] = i as u8;
    }

    let mut tmpfile = NamedTempFile::new().unwrap();
    tmpfile.write_all(&data).unwrap();

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args(["bgrep", "-x", "020304", tmpfile.path().to_str().unwrap()])
        .assert()
        .stdout("0x2\n")
        .success();
}

#[test]
fn test_bgrep_cli_multiple() {
    let mut tmpfile1 = NamedTempFile::new().unwrap();
    tmpfile1.write_all(b"tmpfile1").unwrap();

    let mut tmpfile2 = NamedTempFile::new().unwrap();
    tmpfile2.write_all(b"2tmpfile").unwrap();

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args([
            "bgrep",
            "tmpfile",
            tmpfile1.path().to_str().unwrap(),
            tmpfile2.path().to_str().unwrap(),
        ])
        .assert()
        .stdout(contains(": 0x0\n"))
        .stdout(contains(": 0x1\n"))
        .success();
}

#[test]
fn test_bgrep_recursive() {
    let tmp_dir = TempDir::new().unwrap();

    {
        let mut tmp_file = File::create(tmp_dir.path().join("test_file.bin")).unwrap();
        tmp_file.write_all(b"2tmpfile").unwrap();
    }

    Command::cargo_bin("rsbkb")
        .expect("Could not run binary")
        .args([
            "bgrep",
            "--recursive",
            "tmpfile",
            tmp_dir
                .path()
                .to_str()
                .expect("Could not convert temp path to unicode"),
        ])
        .assert()
        .stdout(contains(": 0x1\n"))
        .success();
}
