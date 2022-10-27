use loading_dock::{Config, Result, WithMessage};
use std::{env, fs, io};

const USAGE: &str = "usage: ldock {arg}";

fn main() {
    let (args, flags) = parse_args();

    // Get the current config, or a default one
    let mut cfg = load_config(flags.config.as_deref()).expect("error parsing config");

    // Determine the action being taken and then dispatch execution to it
    if args.is_empty() {
        // Didn't give us anything to work with, print some useage and exit
        panic!("{}", USAGE)
    }

    let result = match args[0].as_str() {
        "load" => {
            if args.len() < 2 {
                panic!("usage: ldock load {{filename}}")
            }
            loading_dock::stage_file(&mut cfg, &args[1])
        }
        "ls" => {
            for filename in cfg.staged {
                println!("{}", filename);
            }
            return;
        }
        _ => panic!("{}", USAGE),
    };
    result.expect("error execting command");

    // Save the config back to where it was
    persist_config(flags.config.as_deref(), &cfg).expect("error persisting config")
}

struct Flags {
    // If this is provided, use it as the config location instead of the default
    config: Option<String>,
}

// Parses the args and produces flags and arguments
fn parse_args() -> (Vec<String>, Flags) {
    let mut all: Vec<String> = env::args().collect();
    all.remove(0);

    // The above has our flags and our arguments (and our target)
    // included in it, so we'll split it out to make it easier to
    // determine what the user wants.
    let mut args: Vec<String> = vec![];
    let mut flags = Flags { config: None };

    for mut arg in all {
        if arg.starts_with("--") {
            arg.replace_range(0..2, "");

            let pieces: Vec<&str> = arg.split('=').collect();
            if pieces.len() != 2 {
                continue;
            }

            match pieces[0] {
                "config" => flags.config = Some(pieces[1].to_owned()),
                &_ => todo!(),
            }
            continue;
        }

        args.push(arg);
    }

    (args, flags)
}

fn default_config_path() -> String {
    let path = home::home_dir().expect("could not get home dir");
    format!(
        "{}/.ld",
        path.as_os_str()
            .to_str()
            .expect("unable to convert home dir to str")
    )
}

// Takes in the args from the line and produces a config
fn load_config(overide_path: Option<&str>) -> Result<Config> {
    let dcp = default_config_path();
    let path = overide_path.unwrap_or(&dcp);
    let config_str = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => match err.kind() {
            // If the file doesn't exist, that's cool. We'll just return a nice zero-value.
            io::ErrorKind::NotFound => return Ok(Config::default()),
            _ => {
                println!("hello");
                println!("{:?}, kind: {:?}", err, err.kind());
                return Err(err.into());
            }
        },
    };

    let cfg = match serde_json::from_str::<Config>(&config_str) {
        Ok(cfg) => cfg,
        Err(err) => {
            println!("Error parsing config: {}", err);
            Config::default()
        }
    };

    Ok(cfg)
}

fn persist_config(overide_path: Option<&str>, cfg: &Config) -> Result<()> {
    let dcp = default_config_path();
    let path = overide_path.unwrap_or(&dcp);
    let f = fs::File::create(path).with_context(&format!("issue creating {}", path))?;

    serde_json::to_writer(f, cfg).with_context(&format!("issue writing to file: {}", path))?;

    Ok(())
}
