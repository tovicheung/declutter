use std::{env, ffi::OsStr, fs, path::{Path, PathBuf}};

use declutter::{Allow, parse_yaml_ruleset, RuleSet};

extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};

fn option_osstr_eq_string(option: Option<&OsStr>, string: &String) -> bool {
    if let Some(osstr) = option {
        if let Some(str) = osstr.to_str() {
            if str == string {
                return true;
            }
        }
    }
    false
}

/// Returns Result<{everything clean?}, {error message}>
fn is_dir_clean(path: &PathBuf, ruleset: &RuleSet) -> Result<bool, String> {
    if !path.is_dir() {
        return Err("path is not directory".to_string());
    }

    let mut clean = true;

    for entry in path.read_dir().map_err(|_| "read_dir() failed")? {
        let entry_path = entry.map_err(|err| err.to_string())?.path();
        'check_rules: {
            for rule in &ruleset.allows {
                if match rule {
                    Allow::Dir => entry_path.is_dir(),
                    Allow::Ext(ext) => option_osstr_eq_string(entry_path.extension(), ext),
                    Allow::Name(name) => option_osstr_eq_string(entry_path.file_name(), name),
                } {
                    // this entry matches one of the rules, check recursive and leave
                    if entry_path.is_dir() && ruleset.recursive {
                        clean = clean && is_dir_clean(&entry_path, &ruleset)?;
                    }
                    break 'check_rules;
                }
            }
            // this entry does not match any of the rules
            clean = false;
            println!("\x1b[1;33m>\x1b[m {}", entry_path.to_str().unwrap());
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

fn error(when: String, err: String) {
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
        Err(msg) => return error(format!("reading config at {}", config_path), msg),
    };

    let hashmap = match doc.into_hash() {
        Some(hash) => hash,
        None => return error(format!("parsing config at {}", config_path), "cannot convert yaml to hashmap".to_string()),
    };

    println!("\x1b[1;33mChecking for clutter\x1b[m");

    let mut clean = true;
    
    for (key, body) in hashmap {
        clean = clean && is_yaml_entry_clean(key, body)
            .unwrap_or_else(
                |(when, err)| {error(when, err); false}
            )
    }

    if clean {
        println!("\x1b[1;32m> No clutter! Well done!\x1b[m")
    }
}
