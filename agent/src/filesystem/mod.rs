use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

//The main structure for working with the directory, this structure acts as a root, you can use CRUD methods on this structure 
#[derive(Debug, Clone)]
pub struct Dir {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<File>,
    pub subdirs: Vec<Dir>,
    pub ignore: Vec<String>,
    pub ignore_size: Option<u64>,
}

///Mark for operation which file, 
///You can get information about the file structure using this structure in code, there are also additional functions for output 
#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
    pub data_line: Vec<Line>,
    pub parent_dir: Option<PathBuf>,
    #[allow(dead_code)]
    pub size: u64,
}

///Contains information about the term, we unload the lines into memory using this structure, yes, this is not very good in relation to memory, but we always have quick access to the content
#[derive(Debug, Clone)]
pub struct Line {
    pub number: usize,
    pub data: String,
}

impl Dir {
    pub fn pretty_print(&self) -> String {
        fn helper(dir: &Dir, prefix: &str, is_last: bool) -> String {
            let mut result = String::new();

            let connector = if prefix.is_empty() {
                ""
            } else if is_last {
                "└─ "
            } else {
                "├─ "
            };
            result.push_str(&format!("{}{}{}\n", prefix, connector, dir.name));

            let new_prefix = if prefix.is_empty() {
                String::new()
            } else if is_last {
                format!("{}   ", prefix)
            } else {
                format!("{}│  ", prefix)
            };

            let total = dir.subdirs.len() + dir.files.len();
            let mut index = 0;

            for subdir in &dir.subdirs {
                index += 1;
                let last_child = index == total;
                result.push_str(&helper(subdir, &new_prefix, last_child));
            }

            for file in &dir.files {
                index += 1;
                let last_child = index == total;
                let file_connector = if last_child { "└─ " } else { "├─ " };
                result.push_str(&format!("{}{}{}\n", new_prefix, file_connector, file.name));
            }

            result
        }

        let mut output = String::from("Filesystem:\n");
        output.push_str(&helper(self, "", true));
        output
    }

    pub fn read_from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::read_from_path_with_options(path, Vec::new(), None)
    }
    
    ///We can ignore folders or files with garbage specified in the config, you can also specify the size for reading large files in read_from_path_with_options(Legasi code)
    #[allow(dead_code)]
    pub fn read_from_path_with_ignore<P: AsRef<Path>>(
        path: P,
        ignore: Vec<String>,
    ) -> io::Result<Self> {
        Self::read_from_path_with_options(path, ignore, None)
    }

    pub fn read_from_path_with_options<P: AsRef<Path>>(
        path: P,
        ignore: Vec<String>,
        ignore_size: Option<u64>,
    ) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let name = path_ref
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_ref.display().to_string());

        let mut dir = Dir {
            name,
            path: path_ref.to_path_buf(),
            files: Vec::new(),
            subdirs: Vec::new(),
            ignore: ignore.clone(),
            ignore_size,
        };

        for entry in fs::read_dir(path_ref)? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if ignore.iter().any(|pattern| entry_name.contains(pattern)) {
                continue;
            }

            if entry_path.is_dir() {
                let subdir =
                    Dir::read_from_path_with_options(&entry_path, ignore.clone(), ignore_size)?;
                dir.subdirs.push(subdir);
            } else if entry_path.is_file() {
                if let Some(max_size) = ignore_size {
                    if let Ok(metadata) = fs::metadata(&entry_path) {
                        if metadata.len() > max_size {
                            continue;
                        }
                    }
                }

                let file = File::read_from_path_with_parent(&entry_path, &dir.path)?;
                dir.files.push(file);
            }
        }

        Ok(dir)
    }

    pub fn create_dir(&mut self, name: &str) -> io::Result<()> {
        let new_path = self.path.join(name);
        fs::create_dir_all(&new_path)?;
        *self = Dir::read_from_path_with_options(&self.path, self.ignore.clone(), self.ignore_size)?;
        Ok(())
    }

    /// Creates a new file and refreshes the data in the Dir structure.
    pub fn create_file(&mut self, name: &str, content: Option<&str>) -> io::Result<()> {
        let new_file = self.path.join(name);
        let mut file = fs::File::create(&new_file)?;
        if let Some(text) = content {
            writeln!(file, "{}", text)?;
        }
        *self = Dir::read_from_path_with_options(&self.path, self.ignore.clone(), self.ignore_size)?;
        Ok(())
    }

    ///Synchronize the file system for a single file, using its full path.
    pub fn refresh_file(&mut self, file_path: &Path) -> io::Result<()> {
        //Find file in current dir patch 
        if let Some(pos) = self.files.iter().position(|f| f.path == file_path) {
            let new_file = File::read_from_path_with_parent(file_path, &self.path)?;
            self.files[pos] = new_file;
            return Ok(());
        }

        // Recursive find in subdir
        for subdir in &mut self.subdirs {
            if file_path.starts_with(&subdir.path) {
                return subdir.refresh_file(file_path);
            }
        }

        Ok(())
    }

    /// Set ignore item
    #[allow(dead_code)]
    pub fn set_ignore(&mut self, ignore: Vec<String>) -> io::Result<()> {
        self.ignore = ignore;
        *self = Dir::read_from_path_with_options(&self.path, self.ignore.clone(), self.ignore_size)?;
        Ok(())
    }

    /// Add new ignore item(Legasi)
    #[allow(dead_code)]
    pub fn add_ignore(&mut self, pattern: String) -> io::Result<()> {
        if !self.ignore.contains(&pattern) {
            self.ignore.push(pattern);
            *self = Dir::read_from_path_with_options(&self.path, self.ignore.clone(), self.ignore_size)?;
        }
        Ok(())
    }

    /// 
    pub fn set_ignore_size(&mut self, max_size: Option<u64>) -> io::Result<()> {
        self.ignore_size = max_size;
        *self = Dir::read_from_path_with_options(&self.path, self.ignore.clone(), self.ignore_size)?;
        Ok(())
    }

    /// Возвращает общее количество файлов (включая вложенные)
    #[allow(dead_code)]
    pub fn total_files_count(&self) -> usize {
        let mut count = self.files.len();
        for subdir in &self.subdirs {
            count += subdir.total_files_count();
        }
        count
    }

    /// Возвращает общий размер всех файлов в директории
    #[allow(dead_code)]
    pub fn total_size(&self) -> u64 {
        let mut size = self.files.iter().map(|f| f.size).sum::<u64>();
        for subdir in &self.subdirs {
            size += subdir.total_size();
        }
        size
    }
}

impl File {
    /// We read the file and set the root folder to data
    pub fn read_from_path_with_parent<P: AsRef<Path>>(
        path: P,
        parent_dir: &Path,
    ) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let name = path_ref
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let size = fs::metadata(path_ref)?.len();

        let file = fs::File::open(path_ref)?;
        let reader = io::BufReader::new(file);

        let data_line = reader
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                line.ok().map(|l| Line {
                    number: i + 1,
                    data: l,
                })
            })
            .collect();

        Ok(File {
            name,
            path: path_ref.to_path_buf(),
            data_line,
            parent_dir: Some(parent_dir.to_path_buf()),
            size,
        })
    }

    /// Read file from disck
    pub fn reload(&mut self) -> io::Result<()> {
        if let Some(parent) = &self.parent_dir {
            let updated = File::read_from_path_with_parent(&self.path, parent)?;
            *self = updated;
        }
        Ok(())
    }

    /// Изменяет строку по номеру и сохраняет файл
    /// Если строки не существует, создаёт её
    pub fn edit_line(&mut self, line_number: usize, new_text: &str) -> io::Result<()> {
        // Если файл пустой или строка больше текущего размера - добавляем недостающие строки
        while self.data_line.len() < line_number {
            self.data_line.push(Line {
                number: self.data_line.len() + 1,
                data: String::new(),
            });
        }
        
        // Теперь редактируем нужную строку
        if let Some(line) = self.data_line.iter_mut().find(|l| l.number == line_number) {
            line.data = new_text.to_string();
        } else {
            // Этот случай не должен произойти после цикла выше, но на всякий случай
            self.data_line.push(Line {
                number: line_number,
                data: new_text.to_string(),
            });
        }
        
        self.save()?;
        self.reload()?;
        Ok(())
    }

    /// Добавляет новую строку в конец файла
    #[allow(dead_code)]
    pub fn add_line(&mut self, new_text: &str) -> io::Result<()> {
        let next_number = self.data_line.len() + 1;
        self.data_line.push(Line {
            number: next_number,
            data: new_text.to_string(),
        });
        self.save()?;
        self.reload()?;
        Ok(())
    }

    /// Сохраняет текущее состояние файла на диск
    pub fn save(&self) -> io::Result<()> {
        let mut file = fs::File::create(&self.path)?;
        for line in &self.data_line {
            writeln!(file, "{}", line.data)?;
        }
        
        // Обновляем размер после сохранения
        Ok(())
    }

    /// Возвращает размер файла в удобочитаемом формате
    #[allow(dead_code)]
    pub fn size_formatted(&self) -> String {
        let size = self.size as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.2} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

impl File {
    /// Add new line which shift next line bottom 
    pub fn insert_line(&mut self, line_number: usize, new_text: &str) -> io::Result<()> {
        if line_number == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Line numbers start from 1",
            ));
        }

        let new_line = Line {
            number: line_number,
            data: new_text.to_string(),
        };

        // Add empty line if we not have plase for add number line
        while self.data_line.len() < line_number - 1 {
            self.data_line.push(Line {
                number: self.data_line.len() + 1,
                data: String::new(),
            });
        }

        // Insert new line which shift 
        if line_number > self.data_line.len() {
            self.data_line.push(new_line);
        } else {
            self.data_line.insert(line_number - 1, new_line);
        }

        // Numbering all Line in file for context 
        self.renumber_lines();
        self.save()?;
        self.reload()?;
        Ok(())
    }

    /// Delete string for line number 
    pub fn delete_line(&mut self, line_number: usize) -> io::Result<()> {
        if line_number == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Line numbers start from 1",
            ));
        }

        if line_number > self.data_line.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Line {} does not exist", line_number),
            ));
        }

        self.data_line.remove(line_number - 1);
        
        // Numbering Line
        self.renumber_lines();
        self.save()?;
        self.reload()?;
        Ok(())
    }

    fn renumber_lines(&mut self) {
        for (i, line) in self.data_line.iter_mut().enumerate() {
            line.number = i + 1;
        }
    }
}