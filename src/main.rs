use toml::{Value};
use std::{fs, env};
use std::io::Write;
use std::process::Command;

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
        CREATE TABLE pkg_mngrs (mngr TEXT, distro TEXT, UNIQUE (mngr));
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
    let result = connection.execute(format!(
        "INSERT INTO pkg_mngrs (mngr,distro) VALUES (\"{}\",\"{}\")",
    "proper_manager", proper_manager));
    match result {
        Ok(_) => {
            println!("Added {} to database", proper_manager);
        }
        Err(_) => {
            println!("Couldn't add {} to database, maybe it already exists?", proper_manager);
        }
    }
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
                println!("Couldn't add {} to database, maybe it already exists?", entry[0]);
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
                Command::new("chmod")
                    .arg("+x")
                    .arg(format!("/usr/bin/{}", value.unwrap()))
                    .status()
                    .expect("Failed to chmod script");
            }
            true
        },
    );
    match result {
        Ok(_) => {}
        Err(_) => println!("Couldn't get value from database"),
    }

}

pub fn dump_database() -> Vec<String> {
    let connection = sqlite::open("/usr/share/pkg_warner/pkg_mngrs.db").unwrap();
    let mut dump = Vec::new();
    let result = connection.iterate(
        format!("SELECT mngr FROM pkg_mngrs WHERE mngr IS NOT \"proper_manager\";"),
        |pairs| {
            for &(_column, value) in pairs.iter() {
                dump.push(value.unwrap().to_string());
            }
            true
        },
    );
    match result {
        Ok(_) => {}
        Err(_) => println!("Couldn't get value from database"),
    }
    return dump;
}

pub fn rem_mngr(mngrs_to_remove: Vec<String>) {
    let connection = sqlite::open("/usr/share/pkg_warner/pkg_mngrs.db").unwrap();
    for mngr in mngrs_to_remove {
        let result = fs::remove_file(format!("/usr/bin/{}", mngr));
        match result {
            Ok(_) => {
                println!("Removed {}", mngr);
            }
            Err(_) => {
                println!("Couldn't remove {}", mngr);
            }
        }
        let result = connection.execute(format!(
            "DELETE FROM pkg_mngrs WHERE mngr = \"{}\"", mngr));
        match result {
            Ok(_) => {
                println!("Removed {} from database", mngr);
            }
            Err(_) => {
                println!("Couldn't remove {} from database", mngr);
            }
        }
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
    let database = fs::read_to_string(file).expect("Unable to read file");
    let db_parsed = database.parse::<Value>().expect("Unable to parse database");
    let mut pkg_managers: Vec<Vec<String>> = Vec::new();
    let proper_manager = db_parsed["proper_manager"].as_str().unwrap().to_string();
    for entry in db_parsed.as_table() {
        for (key, value) in &*entry {
            let mut tempvec = Vec::new();
            tempvec.push(key.to_string());
            tempvec.push(value.to_string().replace("distro = ", "").replace("\n","").replace("\"",""));
            if !tempvec.contains(&proper_manager) {
                pkg_managers.push(tempvec);
            }
            
        }
    }
    
    let dat_mgrs = dump_database();
    let mut pkgs_to_remove: Vec<String> = Vec::new();
    for i in dat_mgrs {
        let mut in_conf = false;
        for managers in &pkg_managers {
            if managers.contains(&&i) {
                in_conf = true;
            }
        }
        if !in_conf {
            pkgs_to_remove.push(i);
        }
    }

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
        "-r" | "remove" => {
            if !pkgs_to_remove.is_empty() {
                println!("Removing {} from database", pkgs_to_remove.join(", "));
                rem_mngr(pkgs_to_remove);
            }
        }
        _ => {
            help();
        }
    }
}