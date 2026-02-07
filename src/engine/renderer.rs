use crate::models::*;

/// All data needed to render a context output.
pub struct RenderContext<'a> {
    pub role: &'a ContextRole,
    pub level: &'a ContextLevel,
    pub meta: &'a Option<ProjectMeta>,
    pub decisions: &'a [Decision],
    pub tasks: &'a [Task],
    pub files: &'a [FileSummary],
    pub sessions: &'a [Session],
    pub active_session: &'a Option<Session>,
}

/// Render project memory into a markdown string based on role and detail level.
pub fn render(ctx: &RenderContext<'_>) -> String {
    let mut out = String::new();

    // Header
    if let Some(ref m) = ctx.meta {
        out.push_str(&format!("# {}\n", m.name));
        if !m.description.is_empty() {
            out.push_str(&format!("{}\n", m.description));
        }
        if !m.stack.is_empty() {
            out.push_str(&format!("**Stack**: {}\n", m.stack.join(", ")));
        }
    } else {
        out.push_str("# (unnamed project)\n");
    }
    out.push('\n');

    // Current session
    if let Some(ref s) = ctx.active_session {
        out.push_str(&format!("**Active session**: {} — {}\n\n", s.agent, s.goal));
    }

    match ctx.level {
        ContextLevel::Minimal => render_minimal(&mut out, ctx.role, ctx.tasks, ctx.sessions),
        ContextLevel::Standard => {
            render_standard(&mut out, ctx.role, ctx.decisions, ctx.tasks, ctx.sessions)
        }
        ContextLevel::Full => render_full(
            &mut out,
            ctx.role,
            ctx.meta,
            ctx.decisions,
            ctx.tasks,
            ctx.files,
            ctx.sessions,
        ),
    }

    out
}

fn render_minimal(out: &mut String, role: &ContextRole, tasks: &[Task], sessions: &[Session]) {
    // Next task
    let next = tasks.iter().find(|t| {
        matches!(t.status, TaskStatus::InProgress) || matches!(t.status, TaskStatus::Todo)
    });
    if let Some(t) = next {
        out.push_str(&format!("**Next**: {} ({})\n", t.title, t.status.as_str()));
    }

    // Last handoff
    if matches!(role, ContextRole::Resume) {
        if let Some(last) = sessions.iter().find(|s| s.handoff.is_some()) {
            out.push_str(&format!(
                "\n**Last handoff** ({}): {}\n",
                last.agent,
                last.handoff.as_deref().unwrap_or("")
            ));
        }
    }
}

fn render_standard(
    out: &mut String,
    role: &ContextRole,
    decisions: &[Decision],
    tasks: &[Task],
    sessions: &[Session],
) {
    // Decisions (max 5)
    if matches!(role, ContextRole::Build | ContextRole::Review) && !decisions.is_empty() {
        out.push_str("## Decisions\n");
        for d in decisions.iter().take(5) {
            out.push_str(&format!("- **{}**: {}\n", d.title, d.decision));
        }
        out.push('\n');
    }

    // Active/todo tasks (max 10)
    let active_tasks: Vec<_> = tasks
        .iter()
        .filter(|t| !matches!(t.status, TaskStatus::Done))
        .take(10)
        .collect();
    if !active_tasks.is_empty() {
        out.push_str("## Tasks\n");
        for t in &active_tasks {
            let marker = match t.status {
                TaskStatus::InProgress => "🔧",
                TaskStatus::Blocked => "⛔",
                TaskStatus::Todo => "☐",
                TaskStatus::Done => "✓",
            };
            let phase = t.phase.as_deref().unwrap_or("");
            let phase_str = if phase.is_empty() {
                String::new()
            } else {
                format!(" [{phase}]")
            };
            out.push_str(&format!(
                "- {marker} {} ({}){phase_str}\n",
                t.title,
                t.priority.as_str()
            ));
        }
        out.push('\n');
    }

    // Last session
    if let Some(last) = sessions.first() {
        out.push_str("## Last Session\n");
        let status = if last.ended_at.is_some() {
            "ended"
        } else {
            "active"
        };
        out.push_str(&format!("**{}** ({status}): {}\n", last.agent, last.goal));
        if let Some(ref handoff) = last.handoff {
            out.push_str(&format!("Handoff: {handoff}\n"));
        }
        out.push('\n');
    }

    // Debug-specific: recent changes
    if matches!(role, ContextRole::Debug) {
        let recent: Vec<_> = tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Done))
            .take(5)
            .collect();
        if !recent.is_empty() {
            out.push_str("## Recently Completed\n");
            for t in &recent {
                out.push_str(&format!("- ✓ {}\n", t.title));
            }
            out.push('\n');
        }
    }
}

fn render_full(
    out: &mut String,
    role: &ContextRole,
    meta: &Option<ProjectMeta>,
    decisions: &[Decision],
    tasks: &[Task],
    files: &[FileSummary],
    sessions: &[Session],
) {
    // Conventions
    if let Some(ref m) = meta {
        if !m.conventions.is_empty() {
            out.push_str("## Conventions\n");
            for c in &m.conventions {
                out.push_str(&format!("- {c}\n"));
            }
            out.push('\n');
        }
    }

    // All decisions with rationale
    if !decisions.is_empty() {
        out.push_str("## Decisions\n");
        for d in decisions {
            out.push_str(&format!("### {}\n", d.title));
            out.push_str(&format!("**Context**: {}\n", d.context));
            out.push_str(&format!("**Decision**: {}\n", d.decision));
            if !d.alternatives.is_empty() {
                out.push_str(&format!(
                    "**Alternatives**: {}\n",
                    d.alternatives.join(", ")
                ));
            }
            out.push('\n');
        }
    }

    // All tasks
    if !tasks.is_empty() {
        out.push_str("## Tasks\n");
        for t in tasks {
            let marker = match t.status {
                TaskStatus::InProgress => "🔧",
                TaskStatus::Blocked => "⛔",
                TaskStatus::Todo => "☐",
                TaskStatus::Done => "✓",
            };
            let phase = t.phase.as_deref().unwrap_or("");
            let phase_str = if phase.is_empty() {
                String::new()
            } else {
                format!(" [{phase}]")
            };
            out.push_str(&format!(
                "- {marker} {} ({}){phase_str}\n",
                t.title,
                t.priority.as_str()
            ));
            if !t.description.is_empty() {
                out.push_str(&format!("  {}\n", t.description));
            }
        }
        out.push('\n');
    }

    // File summaries (for review and build roles)
    if matches!(
        role,
        ContextRole::Build | ContextRole::Review | ContextRole::Debug
    ) && !files.is_empty()
    {
        out.push_str("## Files\n");
        for f in files {
            out.push_str(&format!("- **{}**: {}\n", f.path, f.summary));
            if !f.key_types.is_empty() {
                out.push_str(&format!("  Types: {}\n", f.key_types.join(", ")));
            }
        }
        out.push('\n');
    }

    // Session history
    if !sessions.is_empty() {
        out.push_str("## Session History\n");
        for s in sessions {
            let status = if s.ended_at.is_some() {
                "ended"
            } else {
                "active"
            };
            out.push_str(&format!(
                "- **{}** ({status}, {}): {}\n",
                s.agent,
                s.started_at.format("%Y-%m-%d %H:%M"),
                s.goal,
            ));
            if let Some(ref handoff) = s.handoff {
                out.push_str(&format!("  Handoff: {handoff}\n"));
            }
        }
        out.push('\n');
    }
}
