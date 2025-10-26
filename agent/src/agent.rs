use std::{error::Error, fs, io::Write};
use chrono::Local;
use crate::filesystem::Dir;
use crate::model::Promt;
use crate::coderun::parser::{parse_and_execute, CMD};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GptRequest {
    model: String,
    messages: Vec<GptMessage>,
}

#[derive(Serialize, Clone)]
struct GptMessage {
   pub role: String,
   pub content: String,
}

#[derive(Deserialize)]
struct GptResponse {
    choices: Vec<GptChoice>,
}

#[derive(Deserialize)]
struct GptChoice {
    message: GptChoiceMessage,
}

#[derive(Deserialize)]
struct GptChoiceMessage {
    content: String,
}

pub struct Agent {
    pub promt: Promt,
    pub current_script: usize,
    pub log_path: String,
    pub conversation_history: Vec<GptMessage>,
}

impl Agent {
    pub fn new(promt: Promt) -> Self {
        fs::create_dir_all("log").ok();

        let now = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let log_filename = format!("log/Agent_{}.log", now);

        let system_message = promt.system.clone().unwrap_or_else(|| 
            "You are an autonomous coding agent. Respond only with DSL commands. Not md format.".to_string()
        );

        let conversation_history = vec![
            GptMessage {
                role: "system".into(),
                content: system_message,
            },
        ];

        Self {
            promt,
            current_script: 1,
            log_path: log_filename,
            conversation_history,
        }
    }

    pub fn run(&mut self, dir: &mut Dir) -> std::io::Result<()> {
        self.log_event("🤖 Agent starting work...");

        let initial_prompt = self.promt.message.clone().unwrap_or_default();
        self.conversation_history.push(GptMessage {
            role: "user".into(),
            content: initial_prompt,
        });

        let mut script = match self.generate_script(None,None) {
            Ok(s) => {
                self.log_event(&format!("✅ Script #{} generated:\n{}", self.current_script, s));
                s
            }
            Err(err) => {
                self.log_event(&format!("❌ Failed to generate script: {}", err));
                return Err(std::io::Error::new(std::io::ErrorKind::Other, err.to_string()));
            }
        };

        loop {
            self.log_event(&format!("📜 Executing script #{}", self.current_script));

            let results = parse_and_execute(dir, &script)?;
            let mut callback_triggered = false;
            let mut callback_msg = String::new();

            let execution_feedback = self.collect_execution_feedback(&results.running);

            for cmd in results.running {
                if let CMD::Callback(message) = cmd {
                    callback_triggered = true;
                    callback_msg = message.clone();
                    self.log_event(&format!("💬 Callback: {}", message));
                }
            }

            if callback_triggered {
                self.current_script += 1;
                self.log_event(&format!("🔄 Requesting next script from GPT (#{})...", self.current_script));

                match self.generate_script(Some(&callback_msg), Some(&execution_feedback)) {
                    Ok(new_script) => {
                        self.log_event(&format!("✅ Received script #{}:\n{}", self.current_script, new_script));
                        script = new_script;
                        continue;
                    }
                    Err(err) => {
                        self.log_event(&format!("⚠️ GPT request failed: {}", err));
                        break;
                    }
                }
            } else {
                self.log_event("✅ No callback found, execution finished.");
                break;
            }
        }

        self.log_event("🏁 Agent finished successfully.");
        Ok(())
    }

    /// Собирает информацию из выполненных команд
    fn collect_execution_feedback(&self, commands: &[CMD]) -> String {
        let mut feedback = Vec::new();

        for cmd in commands {
            match cmd {
                CMD::Comments(text) => {
                    feedback.push(format!("💬 Comment: {}", text));
                }
                CMD::CreateDir(path) => {
                    feedback.push(format!("📁 Created directory: {}", path));
                }
                CMD::CreateFile(path) => {
                    feedback.push(format!("📄 Created file: {}", path));
                }
                CMD::OpenDir { path, content } => {
                    feedback.push(format!("📂 Opened directory: {}\n{}", path, content));
                }
                CMD::OpenFile { path, content } => {
                    feedback.push(format!("📄 Opened file: {}\n{}", path, content));
                }
                CMD::EditFile { path, line, content } => {
                    feedback.push(format!("✏️ Edited file: {} at line {} with: {}", path, line, content));
                }
                CMD::InsertFile { path, line, content } => {
                    feedback.push(format!("➕ Inserted into file: {} at line {}: {}", path, line, content));
                }
                CMD::DeleteFile { path, line } => {
                    feedback.push(format!("🗑️ Deleted line {} from file: {}", line, path));
                }
                CMD::Run { command, output, exit_code } => {
                    feedback.push(format!(
                        "🚀 Executed command: {}\n📤 Output (exit code {}):\n{}", 
                        command, exit_code, output
                    ));
                }
                CMD::Callback(message) => {
                    feedback.push(format!("↩️ Callback: {}", message));
                }
                CMD::Unknown(text) => {
                    feedback.push(format!("❓ Unknown command: {}", text));
                }
            }
        }

        if feedback.is_empty() {
            "No commands were executed.".to_string()
        } else {
            feedback.join("\n")
        }
    }

    fn generate_script(&mut self, callback: Option<&str>, execution_feedback: Option<&str>) -> Result<String, Box<dyn Error>> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .expect("⚠️ OPENAI_API_KEY environment variable not set");

        let client = Client::new();

        // Если есть callback, добавляем его в историю вместе с результатами выполнения
        if let Some(cb_msg) = callback {
            let feedback = execution_feedback.unwrap_or("No execution feedback available.");
            
            let user_message = format!(
                "=== Execution Results ===\n{}\n\n=== Callback ===\n{}\n\n\
                Continue the orchestration using same format.\n\
                Output only commands (no text, no explanations; if you want you can use COMMENTS, no md format).",
                feedback, cb_msg
            );
            
            self.conversation_history.push(GptMessage {
                role: "user".into(),
                content: user_message,
            });
            
            self.log_event(&format!("🧠 Sending GPT request (callback: {})", cb_msg));
        } else {
            self.log_event("🧠 Sending GPT request for first script...");
        }

        let request = GptRequest {
            model: "gpt-4o-mini".to_string(),
            messages: self.conversation_history.clone(),
        };

        let res = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&request)
            .send()?;

        if !res.status().is_success() {
            return Err(format!("Bad response from GPT: {}", res.status()).into());
        }

        let body: GptResponse = res.json()?;
        let script = body.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "COMMENTS \"No script generated\"".to_string());

        self.conversation_history.push(GptMessage {
            role: "assistant".into(),
            content: script.clone(),
        });

        Ok(script)
    }

    fn log_event(&self, message: &str) {
        let timestamp = Local::now().format("%H:%M:%S").to_string();
        let log_line = format!("[{}] {}\n", timestamp, message);

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = file.write_all(log_line.as_bytes());
        }
        //dbg message for dev , 
        //dbg!(message);
    }
}