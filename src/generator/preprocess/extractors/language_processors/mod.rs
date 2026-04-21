use std::path::Path;

use crate::types::code::{CodeComplexity, Dependency, InterfaceInfo};

/// Language processor trait
pub trait LanguageProcessor: Send + Sync + std::fmt::Debug {
    /// Get supported file extensions
    fn supported_extensions(&self) -> Vec<&'static str>;

    /// Extract file dependencies
    fn extract_dependencies(&self, content: &str, file_path: &Path) -> Vec<Dependency>;

    /// Determine component type
    #[allow(dead_code)]
    fn determine_component_type(&self, file_path: &Path, content: &str) -> String;

    /// Identify important code lines
    #[allow(dead_code)]
    fn is_important_line(&self, line: &str) -> bool;

    /// Get language name
    #[allow(dead_code)]
    fn language_name(&self) -> &'static str;

    /// Extract code interface definitions
    fn extract_interfaces(&self, content: &str, file_path: &Path) -> Vec<InterfaceInfo>;
}

/// Language processor manager
#[derive(Debug)]
pub struct LanguageProcessorManager {
    processors: Vec<Box<dyn LanguageProcessor>>,
}

impl Clone for LanguageProcessorManager {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl LanguageProcessorManager {
    pub fn new() -> Self {
        Self {
            processors: vec![
                Box::new(rust::RustProcessor::new()),
                Box::new(javascript::JavaScriptProcessor::new()),
                Box::new(typescript::TypeScriptProcessor::new()),
                Box::new(php::PhpProcessor::new()),
                Box::new(react::ReactProcessor::new()),
                Box::new(vue::VueProcessor::new()),
                Box::new(svelte::SvelteProcessor::new()),
                Box::new(kotlin::KotlinProcessor::new()),
                Box::new(python::PythonProcessor::new()),
                Box::new(java::JavaProcessor::new()),
                Box::new(csharp::CSharpProcessor::new()),
                Box::new(swift::SwiftProcessor::new()),
            ],
        }
    }

    /// Get processor by file extension
    pub fn get_processor(&self, file_path: &Path) -> Option<&dyn LanguageProcessor> {
        let extension = file_path.extension()?.to_str()?;

        for processor in &self.processors {
            if processor.supported_extensions().contains(&extension) {
                return Some(processor.as_ref());
            }
        }

        None
    }

    /// Extract file dependencies
    pub fn extract_dependencies(&self, file_path: &Path, content: &str) -> Vec<Dependency> {
        if let Some(processor) = self.get_processor(file_path) {
            processor.extract_dependencies(content, file_path)
        } else {
            Vec::new()
        }
    }

    /// Determine component type
    #[allow(dead_code)]
    pub fn determine_component_type(&self, file_path: &Path, content: &str) -> String {
        if let Some(processor) = self.get_processor(file_path) {
            processor.determine_component_type(file_path, content)
        } else {
            "unknown".to_string()
        }
    }

    /// Identify important code lines
    #[allow(dead_code)]
    pub fn is_important_line(&self, file_path: &Path, line: &str) -> bool {
        if let Some(processor) = self.get_processor(file_path) {
            processor.is_important_line(line)
        } else {
            false
        }
    }

    /// Extract code interface definitions
    pub fn extract_interfaces(&self, file_path: &Path, content: &str) -> Vec<InterfaceInfo> {
        if let Some(processor) = self.get_processor(file_path) {
            processor.extract_interfaces(content, file_path)
        } else {
            Vec::new()
        }
    }

    pub fn calculate_complexity_metrics(&self, content: &str) -> CodeComplexity {
        let lines: Vec<&str> = content.lines().collect();
        let lines_of_code = lines.len();

        // Simplified complexity calculation
        let number_of_functions = content.matches("fn ").count()
            + content.matches("def ").count()
            + content.matches("function ").count()
            + content.matches("async ").count();  // C# async methods

        let number_of_classes =
            content.matches("class ").count() 
            + content.matches("struct ").count()
            + content.matches("interface ").count();  // C# interfaces

        // Simplified cyclomatic complexity calculation
        let cyclomatic_complexity = 1.0
            + content.matches("if ").count() as f64
            + content.matches("while ").count() as f64
            + content.matches("for ").count() as f64
            + content.matches("foreach ").count() as f64  // C# foreach
            + content.matches("match ").count() as f64
            + content.matches("switch ").count() as f64  // C# switch
            + content.matches("case ").count() as f64;

        CodeComplexity {
            cyclomatic_complexity,
            lines_of_code,
            number_of_functions,
            number_of_classes,
        }
    }
}

// Submodules
pub mod csharp;
pub mod java;
pub mod javascript;
pub mod kotlin;
pub mod php;
pub mod python;
pub mod react;
pub mod rust;
pub mod svelte;
pub mod swift;
pub mod typescript;
pub mod vue;
