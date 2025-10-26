use regex::Regex;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::filesystem::{Dir, File};

#[derive(Debug, Clone)]
pub enum CommandType {
    Comments,
    Create,
    Open,
    Edit,
    Delete,
    Insert,
    Callback,
    Run,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum CreateType {
    File,
    Dir,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub command_type: CommandType,
    pub body: String,
    pub create_type: Option<CreateType>,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub content: Option<String>,
    pub subcommands: Vec<Command>,
}

/// List of executed commands 
#[derive(Debug, Clone)]
pub enum CMD {
    Comments(String),
    CreateDir(String),
    CreateFile(String),
    OpenDir { path: String, content: String },
    OpenFile { path: String, content: String },
    EditFile { path: String, line: usize, content: String },
    InsertFile { path: String, line: usize, content: String },
    DeleteFile { path: String, line: usize },
    Callback(String),
    Run { command: String, output: String, exit_code: i32 },
    Unknown(String),
}

/// Status which history 
#[derive(Debug)]
pub struct Status {
    pub running: Vec<CMD>,
}

impl Status {
    pub fn new() -> Self {
        Status { running: vec![] }
    }

    pub fn add(&mut self, cmd: CMD) {
        self.running.push(cmd);
    }
}

/// Parse one line DLS
pub fn parse_command(line: &str) -> Command {
    let trimmed = line.trim();

    let re_comments = Regex::new(r#"^COMMENTS\s+"(.*)""#).unwrap();
    if let Some(caps) = re_comments.captures(trimmed) {
        return Command {
            command_type: CommandType::Comments,
            body: caps[1].to_string(),
            create_type: None,
            file: None,
            line: None,
            content: None,
            subcommands: vec![],
        };
    }

    let re_create = Regex::new(r#"^CREATE\s+(DIR|FILE)\s+"(.*)""#).unwrap();
    if let Some(caps) = re_create.captures(trimmed) {
        let create_type = match &caps[1] {
            "DIR" => CreateType::Dir,
            "FILE" => CreateType::File,
            _ => unreachable!(),
        };
        return Command {
            command_type: CommandType::Create,
            body: trimmed.to_string(),
            create_type: Some(create_type),
            file: Some(caps[2].to_string()),
            line: None,
            content: None,
            subcommands: vec![],
        };
    }

    let re_open = Regex::new(r#"^OPEN\s+(DIR|FILE)\s+"(.*)""#).unwrap();
    if let Some(caps) = re_open.captures(trimmed) {
        let create_type = match &caps[1] {
            "DIR" => CreateType::Dir,
            "FILE" => CreateType::File,
            _ => unreachable!(),
        };
        return Command {
            command_type: CommandType::Open,
            body: trimmed.to_string(),
            create_type: Some(create_type),
            file: Some(caps[2].to_string()),
            line: None,
            content: None,
            subcommands: vec![],
        };
    }

    let re_insert = Regex::new(r#"^INSERT\s+FILE\s+"(.*)"\s+LINE\s+(\d+)\s+INSERT\s+"(.*)""#).unwrap();
    if let Some(caps) = re_insert.captures(trimmed) {
        return Command {
            command_type: CommandType::Insert,
            body: trimmed.to_string(),
            create_type: Some(CreateType::File),
            file: Some(caps[1].to_string()),
            line: Some(caps[2].parse().unwrap_or(0)),
            content: Some(caps[3].to_string()),
            subcommands: vec![],
        };
    }

    let re_delete = Regex::new(r#"^DELETE\s+FILE\s+"(.*)"\s+LINE\s+(\d+)"#).unwrap();
    if let Some(caps) = re_delete.captures(trimmed) {
        return Command {
            command_type: CommandType::Delete,
            body: trimmed.to_string(),
            create_type: Some(CreateType::File),
            file: Some(caps[1].to_string()),
            line: Some(caps[2].parse().unwrap_or(0)),
            content: None,
            subcommands: vec![],
        };
    }

    let re_edit = Regex::new(r#"^EDIT\s+(FILE|DIR)\s+"(.*)"\s+LINE\s+(\d+)\s+PUT\s+"(.*)""#).unwrap();
    if let Some(caps) = re_edit.captures(trimmed) {
        let create_type = match &caps[1] {
            "DIR" => CreateType::Dir,
            "FILE" => CreateType::File,
            _ => unreachable!(),
        };
        return Command {
            command_type: CommandType::Edit,
            body: trimmed.to_string(),
            create_type: Some(create_type),
            file: Some(caps[2].to_string()),
            line: Some(caps[3].parse().unwrap_or(0)),
            content: Some(caps[4].to_string()),
            subcommands: vec![],
        };
    }

    let re_run = Regex::new(r#"^RUN\s+"(.*)""#).unwrap();
    if let Some(caps) = re_run.captures(trimmed) {
        return Command {
            command_type: CommandType::Run,
            body: caps[1].to_string(),
            create_type: None,
            file: None,
            line: None,
            content: None,
            subcommands: vec![],
        };
    }

    let re_callback = Regex::new(r#"^CALLBACK\s+"(.*)""#).unwrap();
    if let Some(caps) = re_callback.captures(trimmed) {
        return Command {
            command_type: CommandType::Callback,
            body: caps[1].to_string(),
            create_type: None,
            file: None,
            line: None,
            content: None,
            subcommands: vec![],
        };
    }

    Command {
        command_type: CommandType::Unknown,
        body: trimmed.to_string(),
        create_type: None,
        file: None,
        line: None,
        content: None,
        subcommands: vec![],
    }
}

/// Parser for the entire script, goes through each line of code
pub fn parse_script(script: &str) -> Vec<Command> {
    script
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_command)
        .collect()
}

/// Executes code in projects; if the program runs for more than 3 minutes without problems, we return a response successfully. 
fn execute_shell_command(command: &str, working_dir: &Path) -> io::Result<(String, i32)> {
    use std::sync::mpsc::{channel, RecvTimeoutError};
    use std::thread;
    use std::time::Duration;

    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };
    
    let shell_arg = if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    };

    let command_owned = command.to_string();
    let working_dir_owned = working_dir.to_path_buf();

    let (tx, rx) = channel();

    thread::spawn(move || {
        let output = ProcessCommand::new(shell)
            .arg(shell_arg)
            .arg(&command_owned)
            .current_dir(&working_dir_owned)
            .output();

        let _ = tx.send(output);
    });

    // Wait 3 minutes to close the console. 
    match rx.recv_timeout(Duration::from_secs(180)) {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            let combined_output = if !stderr.is_empty() {
                format!("{}\n{}", stdout, stderr)
            } else {
                stdout
            };

            Ok((combined_output, exit_code))
        }
        Ok(Err(e)) => Err(e),
        Err(RecvTimeoutError::Timeout) => {
            // Timeout means everything is working correctly. 
            Ok((
                "Command is running in background (timeout after 3 minutes)".to_string(),
                0
            ))
        }
        Err(RecvTimeoutError::Disconnected) => {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Command execution thread disconnected"
            ))
        }
    }
}

/// Run the cmd, waiting for Calback 
pub fn parse_and_execute(dir: &mut Dir, script: &str) -> io::Result<Status> {
    let commands = parse_script(script);
    let mut status = Status::new();

    for cmd in commands {
        match cmd.command_type {
            CommandType::Comments => {
                println!("💬 {}", cmd.body);
                status.add(CMD::Comments(cmd.body.clone()));
            }

            CommandType::Create => match cmd.create_type {
                Some(CreateType::Dir) => {
                    //println!("📁 Creating dir: {:?}", cmd.file);
                    if let Some(name) = cmd.file.clone() {
                        dir.create_dir(&name)?;
                        status.add(CMD::CreateDir(name));
                    }
                }
                Some(CreateType::File) => {
                    //println!("📄 Creating file: {:?}", cmd.file);
                    if let Some(name) = cmd.file.clone() {
                        dir.create_file(&name, None)?;
                        status.add(CMD::CreateFile(name));
                    }
                }
                None => eprintln!("⚠️ CREATE missing type"),
            },

            CommandType::Open => match cmd.create_type {
                Some(CreateType::Dir) => {
                    //println!("📂 Opening dir: {:?}", cmd.file);
                    if let Some(path) = cmd.file.clone() {
                        let full_path = if Path::new(&path).is_absolute() {
                            PathBuf::from(&path)
                        } else {
                            dir.path.join(&path)
                        };
                        
                        let opened = Dir::read_from_path(&full_path)?;
                        let content = opened.pretty_print();
                        println!("{}", content);
                        status.add(CMD::OpenDir { 
                            path, 
                            content 
                        });
                    }
                }
                Some(CreateType::File) => {
                    //println!("📂 Opening file: {:?}", cmd.file);
                    if let Some(path) = cmd.file.clone() {
                        let full_path = if Path::new(&path).is_absolute() {
                            PathBuf::from(&path)
                        } else {
                            dir.path.join(&path)
                        };
                        
                        let file = File::read_from_path_with_parent(&full_path, &dir.path)?;
                        let mut content_lines = Vec::new();
                        for line in &file.data_line {
                            let line_str = format!("{} | {}", line.number, line.data);
                            println!("{}", line_str);
                            content_lines.push(line_str);
                        }
                        let content = content_lines.join("\n");
                        status.add(CMD::OpenFile { 
                            path, 
                            content 
                        });
                    }
                }
                None => eprintln!("⚠️ OPEN missing type"),
            },

            CommandType::Edit => match cmd.create_type {
                Some(CreateType::File) => {
                    if let (Some(path), Some(line), Some(content)) =
                        (cmd.file.clone(), cmd.line, cmd.content.clone())
                    {
                        let full_path = if Path::new(&path).is_absolute() {
                            PathBuf::from(&path)
                        } else {
                            dir.path.join(&path)
                        };
                        
                        //println!("📝 Editing file {:?} line {} => {}", full_path.display(), line, content);
                        let mut file = File::read_from_path_with_parent(&full_path, &dir.path)?;
                        file.edit_line(line, &content)?;
                        dir.refresh_file(&full_path)?;
                        status.add(CMD::EditFile { path, line, content });
                    }
                }
                _ => eprintln!("⚠️ EDIT only supports FILE"),
            },

            CommandType::Insert => match cmd.create_type {
                Some(CreateType::File) => {
                    if let (Some(path), Some(line), Some(content)) =
                        (cmd.file.clone(), cmd.line, cmd.content.clone())
                    {
                        let full_path = if Path::new(&path).is_absolute() {
                            PathBuf::from(&path)
                        } else {
                            dir.path.join(&path)
                        };
                        
                        //println!("➕ Inserting into file {:?} at line {} => {}", full_path.display(), line, content);
                        let mut file = File::read_from_path_with_parent(&full_path, &dir.path)?;
                        file.insert_line(line, &content)?;
                        dir.refresh_file(&full_path)?;
                        status.add(CMD::InsertFile { path, line, content });
                    }
                }
                _ => eprintln!("⚠️ INSERT only supports FILE"),
            },

            CommandType::Delete => match cmd.create_type {
                Some(CreateType::File) => {
                    if let (Some(path), Some(line)) = (cmd.file.clone(), cmd.line) {
                        let full_path = if Path::new(&path).is_absolute() {
                            PathBuf::from(&path)
                        } else {
                            dir.path.join(&path)
                        };
                        
                        //println!("🗑️  Deleting line {} from file {:?}", line, full_path.display());
                        let mut file = File::read_from_path_with_parent(&full_path, &dir.path)?;
                        file.delete_line(line)?;
                        dir.refresh_file(&full_path)?;
                        status.add(CMD::DeleteFile { path, line });
                    }
                }
                _ => eprintln!("⚠️ DELETE only supports FILE"),
            },

            CommandType::Run => {
                let command = cmd.body.clone();
                println!("🚀 Running command: {}", command);
                
                match execute_shell_command(&command, &dir.path) {
                    Ok((output, exit_code)) => {
                        if !output.trim().is_empty() {
                            println!("📤 Output:\n{}", output);
                        }
                        if exit_code != 0 {
                            eprintln!("⚠️ Command exited with code: {}", exit_code);
                        }
                        status.add(CMD::Run { 
                            command, 
                            output, 
                            exit_code 
                        });
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to execute command: {}", e);
                        status.add(CMD::Run { 
                            command, 
                            output: format!("Error: {}", e), 
                            exit_code: -1 
                        });
                    }
                }
            }

            CommandType::Callback => {
                //println!("↩️ Callback triggered: {}", cmd.body);
                status.add(CMD::Callback(cmd.body.clone()));
                return Ok(status);
            }

            CommandType::Unknown => {
                eprintln!("❓ Unknown command: {}", cmd.body);
                status.add(CMD::Unknown(cmd.body.clone()));
            }
        }
    }

    Ok(status)
}