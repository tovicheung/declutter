use std::{env, ffi::OsStr, fs, path::{Path, PathBuf}};

use declutter::{Allow, parse_yaml, RuleSet};

extern crate yaml_rust;
use yaml_rust::YamlLoader;

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

fn check(path: &PathBuf, ruleset: &RuleSet) -> Result<bool, String> {
    if !path.is_dir() {
        return Err("path is not directory".to_string());
    }

    let mut clutter = false;

    for entry in path.read_dir().map_err(|_| "read_dir() failed")? {
        if let Ok(entry) = entry {
            let p = entry.path();
            // println!("{:?}", p);

            let mut ok = false;

            for rule in &ruleset.allows {
                match rule {
                    Allow::Dir => {
                        if p.is_dir() {
                            ok = true;
                            break;
                        }
                    },
                    Allow::Ext(ext) => {
                        if option_osstr_eq_string(p.extension(), ext) {
                            ok = true;
                            break;
                        }
                    },
                    Allow::Name(name) => {
                        if option_osstr_eq_string(p.file_name(), name) {
                            ok = true;
                            break;
                        }
                    }
                }
            }

            if !ok {
                clutter = true;
                println!("\x1b[1;33m>\x1b[m {}", p.to_str().unwrap())
            } else if p.is_dir() && ruleset.recursive {
                check(&p, &ruleset)?;
            }
        }
    }
    Ok(clutter)
}

fn error(when: &str, path_string: String, err: String) {
    println!("\x1b[1;31m> Error\x1b[m when {} {}", when, path_string);
    println!("\x1b[1;31m> \x1b[m{}", err);
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let config_path = if args.len() >= 2 {
        args.iter().nth(1).unwrap().clone()
    } else {
        "declutter.yaml".to_string()
    };
    let config_string = match fs::read_to_string(&config_path) {
        Ok(string) => string,
        Err(err) => return error("reading config", config_path, err.to_string()),
    };
    let yaml = match YamlLoader::load_from_str(config_string.as_str()) {
        Ok(yaml) => yaml,
        Err(err) => return error("reading config", config_path, err.to_string()),
    };
    
    let doc = match yaml.into_iter().next() {
        Some(doc) => doc,
        None => return error("reading config", config_path, "no contents".to_string()),
    };

    let hash = match doc.into_hash() {
        Some(hash) => hash,
        None => return error("reading config", config_path, "expected key-value pairs".to_string()),
    };

    println!("\x1b[1;33mChecking for clutter\x1b[m");

    let mut clutter = false;
    
    for (key, body) in hash {

        let path_string = key.into_string().expect("expected string path");
        match Path::new(path_string.as_str()).canonicalize() {
            Ok(path) => {
                match parse_yaml(body) {
                    Ok(ruleset) => {
                        match check(&path, &ruleset) {
                            Ok(child_clutter) => clutter = clutter || child_clutter,
                            Err(err) => error("checking path", path_string, err),
                        }
                    },
                    Err(err) => error("parsing yaml under path", path_string, err),
                }
            },
            Err(err) => error("accessing path", path_string, err.to_string()),
        }
    }

    if !clutter {
        println!("\x1b[1;32m> No clutter! Well done!\x1b[m")
    }
}
