use yaml_rust::Yaml;

pub struct RuleSet {
    pub recursive: bool,
    pub allows: Vec<Allow>,
}

impl IntoIterator for RuleSet {
    type Item = Allow;
    type IntoIter = std::vec::IntoIter<Allow>;

    fn into_iter(self) -> Self::IntoIter {
        self.allows.into_iter()
    }
}

#[derive(PartialEq)]
pub enum Allow {
    Dir,
    Ext(String),
    Name(String),
}

fn parse_allow_type(item: Yaml) -> Result<Allow, String> {
    let x = item.into_string().ok_or("expected string in allow-type")?;
    if x == "dir" {
        return Ok(Allow::Dir);
    }
    if !x.starts_with(".") {
        return Err(format!("Invalid extension: {}", x))
    }
    Ok(Allow::Ext(x.strip_prefix(".").unwrap().to_string()))
}

fn parse_allow_name(item: Yaml) -> Result<Allow, String> {
    let x = item.into_string().ok_or("expected string in allow-name")?;
    Ok(Allow::Name(x))
}

fn parse_array_or_one(body: Yaml, func: fn(Yaml) -> Result<Allow, String>) -> Result<Vec<Allow>, String> {
    Ok(
        match body {
            Yaml::Array(vec) => {
                // Result implements FromIterator
                vec.into_iter().map(func).collect::<Result<Vec<_>, String>>()?
            },
            other => vec![parse_allow_type(other)?],
        }
    )
}

pub fn parse_yaml(yaml: Yaml) -> Result<RuleSet, String> {
    let hash = yaml.into_hash().ok_or("expected key-value pairs under path")?;

    let mut recursive = true;
    let mut allows = Vec::<Allow>::new();

    for (key, value) in hash {
        let x = key.into_string().ok_or("expected string as key")?;
        match x.as_str() {
            "recursive" => match value {
                Yaml::Boolean(bool) => recursive = bool,
                _ => return Err("expected boolean for recursive".to_string()),
            },
            "allow-type" => allows.extend(parse_array_or_one(value, parse_allow_type)?),
            "allow-name" => allows.extend(parse_array_or_one(value, parse_allow_name)?),
            other => return Err(format!("unknown key: {}", other).to_string()),
        };
    }

    if allows.len() == 0 {
        return Err("empty ruleset".to_string());
    }

    // recursive implies allow directories
    if recursive && !allows.contains(&Allow::Dir) {
        allows.push(Allow::Dir);
    }

    Ok(
        RuleSet {
            recursive,
            allows,
        }
    )
}
