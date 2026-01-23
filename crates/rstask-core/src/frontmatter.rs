use crate::Result;
use crate::RstaskError;
use crate::task::Task;
use serde::{Deserialize, Serialize};

/// Serialize a task to markdown with YAML frontmatter
/// The notes field becomes the markdown content, everything else goes in frontmatter
pub fn task_to_markdown(task: &Task) -> Result<String> {
    // Create a copy without notes for frontmatter
    let frontmatter_task = TaskFrontmatter {
        summary: task.summary.clone(),
        tags: if task.tags.is_empty() {
            None
        } else {
            Some(task.tags.clone())
        },
        project: if task.project.is_empty() {
            None
        } else {
            Some(task.project.clone())
        },
        priority: if task.priority.is_empty() {
            None
        } else {
            Some(task.priority.clone())
        },
        delegatedto: if task.delegated_to.is_empty() {
            None
        } else {
            Some(task.delegated_to.clone())
        },
        subtasks: if task.subtasks.is_empty() {
            None
        } else {
            Some(task.subtasks.clone())
        },
        dependencies: if task.dependencies.is_empty() {
            None
        } else {
            Some(task.dependencies.clone())
        },
        created: task.created,
        resolved: task.resolved,
        due: task.due,
    };

    let yaml_frontmatter = serde_yaml::to_string(&frontmatter_task).map_err(RstaskError::Yaml)?;

    let mut result = String::from("---\n");
    result.push_str(&yaml_frontmatter);
    result.push_str("---\n");

    if !task.notes.is_empty() {
        result.push('\n');
        result.push_str(&task.notes);
        if !task.notes.ends_with('\n') {
            result.push('\n');
        }
    }

    Ok(result)
}

/// Deserialize a task from markdown with YAML frontmatter
pub fn task_from_markdown(content: &str, uuid: &str, status: &str, id: i32) -> Result<Task> {
    // Find the frontmatter boundaries
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0] != "---" {
        return Err(RstaskError::Parse(
            "missing frontmatter delimiter".to_string(),
        ));
    }

    // Find the closing ---
    let closing_idx = lines[1..]
        .iter()
        .position(|&line| line == "---")
        .ok_or_else(|| RstaskError::Parse("missing closing frontmatter delimiter".to_string()))?;

    // Extract frontmatter (lines between the two ---)
    let frontmatter_lines = &lines[1..closing_idx + 1];
    let frontmatter_str = frontmatter_lines.join("\n");

    // Extract markdown content (everything after the closing ---)
    let content_start = closing_idx + 2; // +2 to skip past the closing ---
    let notes = if content_start < lines.len() {
        let content_lines = &lines[content_start..];
        let content = content_lines.join("\n");
        // Trim leading empty lines but preserve other whitespace
        content.trim_start_matches('\n').to_string()
    } else {
        String::new()
    };

    // Deserialize frontmatter
    let frontmatter: TaskFrontmatter =
        serde_yaml::from_str(&frontmatter_str).map_err(RstaskError::Yaml)?;

    // Construct the task
    let task = Task {
        uuid: uuid.to_string(),
        status: status.to_string(),
        write_pending: false,
        id,
        deleted: false,
        summary: frontmatter.summary,
        notes,
        tags: frontmatter.tags.unwrap_or_default(),
        project: frontmatter.project.unwrap_or_default(),
        priority: frontmatter.priority.unwrap_or_default(),
        delegated_to: frontmatter.delegatedto.unwrap_or_default(),
        subtasks: frontmatter.subtasks.unwrap_or_default(),
        dependencies: frontmatter.dependencies.unwrap_or_default(),
        created: frontmatter.created,
        resolved: frontmatter.resolved,
        due: frontmatter.due,
        filtered: false,
    };

    Ok(task)
}

/// Task frontmatter structure (task without notes)
#[derive(Debug, Serialize, Deserialize, Clone)]
struct TaskFrontmatter {
    summary: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    project: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    delegatedto: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    subtasks: Option<Vec<crate::task::SubTask>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<Vec<String>>,

    #[serde(with = "crate::task::datetime_rfc3339")]
    created: chrono::DateTime<chrono::Utc>,

    #[serde(
        with = "crate::task::optional_datetime_rfc3339",
        skip_serializing_if = "Option::is_none",
        default
    )]
    resolved: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(
        with = "crate::task::optional_datetime_rfc3339",
        skip_serializing_if = "Option::is_none",
        default
    )]
    due: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_task_to_markdown_basic() {
        let task = Task {
            uuid: "test-uuid".to_string(),
            status: "pending".to_string(),
            write_pending: false,
            id: 1,
            deleted: false,
            summary: "Test task".to_string(),
            notes: "This is a note\nWith multiple lines".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            project: "myproject".to_string(),
            priority: "H".to_string(),
            delegated_to: String::new(),
            subtasks: vec![],
            dependencies: vec![],
            created: Utc::now(),
            resolved: None,
            due: None,
            filtered: false,
        };

        let md = task_to_markdown(&task).unwrap();

        assert!(md.starts_with("---\n"));
        assert!(md.contains("summary: Test task"));
        assert!(md.contains("This is a note\nWith multiple lines"));
    }

    #[test]
    fn test_task_from_markdown_basic() {
        let content = r#"---
summary: Test task
tags:
- tag1
- tag2
project: myproject
priority: H
created: 2024-01-01T00:00:00Z
---

This is a note
With multiple lines"#;

        let task = task_from_markdown(content, "test-uuid", "pending", 1).unwrap();

        assert_eq!(task.summary, "Test task");
        assert_eq!(task.notes, "This is a note\nWith multiple lines");
        assert_eq!(task.tags, vec!["tag1", "tag2"]);
        assert_eq!(task.project, "myproject");
        assert_eq!(task.priority, "H");
    }

    #[test]
    fn test_task_roundtrip() {
        let original = Task {
            uuid: "test-uuid".to_string(),
            status: "pending".to_string(),
            write_pending: false,
            id: 1,
            deleted: false,
            summary: "Test task".to_string(),
            notes: "Note content".to_string(),
            tags: vec!["tag1".to_string()],
            project: "project1".to_string(),
            priority: "M".to_string(),
            delegated_to: String::new(),
            subtasks: vec![],
            dependencies: vec![],
            created: Utc::now(),
            resolved: None,
            due: None,
            filtered: false,
        };

        let md = task_to_markdown(&original).unwrap();
        let restored = task_from_markdown(&md, "test-uuid", "pending", 1).unwrap();

        assert_eq!(original.summary, restored.summary);
        assert_eq!(original.notes, restored.notes);
        assert_eq!(original.tags, restored.tags);
        assert_eq!(original.project, restored.project);
        assert_eq!(original.priority, restored.priority);
    }
}
