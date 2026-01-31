# SQL Project Support for .NET Projects

## Overview

Litho (deepwiki-rs) now supports SQL database projects (`.sqlproj`) and SQL script files (`.sql`) commonly found in .NET solutions. This feature enables comprehensive analysis of database schemas, stored procedures, views, functions, and their dependencies.

## Supported Files

### `.sqlproj` - SQL Database Project Files
MSBuild-based project files that define database schemas, compilation settings, and references.

**Parsed Elements:**
- `<Build>` items - SQL scripts included in the project
- `<ProjectReference>` - References to other projects
- `<ArtifactReference>` - References to DACPAC files

**Example:**
```xml
<Project DefaultTargets="Build">
  <ItemGroup>
    <Build Include="Tables\Users.sql" />
    <Build Include="StoredProcedures\GetUser.sql" />
    <ProjectReference Include="..\SharedDatabase\SharedDatabase.sqlproj" />
    <ArtifactReference Include="$(DacPacRootPath)\System.Data.dll" />
  </ItemGroup>
</Project>
```

### `.sql` - SQL Script Files
SQL scripts containing table definitions, stored procedures, views, functions, triggers, or general SQL commands.

**Parsed Elements:**
- Table references (FROM, JOIN, INTO clauses)
- Stored procedure calls (EXEC, EXECUTE)

**Example:**
```sql
CREATE TABLE Users (
    Id INT PRIMARY KEY,
    Name NVARCHAR(100)
);

CREATE PROCEDURE GetUser @UserId INT
AS
BEGIN
    SELECT * FROM Users WHERE Id = @UserId;
END;
```

## Component Types

SQL files are classified into specific component types:

| Component Type | Description | Example |
|---------------|-------------|---------|
| `sql_database_project` | SQL database project definition | `MyDatabase.sqlproj` |
| `sql_table_definition` | Table schema (CREATE TABLE) | `Users.sql` |
| `sql_stored_procedure` | Stored procedure (CREATE PROCEDURE) | `GetUser.sql` |
| `sql_view` | View definition (CREATE VIEW) | `ActiveUsers.sql` |
| `sql_function` | User-defined function (CREATE FUNCTION) | `CalculateTotal.sql` |
| `sql_trigger` | Database trigger (CREATE TRIGGER) | `AuditLog.sql` |
| `sql_script` | General SQL script | `Setup.sql` |

## Dependency Extraction

### `.sqlproj` Dependencies

| Dependency Type | Description | Example |
|----------------|-------------|---------|
| `database_reference` | Reference to another SQL project | `SharedDatabase.sqlproj` |
| `dacpac_reference` | DACPAC artifact reference | `System.Data.dll` |

### `.sql` Dependencies

| Dependency Type | Description | Example |
|----------------|-------------|---------|
| `table_reference` | Table referenced in SQL code | `Users`, `Orders` |
| `stored_procedure_call` | Called stored procedure | `GetUser`, `UpdateOrder` |

## Code Classification

### Automatic Database Classification

**All `.sqlproj` and `.sql` files are automatically classified as `CodePurpose::Database`** regardless of their location in the project structure. This ensures SQL-related files are properly categorized in the documentation.

### Classification Rules

The code classification system uses the following priority:

1. **Extension-based (Highest Priority)**
   - `.sqlproj` → `Database`
   - `.sql` → `Database`

2. **Path-based**
   - `/database/`, `/db/`, `/storage/` → `Database`
   - `/dao/`, `/repository/`, `/persistence/` → `Dao`

3. **Name-based**
   - Filename contains "database" → `Database`
   - Filename contains "repository" or "persistence" → `Dao`

4. **AI Analysis (Fallback)**
   - If none of the above rules match, AI analysis determines the classification

### Examples

```rust
// ✅ Classified as Database (extension rule)
"/src/MyProject.sqlproj"
"/scripts/CreateTables.sql"
"/Schema.sql"

// ✅ Classified as Database (path rule + extension rule)
"/src/database/schema.sql"
"/src/db/migrations/001_initial.sql"

// ✅ Classified as Dao (path rule)
"/src/repository/UserRepository.cs"
"/src/dao/OrderDao.cs"
```

## Best Practices

### Project Structure

For optimal analysis, organize SQL files in a clear structure:

```
MyProject.sqlproj
├── Tables/
│   ├── Users.sql
│   ├── Orders.sql
│   └── Products.sql
├── StoredProcedures/
│   ├── GetUser.sql
│   ├── CreateOrder.sql
│   └── UpdateProduct.sql
├── Views/
│   ├── ActiveUsers.sql
│   └── OrderSummary.sql
└── Functions/
    ├── CalculateTotal.sql
    └── FormatDate.sql
```

### Naming Conventions

- Use descriptive names that reflect the SQL object type
- Table files: `Users.sql`, `Orders.sql`
- Stored procedures: `GetUser.sql`, `CreateOrder.sql`
- Views: `ActiveUsers.sql`, `OrderSummary.sql`

### SQL Code Style

For better dependency extraction:
- Use explicit table references in queries
- Avoid dynamic SQL when possible
- Include schema names for clarity (e.g., `dbo.Users`)

## Testing

The SQL file classification is verified with comprehensive unit tests:

```rust
#[test]
fn test_sql_file_classification() {
    // All SQL files classified as Database
    assert_eq!(
        CodePurposeMapper::map_by_path_and_name(
            "/src/MyProject.sqlproj",
            "MyProject.sqlproj"
        ),
        CodePurpose::Database
    );
}
```

Run tests with:
```powershell
cargo test types::code::tests
```

## Technical Details

### Implementation

- **Processor**: `CSharpProcessor` in `src/generator/preprocess/extractors/language_processors/csharp.rs`
- **Classification**: `CodePurposeMapper` in `src/types/code.rs`
- **Supported Extensions**: Added to `CSharpProcessor::supported_extensions()`

### Regular Expressions

SQL dependency extraction uses regex patterns:
- **Tables**: `(?i)\b(?:FROM|JOIN|INTO)\s+([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)?)`
- **Stored Procedures**: `(?i)\b(?:EXEC|EXECUTE)\s+([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)?)`

### Limitations

- Dynamic SQL (constructed at runtime) is not analyzed
- Complex T-SQL features (CTEs, subqueries) may have limited dependency extraction
- Schema names are captured but not validated against the actual database

## Future Enhancements

Potential improvements:
- [ ] Parse CREATE TABLE statements for column definitions
- [ ] Extract foreign key relationships
- [ ] Analyze stored procedure dependencies (one calling another)
- [ ] Support for database migrations (e.g., Entity Framework migrations)
- [ ] Integration with SQL Server schema validation
- [ ] DACPAC file introspection

## Related Documentation

- [C# Processor Implementation](../src/generator/preprocess/extractors/language_processors/csharp.rs)
- [Code Classification System](../src/types/code.rs)
- [Configuration Guide](../README.md)
