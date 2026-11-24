use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use victory_pe::ImageInfo;
use victory_shader::{Container, FourCC, emit, native_programs};

const MAX_PE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_SHADER_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Parser)]
#[command(version, about = "XEMSS · Xbox Series S compatibility research tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect an AMD64 PE32+ image without loading or executing it.
    InspectPe {
        input: PathBuf,
        /// List imported symbols, including delay-load imports.
        #[arg(long)]
        imports: bool,
    },
    /// Inspect the parts and native shader stages in a single DXBC container.
    InspectShader { input: PathBuf },
    /// Translate one supported console shader to a new DXIL file.
    TranslateShader {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        /// Validate the DXIL with an installed Windows SDK before writing it.
        #[arg(long)]
        validate: bool,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::InspectPe { input, imports } => inspect_pe(&input, imports),
        Command::InspectShader { input } => inspect_shader(&input),
        Command::TranslateShader {
            input,
            output,
            validate,
        } => translate_shader(&input, &output, validate),
    }
}

fn read_input(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file(),
        "{} is not a regular file",
        path.display()
    );
    let too_large = || {
        format!(
            "{} exceeds the {max_bytes}-byte input limit",
            path.display()
        )
    };
    ensure!(metadata.len() <= max_bytes, "{}", too_large());
    let mut bytes = Vec::new();
    file.take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("reading {}", path.display()))?;
    ensure!(bytes.len() as u64 <= max_bytes, "{}", too_large());
    Ok(bytes)
}

fn inspect_pe(input: &Path, list_imports: bool) -> Result<()> {
    let bytes = read_input(input, MAX_PE_BYTES)?;
    let image = ImageInfo::parse(&bytes).with_context(|| format!("parsing {}", input.display()))?;
    println!(
        "AMD64 PE32+ {}",
        if image.is_dll() { "DLL" } else { "image" }
    );
    println!("Subsystem: {}", image.subsystem_name());
    println!("Entry RVA: {:#x}", image.entry_rva);
    println!("Sections: {}", image.sections.len());
    println!(
        "Imports: {} from {} DLLs",
        image.import_count(),
        image.imports.len()
    );
    println!("Delay-load DLLs: {}", image.delay_imports.len());
    println!("Exports: {}", image.exports.len());
    println!(
        "TLS callbacks: {}",
        image.tls.as_ref().map_or(0, |tls| tls.callback_rvas.len())
    );
    if list_imports {
        for (modules, label) in [
            (&image.imports, "import"),
            (&image.delay_imports, "delay import"),
        ] {
            for module in modules {
                for entry in &module.entries {
                    println!(
                        "  {label}: {}!{}",
                        module.dll.escape_debug(),
                        entry.symbol.to_string().escape_debug()
                    );
                }
            }
        }
    }
    Ok(())
}

fn inspect_shader(input: &Path) -> Result<()> {
    let bytes = read_input(input, MAX_SHADER_BYTES)?;
    let container =
        Container::parse(&bytes).with_context(|| format!("parsing {}", input.display()))?;
    println!(
        "DXBC {}.{} · {} parts",
        container.version.0,
        container.version.1,
        container.parts.len()
    );
    for part in &container.parts {
        println!(
            "  {}: {} bytes at {:#x}",
            part.fourcc,
            part.data.len(),
            part.offset
        );
    }
    if let Some(dxil) = container.part(FourCC::DXIL) {
        let programs = native_programs(dxil.data).context("reading console programs")?;
        for program in &programs {
            println!(
                "  Native {:?}: {} code bytes",
                program.stage,
                program.code.len()
            );
        }
        if programs.is_empty() {
            println!("  No supported console programs in the DXIL part");
        }
    }
    Ok(())
}

fn translate_shader(input: &Path, output: &Path, validate: bool) -> Result<()> {
    let bytes = read_input(input, MAX_SHADER_BYTES)?;
    let mut shader =
        emit::translate(&bytes).with_context(|| format!("translating {}", input.display()))?;
    if validate {
        validate_dxil(&mut shader.dxil)?;
    }
    let directory = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut staged = tempfile::NamedTempFile::new_in(directory)
        .with_context(|| format!("creating output in {}", directory.display()))?;
    staged
        .write_all(&shader.dxil)
        .context("writing translated DXIL")?;
    staged.persist_noclobber(output).with_context(|| {
        format!(
            "publishing {} without overwriting existing files",
            output.display()
        )
    })?;
    println!(
        "Translated {:?} to {} ({} bytes, {})",
        shader.stage,
        output.display(),
        shader.dxil.len(),
        if validate {
            "validated"
        } else {
            "validation not requested"
        }
    );
    Ok(())
}

#[cfg(windows)]
fn validate_dxil(bytes: &mut [u8]) -> Result<()> {
    use victory_shader::emit::validator::Validator;

    let path = Validator::default_path()
        .filter(|p| p.is_absolute())
        .context("set VICTORY_DXIL to the absolute path to your Windows SDK's dxil.dll")?;
    let mut validator = Validator::load(Some(&path)).map_err(anyhow::Error::msg)?;
    validator.validate(bytes).map_err(anyhow::Error::msg)
}

#[cfg(not(windows))]
fn validate_dxil(_bytes: &mut [u8]) -> Result<()> {
    anyhow::bail!("DXIL validation requires Windows and an installed Windows SDK")
}
