use std::fs;
use std::process::{Command, Output};

use victory_shader::{Container, FourCC};

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_xemss"))
}

fn expect_success(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// One compute stage with a 64x1x1 group and a single s_endpgm instruction.
fn synthetic_compute(include_ls: bool) -> Vec<u8> {
    const RECORD: usize = 0x68;
    const STAGE: usize = RECORD + 0x54;
    const CODE: usize = STAGE + 0x34;
    let mut payload = vec![0; CODE];
    payload[..8].copy_from_slice(&6u64.to_le_bytes());
    for slot in [0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x48] {
        payload[slot..slot + 8].copy_from_slice(&0xDEAD_BEEF_DEAD_BEEFu64.to_le_bytes());
    }
    if include_ls {
        payload[0x08..0x10].copy_from_slice(&(RECORD as u64).to_le_bytes());
    }
    payload[0x60..0x68].copy_from_slice(&(RECORD as u64).to_le_bytes());
    payload[RECORD..RECORD + 8].copy_from_slice(&(CODE as u64).to_le_bytes());
    payload[RECORD + 8..RECORD + 16].copy_from_slice(&4u64.to_le_bytes());
    for (axis, threads) in [64u32, 1, 1].into_iter().enumerate() {
        let offset = STAGE + 0x28 + axis * 4;
        payload[offset..offset + 4].copy_from_slice(&threads.to_le_bytes());
    }
    payload.extend_from_slice(&0xBF81_0000u32.to_le_bytes());

    let mut fssm = b"FSSM".to_vec();
    fssm.extend_from_slice(&0x101u32.to_le_bytes());
    fssm.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    fssm.extend_from_slice(&payload);
    let stream = miniz_oxide::deflate::compress_to_vec_zlib(&fssm, 6);
    let mut xbch = b"XBCH".to_vec();
    xbch.extend_from_slice(&3u32.to_le_bytes());
    xbch.extend_from_slice(&(stream.len() as u64).to_le_bytes());
    xbch.extend_from_slice(&(fssm.len() as u64).to_le_bytes());
    xbch.extend_from_slice(&stream);

    let mut dxil = vec![0; 0x18];
    dxil.extend_from_slice(b"XDAT");
    for field in [1u32, 0x24, 0x0D, 0x30, xbch.len() as u32] {
        dxil.extend_from_slice(&field.to_le_bytes());
    }
    dxil.extend_from_slice(&xbch);

    let mut dxbc = vec![0; 0x24];
    dxbc[..4].copy_from_slice(b"DXBC");
    dxbc[0x14..0x16].copy_from_slice(&1u16.to_le_bytes());
    dxbc[0x18..0x1C].copy_from_slice(&(0x2C + dxil.len() as u32).to_le_bytes());
    dxbc[0x1C..0x20].copy_from_slice(&1u32.to_le_bytes());
    dxbc[0x20..0x24].copy_from_slice(&0x24u32.to_le_bytes());
    dxbc.extend_from_slice(b"DXIL");
    dxbc.extend_from_slice(&(dxil.len() as u32).to_le_bytes());
    dxbc.extend_from_slice(&dxil);
    dxbc
}

#[test]
fn translates_a_synthetic_shader_without_overwriting() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("compute.dxbc");
    let output = dir.path().join("compute.dxil");
    fs::write(&input, synthetic_compute(false)).unwrap();

    let info = expect_success(
        command()
            .arg("inspect-shader")
            .arg(&input)
            .output()
            .unwrap(),
    );
    assert!(info.contains("Native Cs: 4 code bytes"), "{info}");

    let translate = || {
        let mut cmd = command();
        cmd.arg("translate-shader")
            .arg(&input)
            .arg("-o")
            .arg(&output);
        if let Some(validator) = std::env::var_os("XEMSS_TEST_DXIL") {
            cmd.arg("--validate").env("VICTORY_DXIL", validator);
        }
        cmd.output().unwrap()
    };
    let info = expect_success(translate());
    assert!(info.contains("Translated Cs"), "{info}");
    let emitted = fs::read(&output).unwrap();
    let container = Container::parse(&emitted).unwrap();
    assert!(container.part(FourCC::DXIL).is_some());

    fs::write(&output, b"keep existing file").unwrap();
    let refused = translate();
    assert!(!refused.status.success());
    let error = String::from_utf8_lossy(&refused.stderr);
    assert!(error.contains("without overwriting"), "{error}");
    assert_eq!(fs::read(&output).unwrap(), b"keep existing file");
}

#[test]
fn batch_reports_failures_and_writes_the_other_programs() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("corpus");
    let output = dir.path().join("out");
    fs::create_dir(&input).unwrap();
    fs::write(input.join("bad.dxbc"), b"bad container").unwrap();
    fs::write(input.join("mixed.DXBC"), synthetic_compute(true)).unwrap();
    fs::write(input.join("ordinary.dxbc"), synthetic_compute(false)).unwrap();
    fs::write(input.join("notes.txt"), b"ignored").unwrap();
    let run = || {
        let mut cmd = command();
        cmd.arg("translate-shaders")
            .arg(&input)
            .arg("-o")
            .arg(&output);
        if let Some(validator) = std::env::var_os("XEMSS_TEST_DXIL") {
            cmd.arg("--validate").env("VICTORY_DXIL", validator);
        }
        cmd.output().unwrap()
    };

    let result = run();
    assert!(!result.status.success());
    let stdout = String::from_utf8(result.stdout).unwrap();
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stdout
            .contains("3 containers, 2 programs written, 1 program failures, 1 container failures"),
        "{stdout}"
    );
    assert!(stdout.contains("translation:"), "{stdout}");
    assert!(stderr.contains("bad.dxbc"), "{stderr}");
    assert!(stderr.contains("program 1 (Ls)"), "{stderr}");
    for name in ["mixed.DXBC.2.Cs.dxil", "ordinary.dxbc.1.Cs.dxil"] {
        let bytes = fs::read(output.join(name)).unwrap();
        assert!(
            Container::parse(&bytes)
                .unwrap()
                .part(FourCC::DXIL)
                .is_some()
        );
    }
    assert_eq!(fs::read_dir(&output).unwrap().count(), 2);

    let repeated = run();
    assert!(!repeated.status.success());
    let error = String::from_utf8(repeated.stderr).unwrap();
    assert!(error.contains("without overwriting"), "{error}");
    assert_eq!(fs::read_dir(output).unwrap().count(), 2);
}

#[test]
fn rejects_an_invalid_input_without_creating_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("invalid.dxbc");
    let output = dir.path().join("out.dxil");
    fs::write(&input, b"invalid input").unwrap();
    let result = command()
        .arg("translate-shader")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!output.exists());
}

#[test]
#[cfg(all(windows, target_arch = "x86_64"))]
fn inspects_its_own_amd64_executable() {
    let info = expect_success(
        command()
            .args(["inspect-pe", env!("CARGO_BIN_EXE_xemss"), "--imports"])
            .output()
            .unwrap(),
    );
    assert!(info.contains("AMD64 PE32+ image"), "{info}");
    assert!(info.contains("  import: "), "{info}");
}
