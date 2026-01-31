use crate::generator::compose::types::AgentType;
use crate::generator::{compose::memory::MemoryScope, context::GeneratorContext};
use crate::i18n::TargetLanguage;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;

pub mod summary_generator;
pub mod summary_outlet;
pub mod fixer;

pub use summary_outlet::SummaryOutlet;
pub use fixer::MermaidFixer;

pub trait Outlet {
    async fn save(&self, context: &GeneratorContext) -> Result<()>;
}

pub struct DocTree {
    /// key is the ScopedKey of Documentation in Memory, value is the relative path for document output
    structure: HashMap<String, String>,
}

impl DocTree {
    pub fn new(target_language: &TargetLanguage) -> Self {
        let structure = HashMap::from([
            (
                AgentType::Overview.to_string(),
                target_language.get_doc_filename("overview"),
            ),
            (
                AgentType::Architecture.to_string(),
                target_language.get_doc_filename("architecture"),
            ),
            (
                AgentType::Workflow.to_string(),
                target_language.get_doc_filename("workflow"),
            ),
            (
                AgentType::Boundary.to_string(),
                target_language.get_doc_filename("boundary"),
            ),
            (
                AgentType::Database.to_string(),
                target_language.get_doc_filename("database"),
            ),
        ]);
        Self { structure }
    }

    pub fn insert(&mut self, scoped_key: &str, relative_path: &str) {
        self.structure
            .insert(scoped_key.to_string(), relative_path.to_string());
    }
}

impl Default for DocTree {
    fn default() -> Self {
        // Default to English
        Self::new(&TargetLanguage::English)
    }
}

pub struct DiskOutlet {
    doc_tree: DocTree,
}

impl DiskOutlet {
    pub fn new(doc_tree: DocTree) -> Self {
        Self { doc_tree }
    }
}

impl Outlet for DiskOutlet {
    async fn save(&self, context: &GeneratorContext) -> Result<()> {
        println!("\n🖊️ Saving documentation...");
        // Create output directory
        let output_dir = &context.config.output_path;
        if output_dir.exists() {
            fs::remove_dir_all(output_dir)?;
        }
        fs::create_dir_all(output_dir)?;

        // Iterate through document tree structure and save each document
        for (scoped_key, relative_path) in &self.doc_tree.structure {
            // Get document content from memory
            if let Some(doc_markdown) = context
                .get_from_memory::<String>(MemoryScope::DOCUMENTATION, scoped_key)
                .await
            {
                // Build full output file path
                let output_file_path = output_dir.join(relative_path);

                // Ensure parent directory exists
                if let Some(parent_dir) = output_file_path.parent() {
                    if !parent_dir.exists() {
                        fs::create_dir_all(parent_dir)?;
                    }
                }

                // Write document content to file
                fs::write(&output_file_path, doc_markdown)?;

                println!("💾 Document saved: {}", output_file_path.display());
            } else {
                // If document doesn't exist, log warning but don't interrupt the process
                let msg = context.config.target_language.msg_doc_not_found();
                eprintln!("{}", msg.replace("{}", scoped_key));
            }
        }

        println!("💾 Document save completed, output directory: {}", output_dir.display());

        // Automatically fix mermaid charts after document save
        if let Err(e) = MermaidFixer::auto_fix_after_output(context).await {
            let msg = context.config.target_language.msg_mermaid_error();
            eprintln!("{}", msg.replace("{}", &e.to_string()));
            eprintln!("💡 This will not affect the main documentation generation process");
        }

        Ok(())
    }
}
