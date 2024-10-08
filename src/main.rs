use std::{env, fs, path::{Path, PathBuf}};

use declutter::{parse_yaml_ruleset, RuleSet};

use yaml_rust::{Yaml, YamlLoader};

/// Returns Result<{everything clean?}, {error message}>
fn is_dir_clean(path: &PathBuf, ruleset: &RuleSet) -> Result<bool, String> {
    if !path.is_dir() {
        return Err("path is not directory".to_string());
    }

    let mut clean = true;

    for entry in path.read_dir().map_err(|_| "read_dir() failed")?.map(|e| e.map_err(|err| err.to_string())) {
        let entry = entry?;
        let entry_path = entry.path();
        'check_rules: {
            for rule in &ruleset.rules {
                if !rule.check(&entry)? {
                    // violated a rule
                    clean = false;
                    println!("\x1b[1;33m>\x1b[m {}", entry_path.to_str().unwrap());
                    break 'check_rules;
                }
            }
            // no violated rules
            if entry_path.is_dir() && ruleset.recursive {
                clean = clean && is_dir_clean(&entry_path, &ruleset)?;
            }
        }
    }
    Ok(clean)
}

/// Returns path-content as key-value hash
fn read_config(config_path: &String) -> Result<Yaml, String> {
    YamlLoader::load_from_str(
        fs::read_to_string(config_path)
            .map_err(|err| err.to_string())?
            .as_str()
    )
        .map_err(|err| err.to_string())?
        .into_iter()
        .next()
        .ok_or("no contents".to_string())
        // only read the first document
}

fn print_error(when: String, err: String) {
    eprintln!("\x1b[1;31m> Error\x1b[m when {}", when);
    eprintln!("\x1b[1;31m> \x1b[m{}", err);
}

// Returns Result<{is there clutter?}, ({error when ...}, {error message})>
fn is_yaml_entry_clean(key: Yaml, body: Yaml) -> Result<bool, (String, String)> {
    let path_string = key.clone().into_string().ok_or(("parsing yaml".to_string(), format!("expected string path but got {:?}", key)))?;
    Ok(
        is_dir_clean(
            &Path::new(path_string.as_str())
                .canonicalize()
                .map_err(|err| (format!("accessing path {}", path_string), err.to_string()))?,
            &parse_yaml_ruleset(body)
                .map_err(|err| ("parsing yaml ruleset".to_string(), err))?
        ).map_err(|err| (format!("checking entries at path {}", path_string), err))?
    )
}

fn main() {
    let config_path = env::args().nth(1).unwrap_or("declutter.yaml".to_string());

    println!("\x1b[1mReading config from {}\x1b[m", config_path);

    let doc = match read_config(&config_path) {
        Ok(doc) => doc,
        Err(msg) => return print_error(format!("reading config at {}", config_path), msg),
    };

    let hashmap = match doc.into_hash() {
        Some(hash) => hash,
        None => return print_error(format!("parsing config at {}", config_path), "cannot convert yaml to hashmap".to_string()),
    };

    let mut clean = true;
    
    for (key, body) in hashmap {
        clean = is_yaml_entry_clean(key, body)
            .unwrap_or_else(
                |(when, err)| {print_error(when, err); false}
            ) && clean;
    }

    if clean {
        println!("\x1b[1;32m> No clutter! Well done!\x1b[m")
    }
}
