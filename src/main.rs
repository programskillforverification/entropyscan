use anyhow::{Ok, Result};
use std::env::args;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

const MAX_FILESIZE: u64 = 2147483648;
const MAX_ENTROPY_CHUNK: usize = 2560000;

fn calculate_entropy(path: &Path) -> Result<f64, anyhow::Error> {
    if fs::metadata(path)?.is_dir() {
        return Err(anyhow::anyhow!("This is a directory!"));
    }
    let file_length = fs::metadata(path)?.len();
    if file_length > MAX_FILESIZE {
        return Err(anyhow::anyhow!("File too large"));
    }

    let mut reader = BufReader::new(fs::File::open(path)?);
    let mut buffer = vec![0; MAX_ENTROPY_CHUNK];
    let mut entropy = 0.0f64;
    loop {
        let n = reader.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        let mut frequency: [u32; 256] = [0; 256];
        let mut total_bytes = 0;
        buffer.iter().for_each(|&byte| {
            frequency[byte as usize] += 1;
            total_bytes += 1;
        });
        frequency
            .iter()
            .filter(|&&count| count > 0)
            .for_each(|&count| {
                let probability = count as f64 / total_bytes as f64;
                entropy -= probability * probability.log2();
            });
    }

    Ok(entropy)
}

fn collect_targets(parent_path: &PathBuf) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut targets = vec![];
    for entry in fs::read_dir(parent_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_targets(&path)?;
        } else {
            targets.push(path);
        }
    }
    Ok(targets)
}

fn main() -> Result<(), anyhow::Error> {
    if let Some(target) = args().nth(1) {
        let targets = collect_targets(&PathBuf::from(&target))?;
        for target in targets.iter() {
            let entropy = calculate_entropy(&PathBuf::from(target))?;
            println!("{target:?}: {entropy}");
        }
        Ok(())
    } else {
        panic!("No path provided!")
    }
}
