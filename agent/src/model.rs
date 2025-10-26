use crate::filesystem::Dir;
use crate::config::Employee;

#[derive(Debug)]
pub struct Promt {
    pub message: Option<String>,
    pub system: Option<String>
}

impl Promt {
    pub fn new(curent_employee: String, dir: Dir, employee: Vec<Employee>, task_msg: String) -> Self {
        let c_employee: Vec<Employee> = employee
            .into_iter()
            .filter(|x| x.name == curent_employee)
            .collect();
        
        let file_system_messege: String = dir.pretty_print();
        
        let agen_info = format!(
            r#"=== Agent Info ===
Name: {0}
Role: {2}
LogDir: "{1}/log"
ReportDir: "{1}/report"
Comment Mark: [{0}] use syntax for language in request and if need in function and class
"#,
            c_employee[0].name,
            c_employee[0].dir,
            c_employee[0].task
        );
        
        let task = format!("=== Task ===\n{}\n", task_msg);
        
        let commands = r#"=== Orchestra Commands ===

📂 DIRECTORY:
  OPEN DIR "path"          - view directory structure
  CREATE DIR "path"        - create new directory

📄 FILE:
  OPEN FILE "path"         - view file content
  CREATE FILE "path"       - create new file
  DELETE FILE "path" LINE n - delete line from file

✏️ EDIT:
  EDIT FILE "path" LINE n PUT "text"     - replace line n
  INSERT FILE "path" LINE n INSERT "text" - insert at line n (shifts down)

🔧 EXECUTION:
  RUN "command"            - execute shell command

💬 META:
  COMMENTS "text"          - document current action
  CALLBACK "message"       - report completion/next step

=== Rules ===
1. ALWAYS use CALLBACK after read operations (OPEN)
2. ALWAYS use CALLBACK before write operations (EDIT, INSERT, DELETE, CREATE)
3. Mark code with comment [YourName]
4. Open files/dirs before editing
5. Be concise - no explanations, just actions

=== Example ===
COMMENTS "Opening project structure"
OPEN DIR "project"
CALLBACK "Viewed project, will open main.py"

COMMENTS "Reading main.py"
OPEN FILE "project/main.py"
CALLBACK "File has 5 lines, need to add import"

COMMENTS "Adding import statement"
INSERT FILE "project/main.py" LINE 1 INSERT "import math  #[Agent]"
CALLBACK "Import added, will add function"

COMMENTS "Creating calculate function"
EDIT FILE "project/main.py" LINE 3 PUT "def calculate(x):  //[Agent]"
INSERT FILE "project/main.py" LINE 4 INSERT "    return math.sqrt(x)  #[Agent]"
CALLBACK "Function ready, will test"

COMMENTS "Running tests"
RUN "python -m pytest tests/"
CALLBACK "Tests passed, task complete"
"#;

        let promt = format!(
            "{}\n{}\n{}",
            file_system_messege,
            agen_info,
            commands,
        );

        Promt {
            message: Some(task.trim().to_string()),
            system:Some(promt.trim().to_string())
        }
    }
}