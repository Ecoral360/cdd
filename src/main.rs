use clap::{arg, command, Parser};
use once_cell::sync::Lazy;
use std::env::current_dir;
use std::env::{set_var, var, vars};
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;

#[derive(Parser)]
#[command(about)]
struct Cli {
    #[arg(short, long, group = "action")]
    into: Option<String>,

    #[arg(short, long, group = "action")]
    save: Option<String>,

    #[arg(short, long, group = "action")]
    remove: Option<String>,

    #[arg(short, long, group = "action")]
    list: bool,

    #[arg(short = 'e', long, group = "action")]
    exists: bool,

    dir: Option<String>,
}

static FILE_PATH: Lazy<String> = Lazy::new(|| {
    format!(
        "{}/cdd/config",
        var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", var("HOME").unwrap()))
    )
});

fn load_cd(name: String) -> Result<String, String> {
    let name = name + "=";
    let lines = read_to_string(FILE_PATH.as_str()).map_err(|err| err.to_string())?;
    Ok(lines
        .split('\n')
        .find(|line| line.starts_with(&name))
        .ok_or_else(|| "")?
        .replace(&name, "")
        .to_owned())
}

fn save_cd(name: String, path: Option<String>) {
    let path = path.unwrap_or(current_dir().unwrap().to_str().unwrap().to_owned());
    let mut opener = OpenOptions::new();
    let mut file = opener
        .append(true)
        .read(true)
        .create(true)
        .open(FILE_PATH.as_str())
        .unwrap();
    if read_to_string(FILE_PATH.as_str())
        .unwrap()
        .split('\n')
        .any(|line| line.starts_with(&name))
    {
        return;
    }
    file.write(name.as_bytes())
        .expect("Failed to write the shortcut name");
    file.write("=".as_bytes()).expect("Failed to write the '='");
    file.write(format!("{path}\n").as_bytes())
        .expect("Failed to write the path");
}

fn get_all() -> Vec<String> {
    read_to_string(FILE_PATH.as_str())
        .unwrap()
        .split('\n')
        .map(ToString::to_string)
        .collect::<Vec<String>>()
}

fn get_name(path: &String) -> Result<String, String> {
    let path = format!("={}", path);
    let lines = read_to_string(FILE_PATH.as_str()).map_err(|err| err.to_string())?;
    Ok(lines
        .split('\n')
        .find(|line| line.ends_with(&path))
        .ok_or_else(|| "Doesn't exists")?
        .replace(&path, "")
        .to_owned())
}

#[derive(Clone)]
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}

impl Node {
    fn insert(&mut self, node: Node) {
        match &mut self.next {
            None => self.next = Some(Box::new(node)),
            Some(n) => n.insert(node),
        }
    }

    fn delete_next(&mut self) {
        let next = self.next.as_ref().unwrap();
        match &next.next {
            Some(n) => {
                self.next = Some(n.clone());
            }
            None => (),
        }
    }
}

fn main() {
    let mut head = Node {
        value: 0,
        next: None,
    };
    head.insert(Node {
        value: 1,
        next: None,
    });
    head.delete_next();

    OpenOptions::new()
        .append(true)
        .create(true)
        .open(FILE_PATH.as_str())
        .expect("Couldn't load the config file");
    let content = Cli::parse();

    if content.save.is_some() {
        let content = content.save.unwrap();
        if content.contains("/") {
            panic!("cdd aliases cannot contain the '/' character (tried to save {content})")
        }
        save_cd(
            content,
            Some(current_dir().unwrap().to_str().unwrap().to_owned()),
        );
        print!("");
    } else if content.into.is_some() {
        let content = content.into.unwrap();
        if let Some((dir, rest)) = content.split_once("/") {
            print!("@ {}/{}", load_cd(dir.to_owned()).unwrap(), rest.to_owned());
        } else {
            let dir = load_cd(content).unwrap();
            print!("@ {}", dir);
        }
    } else if let Some(removed) = content.remove {
        let mut lines = get_all();
        let removed = removed + "=";
        lines.retain(|line| !line.starts_with(&removed) && !line.is_empty());
        let mut file = OpenOptions::new()
            .write(true)
            .open(FILE_PATH.as_str())
            .unwrap();
        file.set_len(0).unwrap();
        lines.iter().for_each(|line| {
            let line = line.to_string() + "\n";
            file.write(line.as_bytes()).unwrap();
        });
        print!("");
    } else if content.list {
        get_all()
            .into_iter()
            .for_each(|line| print!("{};", line.replace("=", " = ")))
    } else if content.exists {
        let path = current_dir().unwrap().to_str().unwrap().to_owned();
        print!("{}: {}", get_name(&path).unwrap(), &path)
    } else {
        print!("@ {}", content.dir.unwrap());
    }
}
