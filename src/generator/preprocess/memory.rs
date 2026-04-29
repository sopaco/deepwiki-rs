pub struct MemoryScope;

impl MemoryScope {
    pub const PREPROCESS: &'static str = "preprocess";
}

pub struct ScopedKeys;

impl ScopedKeys {
    pub const ORIGINAL_DOCUMENT: &'static str = "original_document";
    pub const PROJECT_STRUCTURE: &'static str = "project_structure";
    // CODE_INSIGHTS now stores CodeAndDirectoryInsights (merged file + directory insights)
    pub const CODE_INSIGHTS: &'static str = "code_insights";
    pub const RELATIONSHIPS: &'static str = "relationships";
    pub const DIRECTORY_SELECTION: &'static str = "directory_selection";
}