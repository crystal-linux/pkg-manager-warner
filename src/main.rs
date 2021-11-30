use toml::{Value, toml};
use std::{fs, env};
use std::io::Write;

pub fn help() {
    println!("Usage:
    -h, help: Show this help message
    -v, version: Show version
    -i, init: Initialize the database
    -c, create-script: create the scripts for the warnings
    -w <package manager>, warning <package manager>: Show the warning for specified package manager
    ");
}

pub fn create_database() {
    let homepath = env::var("HOME").unwrap();
    let file = "/usr/share/pkg_warner/pkg_mngrs.db".to_string();
    if !std::path::Path::new(&"/usr/share/pkg_warner/").is_dir() {
        let _cdar = fs::create_dir_all("/usr/share/pkg_warner/".to_string());
        match _cdar {
            Ok(_) => {
                println!("Created path for database (previously missing)");
            }
            Err(_) => {
                println!("Couldn't create path for database (/usr/share/pkg_warner)")
            }
        }
    }
    let connection = sqlite::open(file).unwrap();
    let result = connection
        .execute(
            "
        CREATE TABLE pkg_mngrs (mngr TEXT, distro TEXT);
        ",
        );
    match result {
        Ok(_) => {
            println!("Created database");
        }
        Err(_) => {
            println!("Couldn't create database");
        }
    }
}

pub fn add_mngrs(pkg_managers: Vec<Vec<String>>, proper_manager: String) {
    let connection = sqlite::open("/usr/share/pkg_warner/pkg_mngrs.db".to_string()).unwrap();
    for entry in pkg_managers {
        println!("Don't use {}! {} is used on {}, here you use {}!", entry[0], entry[0], entry[1], proper_manager);
        let result = connection.execute(format!(
            "
            INSERT INTO pkg_mngrs (mngr, distro) VALUES (\"{}\", \"{}\");
            ", entry[0], entry[1]
        ));
        match result {
            Ok(_) => {
                println!("Added {} to database", entry[0]);
            }
            Err(_) => {
                println!("Couldn't add {} to database", entry[0]);
            }
        }
    }
}

pub fn create_script() {
    let connection = sqlite::open("/usr/share/pkg_warner/pkg_mngrs.db").unwrap();
    let result = connection.iterate(
        format!("SELECT mngr FROM pkg_mngrs WHERE mngr IS NOT \"proper_manager\";"),
        |pairs| {
            for &(_column, value) in pairs.iter() {
                writeln!(&mut fs::File::create(format!("/usr/bin/{}",value.unwrap())).unwrap(), "#!/usr/bin/env bash\n pkg-warner -w {}", value.unwrap()).unwrap();
            }
            true
        },
    );
    match result {
        Ok(_) => {}
        Err(_) => println!("Couldn't get value from database"),
    }

}

pub fn warn(proper_manager: String, package_manager: String) {
    let connection = sqlite::open("/usr/share/pkg_warner/pkg_mngrs.db".to_string()).unwrap();
    let mut warned = false;
    let result = connection.iterate(
        format!("SELECT distro FROM pkg_mngrs WHERE mngr = \"{}\";", package_manager),
        |pairs| {
            for &(_column, value) in pairs.iter() {
                if !warned {
                    println!("{} is used on {}! Please use {} instead!", package_manager, value.unwrap(), proper_manager);
                    warned = true;
                }
            }
            true
        },
    );
    match result {
        Ok(_) => {}
        Err(_) => println!("Couldn't get value from database"),
    }
}

fn main() {

    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        help();
        return;
    }

    let oper = &args[0];

    if !nix::unistd::Uid::effective().is_root() && oper != "-w" {
        println!("You need to be root to run this program");
        return;
    }

    let file = "/usr/share/pkg_warner/pkg_mngrs.db".to_string();
    if !std::path::Path::new(&file).exists() {
        create_database();
    }

    let file = format!("/etc/package_managers.toml");
    let mut database = String::new();
    database = fs::read_to_string(file).expect("Unable to read file");
    let db_parsed = database.parse::<Value>().expect("Unable to parse database");
    let mut pkg_managers: Vec<Vec<String>> = Vec::new();
    for entry in db_parsed.as_table() {
        for (key, value) in &*entry {
            let mut tempvec = Vec::new();
            tempvec.push(key.to_string());
            tempvec.push(value.to_string().replace("distro = ", "").replace("\n","").replace("\"",""));
            pkg_managers.push(tempvec);
        }
    }
    
    let connection = sqlite::open("/usr/share/pkg_warner/pkg_mngrs.db").unwrap();
    let mut proper_manager = String::new();
    let mut found = false;
    let result = connection.iterate(
        format!("SELECT distro FROM pkg_mngrs WHERE mngr = \"proper_manager\";"),
        |pairs| {
            for &(_column, value) in pairs.iter() {
                if !found {
                    proper_manager.push_str(value.unwrap());
                    found = true;
                }
            }
            true
        },
    );
    
    match oper.as_str() {
        "-i" | "init" => {
            create_database();
            add_mngrs(pkg_managers, proper_manager);
            create_script();
        }
        "-a" | "add" => {
            add_mngrs(pkg_managers, proper_manager);
        }
        "-c" | "create-script" => {
            create_script();
        }
        "-w" | "warning" => {
            warn(proper_manager, args[1].to_string());
        }
        _ => {
            help();
        }
    }
}