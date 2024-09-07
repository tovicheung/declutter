use std::{ffi::OsStr, fs, path::{Path, PathBuf}};

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

fn check(path: &PathBuf, ruleset: &RuleSet) -> Result<(), String> {
    if !path.is_dir() {
        return Err("path is not directory".to_string());
    }

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
                println!("{} violated rules", p.to_str().unwrap())
            } else if p.is_dir() && ruleset.recursive {
                check(&p, &ruleset)?;
            }
        }
    }
    Ok(())
}

fn error(when: &str, path_string: String, err: String) {
    println!("\x1b[1;31m> Error\x1b[m when {} {}", when, path_string);
    println!("\x1b[1;31m> \x1b[m{}", err);
}

fn main() {
    let config_string = fs::read_to_string("declutter.yaml").expect("error reading file");
    let yaml = YamlLoader::load_from_str(config_string.as_str()).expect("error parsing yaml");
    let hash = yaml.into_iter().next().expect("no documents in yaml").into_hash().expect("error converting to hash");
    
    for (key, body) in hash {

        let path_string = key.into_string().expect("expected string path");
        match Path::new(path_string.as_str()).canonicalize() {
            Ok(path) => {
                match parse_yaml(body) {
                    Ok(ruleset) => {
                        if let Err(err) = check(&path, &ruleset) {
                            error("checking path", path_string, err);
                        }
                    },
                    Err(err) => error("parsing yaml under path", path_string, err),
                }
            },
            Err(err) => error("accessing path", path_string, err.to_string()),
        }
    }
}
