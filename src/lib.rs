use std::{ffi::OsStr, fs::DirEntry, os::windows::fs::MetadataExt, path::PathBuf};

use yaml_rust::Yaml;
use parse_size::parse_size;
pub struct RuleSet {
    pub recursive: bool,
    pub rules: Vec<Rule>,
}

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

#[derive(PartialEq)]
pub enum EntryType {
    Dir,
    File,
    Ext(String),
}

impl EntryType {
    fn check(&self, entry: &PathBuf) -> bool {
        match self {
            EntryType::Dir => entry.is_dir(),
            EntryType::File => entry.is_file(),
            EntryType::Ext(ext) => option_osstr_eq_string(entry.extension(), ext),
        }
    }
}

#[derive(PartialEq)]
pub enum Rule {
    AllowType(Vec<EntryType>),
    AllowName(Vec<String>),
    MinSize(u64),
    MaxSize(u64),
}

impl Rule {
    pub fn check(&self, entry: &DirEntry) -> Result<bool, String> {
        let metadata = entry.metadata().map_err(|err| err.to_string())?;
        Ok(match self {
            Rule::AllowType(types) => 
                types.into_iter().any(|typ| typ.check(&entry.path())),
            Rule::AllowName(names) => 
                names.into_iter().any(|name| entry.file_name().into_string().unwrap() == *name),
            Rule::MinSize(size) => *size <= metadata.file_size(),
            Rule::MaxSize(size) => metadata.file_size() <= *size,
        })
    }
}

fn parse_entry_type(item: Yaml) -> Result<EntryType, String> {
    let x = item.into_string().ok_or("expected string for entry type")?;
    if x == "dir" {
        return Ok(EntryType::Dir);
    }
    if x == "file" {
        return Ok(EntryType::File);
    }
    if !x.starts_with(".") {
        return Err(format!("Invalid extension: {}", x))
    }
    Ok(EntryType::Ext(x.strip_prefix(".").unwrap().to_string()))
}

fn parse_entry_name(item: Yaml) -> Result<String, String> {
    item.into_string().ok_or("expected string for entry name".to_string())
}

fn parse_array_or_one<T>(body: Yaml, func: fn(Yaml) -> Result<T, String>) -> Result<Vec<T>, String> {
    Ok(
        match body {
            Yaml::Array(vec) => {
                // Result implements FromIterator
                vec.into_iter().map(func).collect::<Result<Vec<_>, String>>()?
            },
            other => vec![func(other)?],
        }
    )
}

fn parse_file_size(value: Yaml) -> Result<u64, String> {
    match value {
        Yaml::Integer(x) => x.try_into().map_err(|_| format!("invalid file size: {}", x)),
        Yaml::String(string) => {
            parse_size(string).map_err(|err| err.to_string())
        },
        other => Err(format!("invalid file size: {:?}", other)),
    }
}

pub fn parse_yaml_ruleset(yaml: Yaml) -> Result<RuleSet, String> {
    let hashmap = yaml.into_hash().ok_or("expected key-value pairs under path")?;

    let mut recursive = true;
    let mut rules = Vec::<Rule>::with_capacity(hashmap.len());

    for (key, value) in hashmap {
        match key.into_string().ok_or("expected string as key")?.as_str() {
            "recursive" => recursive = value.into_bool().ok_or("expected bool for recursive")?,
            "allow-type" => rules.push(Rule::AllowType(parse_array_or_one(value, parse_entry_type)?)),
            "allow-name" => rules.push(Rule::AllowName(parse_array_or_one(value, parse_entry_name)?)),
            "min-size" => rules.push(Rule::MinSize(parse_file_size(value)?)),
            "max-size" => rules.push(Rule::MaxSize(parse_file_size(value)?)),
            other => return Err(format!("unknown key: {}", other).to_string()),
        };
    }

    if rules.len() == 0 {
        return Err("empty ruleset".to_string());
    }

    if recursive {
        for rule in &mut rules {
            if let Rule::AllowType(types) = rule {
                if !types.contains(&EntryType::Dir) {
                    types.push(EntryType::Dir);
                }
            }
        }
    }

    Ok(
        RuleSet {
            recursive,
            rules,
        }
    )
}
