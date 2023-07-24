use clap::{arg, command, ArgAction};

use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;
use subprocess::Exec;

fn main() {
    let m: clap::ArgMatches = command!("n")
        .subcommands([
            command!("add")
                .about("Add new packages to dependencies")
                .args([
                    arg!(<package> "Package name"),
                    arg!(-'S' --"save" "Save installed packages to a package.json file as dependencies.")
                        .required(false),
                    arg!(-'D' --"save-dev" "Package will appear in your devDependencies.")
                        .required(false),
                    arg!(-'O' --"save-optional" "Install the specified packages as optionalDependencies.")
                        .required(false)
                        .action(ArgAction::SetTrue),
                    arg!(-'g' --"global" "Install a package globally.")
                        .required(false)
                        .action(ArgAction::SetTrue)
                ]),
            command!("install")
                .alias("i")
                .about("Used to install all dependencies for a project."),
            command!("clean-install")
                .alias("ci")
                .about("Like npm ci"),
            command!("run")
                .alias("r")
                .about("This runs an arbitrary command from a package's 'scripts' object.")
                .args([
                    arg!(<script> "package's 'scripts' object")
                ])
        ])
        .get_matches();

    let pkg_manager = read_package_manager();

    if let [p, v] = &pkg_manager[..2] {
        println!(
            "🥳 The current package manager being used is : '{}@{}' ",
            p, v
        );
        match m.subcommand() {
            Some(("add", add_matches)) => {
                let package = add_matches.get_one::<String>("package").unwrap();
                let save = add_matches.get_flag("save");
                let save_dev = add_matches.get_flag("save-dev");
                let save_optional = add_matches.get_flag("save-optional");
                let global = add_matches.get_flag("global");
                if save {
                    run_shell(format!("{} add --save {}", p, package))
                } else if save_dev {
                    run_shell(format!("{} add --save-dev {}", p, package));
                } else if save_optional {
                    run_shell(format!("{} add --save-optional {}", p, package))
                } else if global {
                    match p.as_str() {
                        "yarn" => run_shell(format!("yarn global add {}", package)),
                        _ => run_shell(format!("{} add --global {}", p, package)),
                    }
                } else {
                    run_shell(format!("{} add {}", p, package));
                }
            }
            Some(("install", _install_matches)) => run_shell(format!("{} install", p)),
            Some(("clean-install", _clean_install_matches)) => match p.as_str() {
                "npm" => run_shell(format!("npm ci")),
                _ => run_shell(format!("{} install --frozen-lockfile", p)),
            },
            Some(("run", run_matches)) => {
                let script = run_matches.get_one::<String>("script").unwrap();
                run_shell(format!("{} run {}", p, script));
            }
            _ => {
                println!("🙁 Sorry, the command you entered is currently not supported.")
            }
        }
    }
}

pub fn read_package_manager() -> Vec<String> {
    let path = Path::new("./package.json");
    if path.exists() {
        let contents = fs::read_to_string(path).unwrap();
        let parsed: Value = serde_json::from_str(&contents).unwrap();
        match parsed["packageManager"].as_str() {
            Some(manager) => {
                let re = Regex::new(r"(npm|pnpm|yarn)@(.*)").unwrap();
                if let Some(caps) = re.captures(manager) {
                    return vec![caps[1].to_string(), caps[2].to_string()];
                } else {
                    panic!("😢 PackageManager parsing failed, possibly due to incorrect format. ");
                }
            }
            None => {
                panic!("😢 Sorry, you must to be configure packageManager in package.json file ");
            }
        }
    }
    panic!("🔎 Could not found package.json");
}

fn run_shell(cmd: String) {
    println!("🎯 The instruction to be executed is : '{}' ", cmd);
    Exec::cmd("sh").arg("-c").arg(cmd).popen().unwrap();
}
