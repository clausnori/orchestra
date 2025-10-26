use std::{fs, io};
use std::io::Write;

mod filesystem;
mod config;
mod model;
mod coderun;
mod agent;

use toml;
use filesystem::Dir;
use config::Config;
use model::Promt;
use agent::Agent;

// Color ANSI
const RED: &str = "\x1b[31m";
const ORANGE: &str = "\x1b[91m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const BLUE: &str = "\x1b[34m";
const PURPLE: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let letters = [
        (RED, "O"),
        (ORANGE, "R"),
        (YELLOW, "C"),
        (GREEN, "H"),
        (CYAN, "E"),
        (BLUE, "S"),
        (PURPLE, "T"),
        (RED, "R"),
        (YELLOW, "A"),
    ];
    print!("{}*****{} ", BLUE, RESET);
    for (color, ch) in letters {
        print!("{}{}{}", color, ch, RESET);
    }
    println!(" {}*****{}", BLUE, RESET);
    println!("{}Hi in Orchestra{} \n Dev: claus0nori@gmail.com", GREEN, RESET);
    println!("{}task{} - create new task", YELLOW, RESET);
    println!("{}help{} - show help menu", YELLOW, RESET);
    println!("{}ls{} - show file in dir projects", YELLOW, RESET);
    println!("{}*in dev* emp{} - show current employment", YELLOW, RESET);
    println!("{}*in dev* manager{} - show Manager", YELLOW, RESET);
    println!("{}exit{} - close program", YELLOW, RESET);

    let content = fs::read_to_string("orc.toml")?;
    let config: Config = toml::from_str(&content).expect("Error parsing config");

    loop {
        println!("{}*** Load project in memory ***{}", BLUE, RESET);
        let mut dir = Dir::read_from_path_with_options(
            config.project.dir.clone(),
            config.project.ignore_dir.clone(),
            Some(config.project.max_size.clone())
        )?;

        print!("{}> {}", GREEN, RESET);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim();

        match trimmed {
            "exit" => break,
            "help" => help_menu(),
            "task" => { 
                println!("{}Who will work on this task?{}", YELLOW, RESET);
                print!("> ");
                io::stdout().flush().unwrap();
                let mut agent_name = String::new();
                io::stdin().read_line(&mut agent_name).unwrap();
                let agent_name = agent_name.trim().to_string();

                println!("{}Describe the task for {}:{}", YELLOW, agent_name, RESET);
                print!("> ");
                io::stdout().flush().unwrap();
                let mut task_msg = String::new();
                io::stdin().read_line(&mut task_msg).unwrap();
                let task_msg = task_msg.trim().to_string();

                load_project(&config, &mut dir, agent_name, task_msg)?;
            },
            "emp" => {
                println!("{}=== Employees ==={}", BLUE, RESET);
                for emp in &config.employee {
                    println!("{}{}{} -> {} ({})", GREEN, emp.name, RESET, emp.dir, emp.task);
                }
            },
            "manager" => {
                println!("\n{}=== Managers ==={}", BLUE, RESET);
                for mgr in &config.manager {
                    println!("{}{}{} [{}] -> {:?} ({})", GREEN, mgr.name, RESET, mgr.level, mgr.team, mgr.dir);
                }
            },
            "ls" => println!("{}", dir.pretty_print()),
            _ => continue,
        }

        println!("{}CMD:{} {}", YELLOW, RESET, trimmed);
    }

    println!("{}See you later (: {}", GREEN, RESET);
    Ok(())
}

fn help_menu() {
    println!("{}Commands:{}", BLUE, RESET);
    println!("{}exit{} - quit program", YELLOW, RESET);
    println!("{}help{} - show this menu", YELLOW, RESET);
    println!("{}task{} - create new task", YELLOW, RESET);
    println!("{}emp{} - show employees", YELLOW, RESET);
    println!("{}manager{} - show managers", YELLOW, RESET);
    println!("{}ls{} - list project directory", YELLOW, RESET);
}

fn load_project(config: &Config, dir: &mut Dir, agent_name: String, task_msg: String) -> std::io::Result<()> {

    let promt = Promt::new(agent_name, dir.clone(), config.employee.clone(), task_msg);

    let mut agent = Agent::new(promt);

    println!("{}Starting...{}", BLUE, RESET);
    agent.run(dir)?;

    Ok(())
}