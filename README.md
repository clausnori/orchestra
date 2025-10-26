
# 🎼 Orchestra — AI-Driven Project Agent System

[![Demo Video](https://img.shields.io/badge/▶️-Watch%20Demo-red?style=for-the-badge&logo=youtube)](https://youtu.be/e0F3OJVcOYY?si=msm_tBZmtwKSk9c6)

Orchestra is an AI-powered command-line system written in Rust that manages projects, agents, and task automation through an interactive shell.
It's designed to simulate a collaborative workspace, where **agents** (AI-powered entities) read project structures, interpret tasks, and execute them intelligently.

---

## 🧠 Core Concept

In Orchestra, every task is given to an **AI Agent** — an autonomous entity capable of:

- 📖 Reading and analyzing project files
- 🧩 Understanding textual tasks
- ✍️ Generating or editing code and data
- 📊 Reporting progress back to the user

Each agent has its own context, memory, and workspace, all defined via the `orc.toml` configuration.

---

## 🚀 Features

- 🤖 **AI Integration** — agents interpret and execute tasks dynamically
- 🗂️ **Project Loader** — reads directory structures with ignore rules and size limits
- 💬 **Interactive Shell** — command-based interface for human-AI collaboration
- 👩‍💼 **Agents & Managers** — simulate multi-agent collaboration with hierarchy
- 🧩 **Custom DSL Scripts** — agents can load and execute structured script files
- 🎨 **Colorized Output** — intuitive, colorful CLI using ANSI codes

---

## 🧰 Commands

| Command   | Description                        |
|-----------|------------------------------------|
| `help`    | Show help menu                     |
| `task`    | Assign a new task to an AI agent   |
| `emp`     | List all employee agents           |
| `manager` | List all manager agents            |
| `ls`      | Show project directory structure   |
| `exit`    | Exit the program                   |

---

## ⚙️ Configuration (orc.toml)

Example configuration file:

```toml
[project]
dir = "./project"
ignore_dir = ["target", "node_modules"]
max_size = 10240

[[employee]]
name = "Alex"
dir = "./agents/employee/alex"
task = "AI Developer"

[[employee]]
name = "Emma"
dir = "./agents/employee/emma"
task = "Code Reviewer"

[[manager]]
name = "Liam"
level = "senior"
dir = "./agents/manager/liam"
team = ["Alex", "Emma"]
```

### 🧑‍💻 Example Workflow

```bash
$ cargo run
***** ORCHESTRA ***** 
Hi in Orchestra 
Dev: claus0nori@gmail.com

> task
Who will work on this task?
> Alex
Describe the task for Alex:
> Analyze project and refactor utils.rs
*** Load project in memory ***
Starting AI agent...
[AI] Alex: Reading source files...
[AI] Alex: Suggesting improvements in utils.rs
CMD: task
```

### 💡 Developer Info

**Author:** claus0nori  
**Email:** claus0nori@gmail.com  
**Language:** Rust  
**License:** MIT

---

## 📥 Installation

```bash
# Clone the repository
git clone ....
cd orchestra
cd agent

# Build the project
cargo build --release

# Run
export OPENAI_API_KEY="sp..."
cargo run
```

---

## 🔧 Requirements

- Rust 1.70+
- API key for AI provider (OpenAI, soon Anthropic)
- Configuration file `orc.toml`

---

## 🤝 Contributing

Contributions, suggestions, and pull requests are welcome! Feel free to open an issue to discuss new features or report bugs.

---

## 📜 License

This project is licensed under the MIT License. See the `LICENSE` file for details.
