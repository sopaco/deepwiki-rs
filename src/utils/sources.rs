use std::path::PathBuf;

use crate::{
    generator::preprocess::extractors::language_processors::LanguageProcessorManager,
    i18n::TargetLanguage,
    types::code::CodeInsight,
};

pub fn read_code_source(
    language_processor: &LanguageProcessorManager,
    project_path: &PathBuf,
    file_path: &PathBuf,
    target_language: &TargetLanguage,
    max_file_size: usize,
) -> String {
    // Build full file path
    let full_path = project_path.join(file_path);

    // Read source code with size limit to prevent OOM
    if let Ok(metadata) = std::fs::metadata(&full_path) {
        let file_size = metadata.len() as usize;
        let read_size = file_size.min(max_file_size);

        if let Ok(content) = read_file_with_limit(&full_path, read_size) {
            // If code is too long, intelligently truncate
            truncate_source_code(language_processor, &full_path, &content, max_file_size)
        } else {
            let msg = target_language.msg_cannot_read_file();
            msg.replace("{}", &full_path.display().to_string())
        }
    } else {
        let msg = target_language.msg_cannot_read_file();
        msg.replace("{}", &full_path.display().to_string())
    }
}

/// Read file with byte limit to prevent OOM
fn read_file_with_limit(path: &PathBuf, max_bytes: usize) -> std::io::Result<String> {
    use std::io::Read;

    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = vec![0u8; max_bytes];
    let bytes_read = reader.read(&mut buffer)?;

    buffer.truncate(bytes_read);
    String::from_utf8(buffer).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))
}

fn truncate_source_code(
    language_processor: &LanguageProcessorManager,
    file_path: &std::path::Path,
    content: &str,
    max_length: usize,
) -> String {
    if content.len() <= max_length {
        return content.to_string();
    }

    // Smart truncation: prioritize function definitions, struct definitions, and other important parts
    let lines: Vec<&str> = content.lines().collect();
    let mut result = String::new();
    let mut current_length = 0;
    let mut important_lines = Vec::new();
    let mut other_lines = Vec::new();

    // Classify lines: important and regular
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if language_processor.is_important_line(file_path, trimmed) {
            important_lines.push((i, line));
        } else {
            other_lines.push((i, line));
        }
    }

    // First add important lines
    for (_, line) in important_lines {
        if current_length + line.len() > max_length {
            break;
        }
        result.push_str(line);
        result.push('\n');
        current_length += line.len() + 1;
    }

    // Then add regular lines until length limit is reached
    for (_, line) in other_lines {
        if current_length + line.len() > max_length {
            break;
        }
        result.push_str(line);
        result.push('\n');
        current_length += line.len() + 1;
    }

    if current_length >= max_length {
        result.push_str("\n... (code truncated) ...\n");
    }

    result
}

pub fn read_dependency_code_source(
    language_processor: &LanguageProcessorManager,
    analysis: &CodeInsight,
    project_path: &PathBuf,
) -> String {
    let mut dependency_code = String::new();

    // Limit total length of dependency code
    let mut total_length = 0;
    const MAX_DEPENDENCY_CODE_LENGTH: usize = 4000;

    for dep_info in &analysis.dependencies {
        if total_length >= MAX_DEPENDENCY_CODE_LENGTH {
            dependency_code.push_str("\n... (more dependency code omitted) ...\n");
            break;
        }

        // Try to find dependency file
        if let Some(dep_path) =
            find_dependency_file(language_processor, project_path, &dep_info.name)
        {
            if let Ok(content) = std::fs::read_to_string(&dep_path) {
                let truncated =
                    truncate_source_code(language_processor, &dep_path, &content, 8_1024);
                dependency_code.push_str(&format!(
                    "\n### Dependency: {} ({})\n```\n{}\n```\n",
                    dep_info.name,
                    dep_path.display(),
                    truncated
                ));
                total_length += truncated.len();
            }
        }
    }

    if dependency_code.is_empty() {
        "No available dependency code".to_string()
    } else {
        dependency_code
    }
}

/// Todo: Use LanguageProcessorManager approach
fn find_dependency_file(
    _language_processor: &LanguageProcessorManager,
    project_path: &PathBuf,
    dep_name: &str,
) -> Option<std::path::PathBuf> {
    // Clean dependency name, remove path prefix
    let clean_name = dep_name
        .trim_start_matches("./")
        .trim_start_matches("../")
        .trim_start_matches("@/")
        .trim_start_matches("/");

    // Try various possible file paths
    let possible_paths = vec![
        // Rust
        format!("{}.rs", clean_name),
        format!("{}/mod.rs", clean_name),
        format!("src/{}.rs", clean_name),
        format!("src/{}/mod.rs", clean_name),
        // JavaScript/TypeScript
        format!("{}.js", clean_name),
        format!("{}.ts", clean_name),
        format!("{}.jsx", clean_name),
        format!("{}.tsx", clean_name),
        format!("{}.mjs", clean_name),
        format!("{}.cjs", clean_name),
        format!("{}/index.js", clean_name),
        format!("{}/index.ts", clean_name),
        format!("{}/index.jsx", clean_name),
        format!("{}/index.tsx", clean_name),
        format!("src/{}.js", clean_name),
        format!("src/{}.ts", clean_name),
        format!("src/{}.jsx", clean_name),
        format!("src/{}.tsx", clean_name),
        format!("src/{}/index.js", clean_name),
        format!("src/{}/index.ts", clean_name),
        // Vue
        format!("{}.vue", clean_name),
        format!("src/components/{}.vue", clean_name),
        format!("src/views/{}.vue", clean_name),
        format!("src/pages/{}.vue", clean_name),
        format!("components/{}.vue", clean_name),
        format!("views/{}.vue", clean_name),
        format!("pages/{}.vue", clean_name),
        // Svelte
        format!("{}.svelte", clean_name),
        format!("src/components/{}.svelte", clean_name),
        format!("src/routes/{}.svelte", clean_name),
        format!("src/lib/{}.svelte", clean_name),
        format!("components/{}.svelte", clean_name),
        format!("routes/{}.svelte", clean_name),
        format!("lib/{}.svelte", clean_name),
        // Kotlin
        format!("{}.kt", clean_name),
        format!("src/main/kotlin/{}.kt", clean_name),
        format!("src/main/java/{}.kt", clean_name),
        format!("app/src/main/kotlin/{}.kt", clean_name),
        format!("app/src/main/java/{}.kt", clean_name),
        // Python
        format!("{}.py", clean_name),
        format!("{}/__init__.py", clean_name),
        format!("src/{}.py", clean_name),
        format!("src/{}/__init__.py", clean_name),
        // Java
        format!("{}.java", clean_name),
        format!("src/main/java/{}.java", clean_name),
        format!("app/src/main/java/{}.java", clean_name),
        // C#
        format!("{}.cs", clean_name),
        format!("{}.csx", clean_name),
        format!("src/{}.cs", clean_name),
        format!("{}Program.cs", clean_name),
        format!("Controllers/{}.cs", clean_name),
        format!("Models/{}.cs", clean_name),
        format!("Services/{}.cs", clean_name),
        format!("Views/{}.cs", clean_name),
        format!("ViewModels/{}.cs", clean_name),
        format!("Helpers/{}.cs", clean_name),
        format!("Utils/{}.cs", clean_name),
        format!("Core/{}.cs", clean_name),
        format!("Data/{}.cs", clean_name),
        format!("Repositories/{}.cs", clean_name),
        format!("Interfaces/{}.cs", clean_name),
    ];

    for path_str in possible_paths {
        let full_path = project_path.join(&path_str);
        if full_path.exists() {
            return Some(full_path);
        }
    }

    // If direct path lookup fails, try recursive search
    recursive_find_file(project_path, clean_name)
}

fn recursive_find_file(project_path: &PathBuf, file_name: &str) -> Option<std::path::PathBuf> {
    use std::fs;

    // Define search extensions
    let extensions = vec![
        "rs", "py", "js", "ts", "jsx", "tsx", "vue", "svelte", "kt", "java", "mjs", "cjs",
        "cs", "csx",
    ];

    // Recursive search function
    fn search_directory(
        dir: &PathBuf,
        target_name: &str,
        extensions: &[&str],
    ) -> Option<std::path::PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    if let Some(file_name) = path.file_stem() {
                        if let Some(ext) = path.extension() {
                            if file_name.to_string_lossy() == target_name
                                && extensions.contains(&ext.to_string_lossy().as_ref())
                            {
                                return Some(path);
                            }
                        }
                    }
                } else if path.is_dir() {
                    // Skip common ignored directories
                    if let Some(dir_name) = path.file_name() {
                        let dir_name_str = dir_name.to_string_lossy();
                        if !dir_name_str.starts_with('.')
                            && dir_name_str != "node_modules"
                            && dir_name_str != "target"
                            && dir_name_str != "build"
                            && dir_name_str != "dist"
                        {
                            if let Some(found) = search_directory(&path, target_name, extensions) {
                                return Some(found);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    search_directory(project_path, file_name, &extensions)
}
