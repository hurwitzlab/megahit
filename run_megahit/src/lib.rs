extern crate clap;
extern crate regex;

use clap::{App, Arg};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::process::{Command, Stdio};
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct SplitPath {
    stem: String,
    ext: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    query: Vec<String>,
    out_dir: PathBuf,
    num_concurrent_jobs: Option<u32>,
    num_halt: Option<u32>,
    min_count: Option<u32>,
    k_min: Option<u32>,
    k_max: Option<u32>,
    k_step: Option<u32>,
    memory: Option<f32>,
    min_contig_length: Option<u32>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum ReadDirection {
    Forward,
    Reverse,
}

type MyResult<T> = Result<T, Box<dyn Error>>;
type ReadPair = HashMap<ReadDirection, String>;
type ReadPairLookup = HashMap<String, ReadPair>;
type SingleReads = Vec<String>;

// --------------------------------------------------
pub fn get_args() -> MyResult<Config> {
    let matches = App::new("run_megahit")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@email.arizona.edu>")
        .about("Runs TrimGalore")
        .arg(
            Arg::with_name("query")
                .short("Q")
                .long("query")
                .value_name("FILE_OR_DIR")
                .help("File input or directory")
                .required(true)
                .min_values(1),
        )
        .arg(
            Arg::with_name("out_dir")
                .short("o")
                .long("out_dir")
                .value_name("DIR")
                .help("Output directory"),
        )
        .arg(
            Arg::with_name("num_concurrent_jobs")
                .short("J")
                .long("num_concurrent_jobs")
                .value_name("INT")
                .default_value("8")
                .help("Number of concurrent jobs for parallel"),
        )
        .arg(
            Arg::with_name("num_halt")
                .short("H")
                .long("num_halt")
                .value_name("INT")
                .default_value("0")
                .help("Halt after this many failing jobs"),
        )
        .arg(
            Arg::with_name("min_count")
                .long("min_count")
                .value_name("INT")
                .help("minimum multiplicity for filtering (k_min+1)-mers")
        )
        .arg(
            Arg::with_name("k_min")
                .long("k_min")
                .value_name("INT")
                .help("minimum kmer size (<= 255), must be odd number")
        )
        .arg(
            Arg::with_name("k_max")
                .long("k_max")
                .value_name("INT")
                .help("maximum kmer size (<= 255), must be odd number")
        )
        .arg(
            Arg::with_name("k_step")
                .long("k_step")
                .value_name("INT")
                .help("increment of kmer size of each iteration (<= 28), must be even number")
        )
        .arg(
            Arg::with_name("min_contig_len")
                .long("min_contig_len")
                .value_name("INT")
                .help("minimum length of contigs to output")
        )
        .arg(
            Arg::with_name("memory")
                .short("m")
                .long("memory")
                .value_name("FLOAT")
                .default_value("1000000000")
                .help("Amount/percentage of memory"),
        )
        .get_matches();

    let out_dir = match matches.value_of("out_dir") {
        Some(x) => PathBuf::from(x),
        _ => {
            let cwd = env::current_dir()?;
            cwd.join(PathBuf::from("megahit-out"))
        }
    };

    let num_concurrent_jobs = matches
        .value_of("num_concurrent_jobs")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let num_halt = matches
        .value_of("num_halt")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let min_count = matches
        .value_of("min_count")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let k_min = matches
        .value_of("k_min")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let k_max = matches
        .value_of("k_max")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let k_step = matches
        .value_of("k_step")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let min_contig_length = matches
        .value_of("min_contig_len")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let memory = matches
        .value_of("memory")
        .and_then(|x| x.trim().parse::<f32>().ok());

    Ok(Config {
        query: matches.values_of_lossy("query").unwrap(),
        out_dir,
        num_concurrent_jobs,
        num_halt,
        min_count,
        k_min,
        k_max,
        k_step,
        min_contig_length,
        memory,
    })
}

// --------------------------------------------------
pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.query)?;

    if files.is_empty() {
        let msg = format!("No input files from query \"{:?}\"", &config.query);
        return Err(From::from(msg));
    }

    let (pairs, singles) = classify(&files)?;

    println!(
        "Processing {} pair, {} single.",
        pairs.keys().len(),
        singles.len()
    );

    let jobs = make_jobs(&config, pairs, singles)?;

    run_jobs(
        &jobs,
        "Running Megahit",
        config.num_concurrent_jobs.unwrap_or(8),
        config.num_halt.unwrap_or(0),
    )?;

    println!("Done, see output in \"{}\"", &config.out_dir.display());

    Ok(())
}

// --------------------------------------------------
fn make_jobs(
    config: &Config,
    pairs: ReadPairLookup,
    singles: SingleReads,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut args: Vec<String> = vec![];

    if let Some(min_count) = config.min_count {
        args.push(format!("--min-count {}", min_count));
    }

    if let Some(k_min) = config.k_min {
        args.push(format!("--k-min {}", k_min));
    }

    if let Some(k_max) = config.k_max {
        args.push(format!("--k-max {}", k_max));
    }

    if let Some(k_step) = config.k_step {
        args.push(format!("--k-step {}", k_step));
    }

    if let Some(min_contig_length) = config.min_contig_length {
        args.push(format!("--min-contig-len {}", min_contig_length));
    }

    if let Some(memory) = config.memory {
        args.push(format!("--memory {}", memory));
    }

    let mut jobs: Vec<String> = vec![];
    for (i, (sample, val)) in pairs.iter().enumerate() {
        println!("{:3}: Pair {}", i + 1, sample);

        if let (Some(fwd), Some(rev)) = (
            val.get(&ReadDirection::Forward),
            val.get(&ReadDirection::Reverse),
        ) {
            jobs.push(format!(
                "megahit -o {} {} -1 {} -2 {}",
                config.out_dir.display(),
                args.join(" "),
                fwd,
                rev,
            ));
        }
    }

    for (i, file) in singles.iter().enumerate() {
        let path = Path::new(file);
        let basename = path.file_name().expect("basename");
        let basename = &basename.to_string_lossy().to_string();

        println!("{:3}: Single {}", i + 1, basename);

        jobs.push(format!(
            "megahit -o {} {} -r {}",
            config.out_dir.display(),
            args.join(" "),
            file,
        ));
    }

    Ok(jobs)
}

// --------------------------------------------------
fn find_files(paths: &[String]) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files = vec![];
    for path in paths {
        let meta = fs::metadata(path)?;
        if meta.is_file() {
            files.push(path.to_owned());
        } else {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let meta = entry.metadata()?;
                if meta.is_file() {
                    files.push(entry.path().display().to_string());
                }
            }
        };
    }

    if files.is_empty() {
        return Err(From::from("No input files"));
    }

    Ok(files)
}

// --------------------------------------------------
fn classify(
    paths: &[String],
) -> Result<(ReadPairLookup, SingleReads), Box<dyn Error>> {
    let paths = paths.iter().map(Path::new);
    let mut exts: Vec<String> =
        paths.clone().map(get_extension).filter_map(|x| x).collect();
    exts.dedup();

    let dots = Regex::new(r"\.").unwrap();
    let exts: Vec<String> = exts
        .into_iter()
        .map(|x| dots.replace(&x, r"\.").to_string())
        .collect();

    let pattern = format!(r"(.+)[_-][Rr]?([12])?\.(?:{})$", exts.join("|"));
    let re = Regex::new(&pattern).unwrap();
    let mut pairs: ReadPairLookup = HashMap::new();
    let mut singles: Vec<String> = vec![];

    for path in paths.map(Path::new) {
        let path_str = path.to_str().expect("Convert path");

        if let Some(file_name) = path.file_name() {
            let basename = file_name.to_string_lossy();
            if let Some(cap) = re.captures(&basename) {
                let sample_name = &cap[1];
                let direction = if &cap[2] == "1" {
                    ReadDirection::Forward
                } else {
                    ReadDirection::Reverse
                };

                if !pairs.contains_key(sample_name) {
                    let mut pair: ReadPair = HashMap::new();
                    pair.insert(direction, path_str.to_string());
                    pairs.insert(sample_name.to_string(), pair);
                } else if let Some(pair) = pairs.get_mut(sample_name) {
                    pair.insert(direction, path_str.to_string());
                }
            } else {
                singles.push(path_str.to_string());
            }
        }
    }

    let bad: Vec<String> = pairs
        .iter()
        .filter_map(|(k, v)| {
            if !v.contains_key(&ReadDirection::Forward)
                || !v.contains_key(&ReadDirection::Reverse)
            {
                Some(k.to_string())
            } else {
                None
            }
        })
        .collect();

    // Push unpaired samples to the singles
    for key in bad {
        if let Some(pair) = pairs.get(&key) {
            for val in pair.values() {
                singles.push(val.to_string());
            }
        }
        pairs.remove(&key);
    }

    Ok((pairs, singles))
}

// --------------------------------------------------
/// Returns the extension plus optional ".gz"
fn get_extension(path: &Path) -> Option<String> {
    let re = Regex::new(r"\.([^.]+(?:\.gz)?)$").unwrap();
    if let Some(basename) = path.file_name() {
        let basename = basename.to_string_lossy();
        if let Some(cap) = re.captures(&basename) {
            return Some(cap[1].to_string());
        }
    }
    None
}

// --------------------------------------------------
fn run_jobs(
    jobs: &[String],
    msg: &str,
    num_concurrent_jobs: u32,
    num_halt: u32,
) -> MyResult<()> {
    let num_jobs = jobs.len();

    if num_jobs > 0 {
        println!(
            "{} (# {} job{} @ {})",
            msg,
            num_jobs,
            if num_jobs == 1 { "" } else { "s" },
            num_concurrent_jobs,
        );

        let mut args: Vec<String> =
            vec!["-j".to_string(), num_concurrent_jobs.to_string()];

        if num_halt > 0 {
            args.push("--halt".to_string());
            args.push(format!("soon,fail={}", num_halt.to_string()));
        }

        let mut process = Command::new("parallel")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()?;

        {
            let stdin = process.stdin.as_mut().expect("Failed to open stdin");
            stdin
                .write_all(jobs.join("\n").as_bytes())
                .expect("Failed to write to stdin");
        }

        let result = process.wait()?;
        if !result.success() {
            return Err(From::from("Failed to run jobs in parallel"));
        }
    }

    Ok(())
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extension() {
        assert_eq!(
            get_extension(Path::new("foo.fna")),
            Some("fna".to_string())
        );

        assert_eq!(
            get_extension(Path::new("foo.fasta.gz")),
            Some("fasta.gz".to_string())
        );

        assert_eq!(
            get_extension(Path::new("foo.fa.gz")),
            Some("fa.gz".to_string())
        );

        assert_eq!(
            get_extension(Path::new("foo.fasta")),
            Some("fasta".to_string())
        );

        assert_eq!(get_extension(Path::new("foo.fq")), Some("fq".to_string()));

        assert_eq!(get_extension(Path::new("foo")), None);
    }

    #[test]
    fn test_classify() {
        let res = classify(&["ERR1711926.fastq.gz".to_string()]);
        assert!(res.is_ok());

        if let Ok((pairs, singles)) = res {
            assert_eq!(pairs.len(), 0);
            assert_eq!(singles.len(), 1);
        }

        let res = classify(&[
            "/foo/bar/ERR1711926_1.fastq.gz".to_string(),
            "/foo/bar/ERR1711926_2.fastq.gz".to_string(),
            "/foo/bar/ERR1711927-R1.fastq.gz".to_string(),
            "/foo/bar/ERR1711927_R2.fastq.gz".to_string(),
            "/foo/bar/ERR1711928.fastq.gz".to_string(),
            "/foo/bar/ERR1711929_1.fastq.gz".to_string(),
        ]);
        assert!(res.is_ok());

        if let Ok((pairs, singles)) = res {
            assert_eq!(pairs.len(), 2);
            assert_eq!(singles.len(), 2);

            assert!(pairs.contains_key("ERR1711926"));
            assert!(pairs.contains_key("ERR1711927"));

            //assert!(!singles.contains_key("ERR1711928"));
            //assert!(!singles.contains_key("ERR1711929"));

            if let Some(val) = pairs.get("ERR1711926") {
                assert!(val.contains_key(&ReadDirection::Forward));
                assert!(val.contains_key(&ReadDirection::Reverse));

                if let Some(fwd) = val.get(&ReadDirection::Forward) {
                    assert_eq!(fwd, &"/foo/bar/ERR1711926_1.fastq.gz");
                }
                if let Some(rev) = val.get(&ReadDirection::Reverse) {
                    assert_eq!(rev, &"/foo/bar/ERR1711926_2.fastq.gz");
                }
            }

            if let Some(val) = pairs.get("ERR1711927") {
                assert!(val.contains_key(&ReadDirection::Forward));
                assert!(val.contains_key(&ReadDirection::Reverse));

                if let Some(fwd) = val.get(&ReadDirection::Forward) {
                    assert_eq!(fwd, &"/foo/bar/ERR1711927-R1.fastq.gz");
                }
                if let Some(rev) = val.get(&ReadDirection::Reverse) {
                    assert_eq!(rev, &"/foo/bar/ERR1711927_R2.fastq.gz");
                }
            }
        }
    }
}
