use crate::generator::context::GeneratorContext;
use crate::generator::preprocess::agents::directory_scoring::DirectoryScorer;
use crate::types::project_structure::ProjectStructure;
use crate::types::{DirectoryInfo, FileInfo};
use crate::utils::file_utils::{is_binary_file_path, is_test_directory, is_test_file};
use anyhow::Result;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::fs::Metadata;
use std::path::PathBuf;
use std::process::Command;

/// Project structure extractor
pub struct StructureExtractor {
    directory_scorer: DirectoryScorer,
    context: GeneratorContext,
}

impl StructureExtractor {
    pub fn new(context: GeneratorContext) -> Self {
        Self {
            directory_scorer: DirectoryScorer::new(),
            context,
        }
    }

    /// Extract project structure
    pub async fn extract_structure(&self, project_path: &PathBuf) -> Result<ProjectStructure> {
        let cache_key = format!("structure_{}", project_path.display());

        // Execute structure extraction
        let structure = self.extract_structure_impl(project_path).await?;

        // Cache results, structure cache is only used for observation records
        self.context
            .cache_manager
            .write()
            .await
            .set("structure", &cache_key, &structure)
            .await?;

        Ok(structure)
    }

    async fn extract_structure_impl(&self, project_path: &PathBuf) -> Result<ProjectStructure> {
        let mut directories = Vec::new();
        let mut file_types = HashMap::new();
        let mut size_distribution = HashMap::new();

        // Get git tracked files for filtering (only if needed)
        let tracked_files = if self.context.config.git_tracked_only {
            self.get_tracked_files(project_path)
        } else {
            HashMap::new()
        };

        // Collect all files during scan
        let mut files = Vec::new();

        // Scan directory, extract internal directory and file structure and basic file information
        self.scan_directory(
            project_path,
            project_path,
            &mut directories,
            &mut files,
            &mut file_types,
            &mut size_distribution,
            &tracked_files,
            0,
            self.context.config.max_depth.into(),
        )
        .await?;

        // Sort directories lexicographically for deterministic ordering
        directories.sort_by(|a, b| a.path.cmp(&b.path));

        // Sort files by importance score
        files.sort_by(|a, b| {
            b.importance_score.partial_cmp(&a.importance_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        let total_files = files.len();
        let _ = total_files; // Used for ProjectStructure.total_files

        // Apply LLM directory scoring boost to all directories
        match self.directory_scorer.score_directories(&self.context, &directories).await {
            Ok(dir_scores) => {
                self.apply_directory_score_boost(&mut files, &dir_scores);
            }
            Err(e) => {
                eprintln!("⚠️  Directory scoring failed: {}, skipping", e);
            }
        }

        // Calculate importance scores (scores already calculated during scan, just refine)
        self.calculate_importance_scores(&mut files, &mut directories);

        let project_name = self.context.config.get_project_name();

        Ok(ProjectStructure {
            project_name,
            root_path: project_path.clone(),
            total_files: files.len(),
            total_directories: directories.len(),
            directories,
            files,
            file_types,
            size_distribution,
        })
    }

    /// Get files tracked by git as a HashSet of absolute paths
    fn get_tracked_files(&self, project_path: &PathBuf) -> HashMap<PathBuf, ()> {
        let mut tracked = HashMap::new();

        // Use git ls-files to get all tracked files
        if let Ok(output) = Command::new("git")
            .args(["ls-files"])
            .current_dir(project_path)
            .output()
        {
            if output.status.success() {
                let files = String::from_utf8_lossy(&output.stdout);
                for line in files.lines() {
                    let path = project_path.join(line);
                    tracked.insert(path, ());
                }
            } else {
                eprintln!("⚠️  Warning: git ls-files failed, git_tracked_only will be ignored");
            }
        } else {
            eprintln!("⚠️  Warning: Failed to run git ls-files, git_tracked_only will be ignored");
        }

        tracked
    }

    fn scan_directory<'a>(
        &'a self,
        current_path: &'a PathBuf,
        root_path: &'a PathBuf,
        directories: &'a mut Vec<DirectoryInfo>,
        files: &'a mut Vec<FileInfo>,
        file_types: &'a mut HashMap<String, usize>,
        size_distribution: &'a mut HashMap<String, usize>,
        tracked_files: &'a HashMap<PathBuf, ()>,
        current_depth: usize,
        max_depth: usize,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if current_depth > max_depth {
                return Ok(());
            }

            let mut entries = tokio::fs::read_dir(current_path).await?;
            let mut dir_file_count = 0;
            let mut dir_subdirectory_count = 0;
            let mut dir_total_size = 0;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let file_type = entry.file_type().await?;

                if file_type.is_file() {
                    // Check if this file should be ignored
                    if !self.should_ignore_file(&path, tracked_files) {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            let mut file_info = self.create_file_info(&path, root_path, &metadata)?;

                            // Calculate importance score during scan (lazy calculation)
                            self.calculate_file_importance_score(&mut file_info);

                            // Update statistics
                            if let Some(ext) = &file_info.extension {
                                *file_types.entry(ext.clone()).or_insert(0) += 1;
                            }

                            let size_category = self.categorize_file_size(file_info.size);
                            *size_distribution.entry(size_category).or_insert(0) += 1;

                            dir_file_count += 1;
                            dir_total_size += file_info.size;

                            files.push(file_info);
                        }
                    }
                } else if file_type.is_dir() {
                    let dir_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    // Skip hidden directories and commonly ignored directories
                    if !self.should_ignore_directory(&dir_name) {
                        dir_subdirectory_count += 1;

                        // Recursively scan subdirectories
                        self.scan_directory(
                            &path,
                            root_path,
                            directories,
                            files,
                            file_types,
                            size_distribution,
                            tracked_files,
                            current_depth + 1,
                            max_depth,
                        )
                        .await?;
                    }
                }
            }

            // Create directory information
            if current_path != root_path {
                let dir_info = DirectoryInfo {
                    path: current_path.clone(),
                    name: current_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    file_count: dir_file_count,
                    subdirectory_count: dir_subdirectory_count,
                    total_size: dir_total_size,
                    importance_score: 0.0, // Calculate later
                };
                directories.push(dir_info);
            }

            Ok(())
        })
    }

    fn create_file_info(
        &self,
        path: &PathBuf,
        root_path: &PathBuf,
        metadata: &Metadata,
    ) -> Result<FileInfo> {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string());

        let relative_path = path.strip_prefix(root_path).unwrap_or(path).to_path_buf();

        let last_modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs().to_string());

        Ok(FileInfo {
            path: relative_path,
            name,
            size: metadata.len(),
            extension,
            is_core: false,        // Calculate later
            importance_score: 0.0, // Calculate later
            complexity_score: 0.0, // Calculate later
            last_modified,
        })
    }

    fn categorize_file_size(&self, size: u64) -> String {
        match size {
            0..=1024 => "tiny".to_string(),
            1025..=10240 => "small".to_string(),
            10241..=102400 => "medium".to_string(),
            102401..=1048576 => "large".to_string(),
            _ => "huge".to_string(),
        }
    }

    fn calculate_file_importance_score(&self, file: &mut FileInfo) {
        let mut score: f64 = 0.0;

        // Weight based on file location (backend paths preferred)
        let path_str = file.path.to_string_lossy().to_lowercase();
        // Backend/Core paths get location bonus
        if path_str.contains("cmd") || path_str.contains("internal") || path_str.contains("pkg") {
            score += 0.3;
        }
        if path_str.contains("main") || path_str.contains("index") {
            score += 0.15;
        }
        if path_str.contains("config") || path_str.contains("setup") {
            score += 0.1;
        }

        // Weight based on file size
        if file.size > 1024 && file.size < 50 * 1024 {
            score += 0.15;
        }

        // Weight based on file type
        // Backend languages (higher priority): *.py, *.go, *.rs, *.java, *.kt, *.cpp, *.c, etc.
        // Frontend languages (lower priority): *.ts, *.js, *.tsx, *.jsx, *.vue, *.svelte, etc.
        if let Some(ref ext) = file.extension {
            match ext.as_str() {
                // Backend/Core languages - highest priority
                "rs" | "py" | "java" | "kt" | "cpp" | "c" | "go" | "rb" | "php" | "m"
                | "swift" | "dart" | "cs" => score += 0.4,
                // SQL and database files
                "sql" | "sqlproj" => score += 0.3,
                // Frontend frameworks (React/Vue/Svelte) - medium priority
                "jsx" | "tsx" => score += 0.2,
                "vue" | "svelte" => score += 0.2,
                "wxml" | "ttml" | "ksml" => score += 0.2,
                // JavaScript/TypeScript - lower priority
                "js" | "ts" | "mjs" | "cjs" => score += 0.15,
                // .NET project files
                "csproj" | "sln" => score += 0.2,
                // Build and package management files
                "gradle" | "pom" => score += 0.15,
                "package" => score += 0.15,
                "lock" => score += 0.05,
                // Configuration files
                "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "env" => score += 0.1,
                // Style files - lowest priority
                "css" | "scss" | "sass" | "less" | "styl" | "wxss" => score += 0.05,
                // Template files
                "html" | "htm" | "hbs" | "mustache" | "ejs" => score += 0.05,
                _ => {}
            }
        }

        // Bonus for database-related paths
        if path_str.contains("database") || path_str.contains("schema") || path_str.contains("migrations") {
            score += 0.15;
        }

        file.importance_score = score.min(1.0);
    }

    fn should_ignore_directory(&self, dir_name: &str) -> bool {
        let config = &self.context.config;
        let dir_name_lower = dir_name.to_lowercase();

        // Check excluded directories configured in Config
        for excluded_dir in &config.excluded_dirs {
            if dir_name_lower == excluded_dir.to_lowercase() {
                return true;
            }
        }

        // Check if it's a test directory (if not including test files)
        if !config.include_tests && is_test_directory(dir_name) {
            return true;
        }

        // Check hidden directories
        if !config.include_hidden && dir_name.starts_with('.') {
            return true;
        }

        false
    }

    fn should_ignore_file(&self, path: &PathBuf, tracked_files: &HashMap<PathBuf, ()>) -> bool {
        let config = &self.context.config;
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        let _path_str = path.to_string_lossy().to_lowercase();

        // Check excluded files
        for excluded_file in &config.excluded_files {
            if excluded_file.contains('*') {
                // Simple wildcard matching
                let pattern = excluded_file.replace('*', "");
                if file_name.contains(&pattern.to_lowercase()) {
                    return true;
                }
            } else if file_name == excluded_file.to_lowercase() {
                return true;
            }
        }

        // Check excluded extensions
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            if config
                .excluded_extensions
                .contains(&extension.to_lowercase())
            {
                return true;
            }
        }

        // Check included extensions (if specified)
        if !config.included_extensions.is_empty() {
            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if !config
                    .included_extensions
                    .contains(&extension.to_lowercase())
                {
                    return true;
                }
            } else {
                return true; // No extension and include list is specified
            }
        }

        // Check test files (if not including test files)
        if !config.include_tests && is_test_file(path) {
            return true;
        }

        // Check hidden files
        if !config.include_hidden && file_name.starts_with('.') {
            return true;
        }

        // Check git tracked files (if git_tracked_only is true)
        if config.git_tracked_only && !tracked_files.is_empty() && !tracked_files.contains_key(path) {
            return true;
        }

        // Check binary files
        if is_binary_file_path(path) {
            return true;
        }

        false
    }

    fn calculate_importance_scores(
        &self,
        _files: &mut Vec<FileInfo>,
        directories: &mut [DirectoryInfo],
    ) {
        // File scores are already calculated during scan via calculate_file_importance_score
        // Only recalculate if files is empty (backward compatible)

        // Calculate directory importance scores
        for dir in directories.iter_mut() {
            let mut score: f64 = 0.0;

            // Based on directory name
            let name_lower = dir.name.to_lowercase();
            if name_lower == "src" || name_lower == "lib" {
                score += 0.4;
            }
            if name_lower.contains("core") || name_lower.contains("main") {
                score += 0.3;
            }

            // Based on file count
            if dir.file_count > 5 {
                score += 0.2;
            }

            // Based on subdirectory count
            if dir.subdirectory_count > 2 {
                score += 0.1;
            }

            dir.importance_score = score.min(1.0);
        }
    }

    /// Apply directory-level LLM score as an additive boost to file importance scores
    fn apply_directory_score_boost(
        &self,
        files: &mut [FileInfo],
        dir_scores: &HashMap<PathBuf, f64>,
    ) {
        for file in files.iter_mut() {
            // Find the parent directory of this file
            if let Some(parent) = file.path.parent() {
                if let Some(&dir_score) = dir_scores.get(parent) {
                    // Add 0.1-0.2 boost based on directory score (multiplied by 0.2 for max boost)
                    let boost = dir_score * 0.2;
                    file.importance_score = (file.importance_score + boost).min(1.0);
                }
            }
        }
    }
}
