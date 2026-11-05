use super::{ACTIONS, App, Input, Screen, config_fields};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
        TableState, Wrap,
    },
};

const BG: Color = Color::Rgb(14, 18, 25);
const TEXT: Color = Color::Rgb(220, 226, 236);
const MUTED: Color = Color::Rgb(133, 146, 165);
const ACCENT: Color = Color::Rgb(133, 181, 233);
fn panel(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .title(format!(" {} ", title.into()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MUTED))
}
pub fn safe(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect()
}
fn paragraph(frame: &mut Frame, area: Rect, title: &str, text: String, scroll: u16) {
    frame.render_widget(
        Paragraph::new(safe(&text))
            .block(panel(title))
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        area,
    );
}
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(BG).fg(TEXT)),
        area,
    );
    if area.width < 60 || area.height < 16 {
        frame.render_widget(
            Paragraph::new(
                "Radar • resize terminal to at least 60×16\nq quit; resizing preserves state",
            )
            .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(7),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(area);
    let connection = if app.busy {
        "SYNCING"
    } else if app.connected {
        "ONLINE"
    } else {
        "OFFLINE / STALE"
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " PERSONAL FREELANCE RADAR ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {connection}"), Style::default().fg(MUTED)),
        ])),
        layout[0],
    );
    let tabs = if area.width < 110 {
        [
            "1Home", "2Jobs", "3Job", "4Score", "5Cfg", "6Feed", "7Alerts", "8Learn", "9Profile",
        ]
    } else {
        [
            "1 Dashboard",
            "2 Jobs",
            "3 Details",
            "4 Evaluation",
            "5 Config",
            "6 Feedback",
            "7 Alerts",
            "8 Learning",
            "9 Profiles",
        ]
    };
    let line = Line::from(
        tabs.iter()
            .enumerate()
            .flat_map(|(i, name)| {
                vec![
                    Span::styled(
                        format!(" {name} "),
                        if i == app.screen.index() {
                            Style::default().fg(BG).bg(ACCENT)
                        } else {
                            Style::default().fg(MUTED)
                        },
                    ),
                    Span::raw(" "),
                ]
            })
            .collect::<Vec<_>>(),
    );
    frame.render_widget(Paragraph::new(line), layout[1]);
    match app.screen {
        Screen::Dashboard => render_dashboard(frame, app, layout[2]),
        Screen::Jobs => render_jobs(frame, app, layout[2]),
        Screen::Details => render_details(frame, app, layout[2], false),
        Screen::Evaluation => render_details(frame, app, layout[2], true),
        Screen::Config => render_config(frame, app, layout[2]),
        Screen::Feedback => render_feedback(frame, app, layout[2]),
        Screen::Notifications => render_notifications(frame, app, layout[2]),
        Screen::Profiles => super::profile::render(frame, app, layout[2]),
        Screen::Preferences => render_preferences(frame, app, layout[2]),
    }
    frame.render_widget(
        Paragraph::new(safe(&app.status))
            .style(Style::default().fg(if app.connected {
                MUTED
            } else {
                Color::Rgb(230, 172, 103)
            }))
            .wrap(Wrap { trim: true }),
        layout[3],
    );
    let help = match app.screen {
        Screen::Profiles => {
            "[ ] section • ↑↓ record • n add • e edit • Del remove • p CV text • v preview • s save • d discard • PgUp/PgDn scroll"
        }
        Screen::Jobs => {
            "↑↓ select • Enter detail • / search • v verdict • g decision • o sort • [ ] page • f feedback"
        }
        Screen::Details | Screen::Evaluation => {
            "e evaluate • a AI view • b approve analysis • T template • G draft • P proposal • E edit • Y copy approved • C cancel • 3/4 score • f feedback"
        }
        Screen::Config => {
            "↑↓ field • Enter edit • s validate/save • d discard draft • changes require server approval"
        }
        Screen::Feedback => {
            "↑↓ select • [ ] page • R correct/exclude learning • r refresh • decisions and outcomes stay separate"
        }
        Screen::Dashboard => {
            "m mock scan • u manual Upwork • p pause/resume • r refresh • 7 delivery history"
        }
        Screen::Notifications => "↑↓ scroll • r refresh • delivery states are server-confirmed",
        Screen::Preferences => {
            "↑↓ model • R rebuild • v shadow • a activate • b rollback • e/d on/off • [ ] feature • s/u suppress/restore • c calibration • x reset • z reject • Y export"
        }
    };
    frame.render_widget(
        Paragraph::new(format!(
            "{help}\n1–8 / Tab screens • r refresh • t retry failed operation • q quit"
        ))
        .style(Style::default().fg(MUTED)),
        layout[4],
    );
    if let Some(input) = &app.input {
        render_input(frame, input);
    }
}
fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(s) = &app.snapshot {
        let sections = Layout::vertical([Constraint::Length(5), Constraint::Min(3)]).split(area);
        let cards = Layout::horizontal([Constraint::Ratio(1, 3); 3]).split(sections[0]);
        for (i, (title, value)) in [
            ("Jobs", s.dashboard.jobs),
            ("Candidates", s.dashboard.candidates),
            ("Feedback", s.dashboard.feedback),
        ]
        .into_iter()
        .enumerate()
        {
            frame.render_widget(
                Paragraph::new(format!(" {value}"))
                    .style(Style::default().fg(ACCENT))
                    .block(panel(title)),
                cards[i],
            );
        }
        let mut text = format!(
            "Mode: {:?}    Config revision: {}\nReview jobs: {}\nLast import: {}\n",
            s.dashboard.mode,
            s.dashboard.config_revision,
            s.dashboard.reviews,
            s.dashboard
                .last_import
                .map(|v| v.to_rfc3339())
                .unwrap_or_else(|| "No imports yet; press m".into())
        );
        text.push_str(&format!(
            "Scheduler: {} | Next: {}\nLast execution: {}\n\n",
            if s.scheduler.enabled {
                "enabled"
            } else {
                "disabled"
            },
            s.scheduler
                .next_scan
                .map(|at| at.to_rfc3339())
                .unwrap_or_else(|| "none while paused/disabled".into()),
            s.scheduler
                .last_run
                .as_ref()
                .map(|run| format!(
                    "{:?} • {} • {}",
                    run.status,
                    run.scheduled_at,
                    run.result
                        .as_ref()
                        .map(|r| r.status.clone())
                        .or_else(|| run.error.clone())
                        .unwrap_or_else(|| "queued/in progress".into())
                ))
                .unwrap_or_else(|| "no runs yet".into())
        ));
        text.push_str(&format!(
            "Active workers: {} | Last heartbeat: {}\n\n",
            s.scheduler.active_workers,
            s.scheduler
                .last_heartbeat
                .map(|at| at.to_rfc3339())
                .unwrap_or_else(|| "none; workers may be disabled/offline".into())
        ));
        text.push_str("Recent runs\n");
        for run in s.scheduler.runs.iter().take(5) {
            text.push_str(&format!(
                "[{:?}] {} • {} • attempts {}\n",
                run.status, run.origin, run.scheduled_at, run.attempts
            ));
        }
        text.push('\n');
        let u = &s.upwork;
        text.push_str(&format!("Upwork [{}] • manual only\n{} • {}\nFreelancer account: {}\nFetched {} • accepted {} • new {} • duplicates {} • invalid {} • filtered {} • partial {}\nLast successful search: {}\nLast error: {}\nIgnored upstream filters: {:?}; skills filtered client-side: {}\nEmpty reason: {}\n\n",
            if u.enabled {"enabled"}else{"disabled"},u.connection,u.authentication,u.account_label.as_deref().unwrap_or(if u.account_selected{"selected"}else{"not selected"}),u.fetched,u.accepted,u.new_jobs,u.duplicates,u.invalid,u.filtered,u.partial,
            u.last_successful_search.map(|v|v.to_rfc3339()).unwrap_or_else(||"none".into()),u.last_error_category.as_deref().unwrap_or("none"),u.filters_ignored,u.skills_filtered_client_side,u.empty_reason.as_deref().unwrap_or("none")));
        text.push_str("Sources\n");
        for source in &s.dashboard.sources {
            text.push_str(&format!(
                "{} [{}]  {}\n",
                source.name,
                if source.enabled {
                    "enabled"
                } else {
                    "disabled"
                },
                source.diagnostic
            ));
        }
        text.push_str("\nDiscord instances\n");
        for bot in &s.notifications.providers {
            text.push_str(&format!(
                "{} [{}] {}\n",
                bot.instance,
                if bot.configured {
                    "configured"
                } else {
                    "unavailable"
                },
                bot.diagnostic
            ));
        }
        text.push_str("\nBackground workers keep running when this terminal is closed. Refresh to read their progress.");
        paragraph(frame, sections[1], "Workspace", text, app.scroll);
    } else {
        paragraph(
            frame,
            area,
            "Workspace",
            "Waiting for authenticated server state. Press r to reconnect.".into(),
            0,
        );
    }
}
fn render_jobs(frame: &mut Frame, app: &App, area: Rect) {
    let Some(snapshot) = &app.snapshot else {
        return paragraph(
            frame,
            area,
            "Job Explorer",
            "No server snapshot available".into(),
            0,
        );
    };
    let title = format!(
        "Job Explorer • {} jobs • page {} • {:?} • verdict {:?} • decision {:?} • search {}",
        snapshot.jobs.total,
        snapshot.jobs.page,
        app.jobs.sort.unwrap_or_default(),
        app.jobs.verdict,
        app.jobs.decision,
        app.jobs.search.as_deref().unwrap_or("")
    );
    let personalized = snapshot
        .jobs
        .items
        .iter()
        .any(|job| job.personalization.is_some());
    let rows = snapshot.jobs.items.iter().map(|job| {
        let mut cells = vec![
            Cell::from(
                job.evaluation
                    .score
                    .map(|v| format!("{v:.1}"))
                    .unwrap_or_else(|| "VETO".into()),
            ),
            Cell::from(format!("{:?}", job.evaluation.verdict)),
            Cell::from(safe(&job.job.title).replace('\n', " ")),
            Cell::from(
                job.decision
                    .map(|a| format!("{a:?}"))
                    .unwrap_or_else(|| "—".into()),
            ),
        ];
        if personalized {
            cells.insert(
                1,
                Cell::from(
                    job.personalization
                        .as_ref()
                        .and_then(|p| p.final_score)
                        .or(job.evaluation.score)
                        .map(|v| format!("{v:.1}"))
                        .unwrap_or_else(|| "—".into()),
                ),
            );
        }
        Row::new(cells)
    });
    let mut widths = vec![
        Constraint::Length(6),
        Constraint::Length(15),
        Constraint::Min(12),
        Constraint::Length(14),
    ];
    let mut headers = vec!["Base", "Verdict", "Title", "Decision"];
    if personalized {
        widths.insert(1, Constraint::Length(6));
        headers.insert(1, "Rank");
    }
    let table = Table::new(rows, widths)
        .header(Row::new(headers).style(Style::default().fg(MUTED)))
        .block(panel(title))
        .row_highlight_style(
            Style::default()
                .bg(Color::Rgb(31, 45, 62))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    let mut state =
        TableState::default().with_selected((!snapshot.jobs.items.is_empty()).then_some(app.row));
    frame.render_stateful_widget(table, area, &mut state);
}
fn render_details(frame: &mut Frame, app: &App, area: Rect, evaluation: bool) {
    if app.ai_view {
        render_ai(frame, app, area);
        return;
    }

    let Some(job) = &app.selected else {
        return paragraph(
            frame,
            area,
            "Job Details",
            "Select a job in Job Explorer and press Enter.".into(),
            0,
        );
    };
    let e = &job.evaluation;
    let text = if evaluation {
        let mut text = format!(
            "{}\nVerdict: {:?}    Score: {}    Confidence: {:?}    Risk: {}\nConfig: {}\nProfile: {}\n\nWeighted signals\n",
            job.job.title,
            e.verdict,
            e.score
                .map(|v| v.to_string())
                .unwrap_or_else(|| "VETO".into()),
            e.confidence,
            e.risk,
            e.config_revision,
            e.profile_revision
        );
        for (signal, value) in &e.breakdown {
            text.push_str(&format!(
                "{signal:?}: {}\n",
                value
                    .map(|v| format!("{v:.1}"))
                    .unwrap_or_else(|| "unknown".into())
            ));
        }
        for (label, values) in [
            ("Hard vetoes", &e.vetoes),
            ("Evidence", &e.evidence),
            ("Strengths", &e.strengths),
            ("Gaps", &e.gaps),
            ("Unknowns", &e.unknowns),
            ("Assumptions", &e.assumptions),
        ] {
            text.push_str(&format!("\n{label}\n"));
            if values.is_empty() {
                text.push_str("—\n");
            } else {
                for value in values {
                    text.push_str(&format!("• {value}\n"));
                }
            }
        }
        text
    } else {
        format!(
            "{}\n\nSource: {} • {}\nURL: {}\nFetched: {}\nPosted: {}\nBudget: {}\nHours/week: {}    Estimated effort: {}\nSkills: {}\nDecision: {:?}    Job revision: {}\nApplication: {:?}    Submitted: {}    Reported: {}    Connects spent: {}\nIngestion history: {} records\n\nDescription (untrusted source data)\n{}\n\nA score is a decision aid. Feedback records your decision; it never submits a proposal.",
            job.job.title,
            job.job.provider,
            job.job.source_id.as_deref().unwrap_or("unknown ID"),
            job.job.url,
            job.fetched_at,
            job.job
                .posted_at
                .map(|v| v.to_rfc3339())
                .unwrap_or_else(|| "unknown".into()),
            job.job
                .budget
                .as_ref()
                .map(|v| format!("{} {} ({:?})", v.amount, v.currency, job.job.job_type))
                .unwrap_or_else(|| "unknown".into()),
            job.job
                .weekly_hours
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".into()),
            job.job
                .estimated_effort_hours
                .map(|v| format!("{v} hours, estimate"))
                .unwrap_or_else(|| "unknown".into()),
            job.job.skills.join(", "),
            job.decision,
            job.revision,
            job.application_status,
            job.submitted_at
                .map(|v| v.to_rfc3339())
                .unwrap_or_else(|| "unknown (not supplied)".into()),
            job.submission_reported_at
                .map(|v| v.to_rfc3339())
                .unwrap_or_else(|| "not reported".into()),
            job.connects_spent
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".into()),
            job.ingestions,
            job.job
                .description
                .as_deref()
                .unwrap_or("Unknown — full description unavailable")
        )
    };
    let text = if app
        .snapshot
        .as_ref()
        .is_some_and(|s| s.config.effective.revision != e.config_revision)
    {
        format!(
            "CONFIG CHANGED — this evaluation uses an earlier revision; re-import to rescore.\n\n{text}"
        )
    } else {
        text
    };
    let text = format!("{}\n{text}", ranking_text(job));
    paragraph(
        frame,
        area,
        if evaluation {
            "Evaluation Breakdown"
        } else {
            "Job Details"
        },
        text,
        app.scroll,
    );
}
fn render_config(frame: &mut Frame, app: &App, area: Rect) {
    let Some(draft) = &app.draft else {
        return paragraph(
            frame,
            area,
            "Configuration",
            "No server configuration available".into(),
            0,
        );
    };
    let items = config_fields(&draft.config)
        .into_iter()
        .map(|(label, _, value)| ListItem::new(format!("{label:<40} {value}")));
    let list = List::new(items.collect::<Vec<_>>())
        .block(panel(format!(
            "Configuration • base revision {} • {}",
            draft.revision,
            if draft.dirty {
                "UNSAVED DRAFT"
            } else {
                "server values"
            }
        )))
        .highlight_style(Style::default().bg(Color::Rgb(31, 45, 62)))
        .highlight_symbol("› ");
    let mut state = ListState::default().with_selected(Some(app.config_row));
    frame.render_stateful_widget(list, area, &mut state);
}
fn render_feedback(frame: &mut Frame, app: &App, area: Rect) {
    let Some(snapshot) = &app.snapshot else {
        return paragraph(
            frame,
            area,
            "Feedback History",
            "No feedback snapshot available".into(),
            0,
        );
    };
    let rows = snapshot.feedback.items.iter().map(|entry| {
        Row::new(vec![
            entry.created_at.format("%m-%d %H:%M").to_string(),
            format!("{:?}", entry.action),
            safe(&entry.job_title).replace('\n', " "),
            format!(
                "{:?}{} • {}",
                entry.reason_code,
                if entry.learning_excluded {
                    " [excluded]"
                } else {
                    ""
                },
                safe(entry.reason.as_deref().unwrap_or("—")).replace('\n', " ")
            ),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Percentage(40),
            Constraint::Min(10),
        ],
    )
    .header(Row::new(["UTC time", "Action", "Job", "Reason"]).style(Style::default().fg(MUTED)))
    .block(panel(format!(
        "Feedback History • {} events • page {}",
        snapshot.feedback.total, snapshot.feedback.page
    )))
    .row_highlight_style(Style::default().bg(Color::Rgb(31, 45, 62)))
    .highlight_symbol("› ");
    let mut state = TableState::default()
        .with_selected((!snapshot.feedback.items.is_empty()).then_some(app.feedback_row));
    frame.render_stateful_widget(table, area, &mut state);
}
fn render_input(frame: &mut Frame, input: &Input) {
    let area = frame.area();
    let width = area.width.saturating_sub(6).min(90);
    let height = area.height.saturating_sub(4).min(12);
    let rect = Rect::new(
        (area.width - width) / 2,
        (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, rect);
    let (title, text) = match input {
        Input::PreferenceReset(value) => (
            "Reset activation".into(),
            format!(
                "Type RESET to clear the active learned model. Jobs, feedback, drafts and model history are retained.\n\n{value}▏\nEnter confirm • Esc cancel"
            ),
        ),
        Input::LearningCorrection {
            code,
            excluded,
            hours,
            ..
        } => (
            "Correct learning feedback".into(),
            format!(
                "Reason: {:?}\nExcluded from learning: {}\nActual hours (completed outcomes only): {hours}▏\n\nCtrl+R reason • Ctrl+D exclusion • Ctrl+S save • Esc cancel\nThese attributes never rewrite the application outcome.",
                super::REASONS[*code],
                excluded
            ),
        ),
        Input::Proposal { text, approve, .. } => (
            "Edit proposal".into(),
            format!(
                "{text}▏\n\nFinal approval: [{}] • Ctrl+A toggle • Ctrl+S save • Enter newline • Esc cancel",
                if *approve { "yes" } else { "no" }
            ),
        ),
        Input::Search(value) => (
            "Search".into(),
            format!("{value}▏\n\nEnter apply • Esc cancel"),
        ),
        Input::ConfigValue { label, value, .. } => (
            format!("Edit {label}"),
            format!(
                "{value}▏\n\nBackspace to replace • Enter keep in draft • Esc cancel\nServer validates on save. Mode: employed / available / busy / paused."
            ),
        ),
        Input::Feedback {
            action,
            reason,
            confirmed,
            reason_code,
            ..
        } => (
            "Record feedback".into(),
            format!(
                "Action: {:?}\nReason code: {:?}\nExplanation: {reason}▏\n\nAlready submitted by me: [{}]\n↑↓ action • Ctrl+R reason code • type explanation • Ctrl+A confirm submission\nCtrl+S send • Esc cancel\nApplied is a human report, never a marketplace action.",
                ACTIONS[*action],
                super::REASONS[*reason_code],
                if *confirmed { "yes" } else { "no" }
            ),
        ),
    };
    frame.render_widget(
        Paragraph::new(safe(&text))
            .block(panel(title))
            .style(Style::default().bg(BG).fg(TEXT))
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn render_notifications(frame: &mut Frame, app: &App, area: Rect) {
    let Some(snapshot) = &app.snapshot else {
        return paragraph(
            frame,
            area,
            "Notifications",
            "No server state available".into(),
            0,
        );
    };
    let mut text =
        String::from("Provider readiness (configuration is not proof of a live delivery)\n");
    for provider in &snapshot.notifications.providers {
        text.push_str(&format!("{}: {}\n", provider.instance, provider.diagnostic));
    }
    text.push_str("\nDelivery history — newest 100\n");
    for delivery in &snapshot.notifications.deliveries {
        text.push_str(&format!(
            "[{}] {} → {} / {} • attempts {}\n{}\n{}\n",
            delivery.status,
            delivery.kind,
            delivery.provider,
            delivery.destination,
            delivery.attempts,
            delivery.title,
            delivery
                .delivered_at
                .map(|at| format!(
                    "Delivered {at}; remote {}",
                    delivery.remote_id.as_deref().unwrap_or("unknown")
                ))
                .unwrap_or_else(|| format!(
                    "Next attempt {}; {}",
                    delivery.next_attempt_at,
                    delivery.error.as_deref().unwrap_or("queued")
                ))
        ));
    }
    if snapshot.notifications.deliveries.is_empty() {
        text.push_str("No outgoing events yet. Queue a mock scan with m.\n");
    }
    paragraph(frame, area, "Notifications & Discord", text, app.scroll);
}

fn render_ai(frame: &mut Frame, app: &App, area: Rect) {
    let mut text = String::new();
    if let Some(job) = &app.selected {
        text.push_str(&format!(
            "Deterministic: {:?} / {:?} — authoritative\n",
            job.evaluation.verdict, job.evaluation.score
        ));
    }
    for op in app.ai.operations.iter().take(4) {
        text.push_str(&format!(
            "{:?}: {} • stale={} • reviewed={} • error={}\n",
            op.kind,
            op.status,
            op.stale,
            op.approved,
            op.error.as_deref().unwrap_or("none")
        ));
    }
    let selected = app.proposal_template_id.as_deref().or_else(|| {
        app.snapshot
            .as_ref()
            .and_then(|s| s.config.effective.radar.proposals.templates.as_ref())
            .and_then(|t| t.default_id.as_deref())
    });
    for operation in app
        .ai
        .operations
        .iter()
        .filter(|o| o.kind == radar_application::ai::TaskKind::Proposal)
        .take(1)
    {
        if let Some(template) = &operation.proposal_template {
            text.push_str(&format!(
                "Last draft template: {} • revision {}\n",
                template.definition.name,
                template.revision.get(..12).unwrap_or(&template.revision)
            ));
        }
    }
    if let Some(template) = selected.and_then(|id| {
        app.snapshot
            .as_ref()
            .and_then(|s| s.config.effective.radar.proposals.templates.as_ref())
            .and_then(|catalog| catalog.items.iter().find(|t| t.id == id))
    }) {
        text.push_str(&format!(
            "\nSelected template: {} • <= {} words\n{}\n",
            template.name, template.max_words, template.use_when
        ));
    }
    if app.proposal_view {
        if let Some(draft) = app.ai.drafts.first() {
            text.push_str(&format!(
                "\nProposal revision {} • approved={}\n\n{}\n\nAI original:\n{}",
                draft.revision,
                draft.approved_text.is_some(),
                draft.user_text.as_ref().unwrap_or(&draft.ai_text),
                draft.ai_text
            ));
            if let Some(output) = app
                .ai
                .operations
                .iter()
                .find(|op| op.id == draft.operation_id)
                .and_then(|op| op.output.clone())
                && let Ok(proposal) =
                    serde_json::from_value::<radar_application::ai::Proposal>(output)
            {
                for answer in proposal.screening_answers {
                    text.push_str(&format!(
                        "\n\nScreening question: {}\n{}",
                        answer.question, answer.answer
                    ));
                }
            }
        } else {
            text.push_str("\nNo proposal. Review an evaluation (b), then explicitly generate (G). No submission occurs.");
        }
    } else if let Some(op) = app
        .ai
        .operations
        .iter()
        .find(|o| o.kind == radar_application::ai::TaskKind::Evaluate && o.output.is_some())
    {
        if let Ok(analysis) =
            serde_json::from_value::<radar_application::ai::Analysis>(op.output.clone().unwrap())
        {
            text.push_str(&format!("\nAI recommendation: {:?} • confidence: {:?}\n{}\n\nStrengths: {}\nGaps: {}\nRisks: {}\nUnknowns: {}\nAssumptions: {}\nEffort: {:?}–{:?} hours (advisory)\nDependencies: {}\nPricing: {}\nNext: {}\n",analysis.recommendation,analysis.confidence,analysis.technical_fit,analysis.strengths.join("; "),analysis.gaps.join("; "),analysis.risks.join("; "),analysis.unknowns.join("; "),analysis.assumptions.join("; "),analysis.estimated_effort.best_case_hours,analysis.estimated_effort.conservative_hours,analysis.estimated_effort.dependencies.join("; "),analysis.pricing_considerations.join("; "),analysis.suggested_next_action));
            for fact in &op.evidence.facts {
                text.push_str(&format!(
                    "Evidence {} [{}; {:?}]: {}\n",
                    fact.id, fact.source, fact.context, fact.statement
                ));
            }
        }
    } else {
        text.push_str("\nNo completed AI evaluation. e evaluates explicitly; r refreshes status. AI may be disabled in configuration.");
    }
    paragraph(
        frame,
        area,
        if app.proposal_view {
            "Proposal review"
        } else {
            "AI assessment / evidence"
        },
        text,
        app.scroll,
    );
}

fn ranking_text(job: &radar_application::api::JobView) -> String {
    let Some(ranking) = &job.personalization else {
        return String::new();
    };
    let mut text = format!(
        "Base: {:?} • Personalization: {:+.2} • Final ranking: {:?}\nEvidence: {} jobs • confidence {:.0}% (not win probability) • updated {:?}\n{}\n",
        ranking.base_score,
        ranking.adjustment,
        ranking.final_score,
        ranking.evidence_count,
        ranking.confidence * 100.0,
        ranking.model_updated_at,
        ranking.diagnostic
    );
    for contribution in &ranking.contributions {
        text.push_str(&format!(
            "{:+.2} {} ({} jobs)\n",
            contribution.points, contribution.feature, contribution.evidence_count
        ));
    }
    if ranking.raw_adjustment != ranking.adjustment {
        text.push_str(&format!(
            "Raw contributions {:+.2}; cap/score ceiling applied.\n",
            ranking.raw_adjustment
        ));
    }
    text
}
fn render_preferences(frame: &mut Frame, app: &App, area: Rect) {
    let Some(snapshot) = &app.snapshot else {
        return paragraph(
            frame,
            area,
            "Preference learning",
            "Refresh to load server state".into(),
            0,
        );
    };
    let view = &snapshot.preferences;
    let mut text = format!(
        "Enabled: {} • active: {:?} • stale: {} • control revision: {}\nSuppressed: {}\n\nModels (↑↓ select; no automatic activation)\n",
        view.enabled,
        view.active_model,
        view.stale,
        view.revision,
        view.suppressed
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
    for (index, model) in view
        .models
        .iter()
        .enumerate()
        .skip(app.model_row.saturating_sub(5))
        .take(12)
    {
        text.push_str(&format!(
            "{} {} • {} • {} jobs • rejected={}\n",
            if index == app.model_row { ">" } else { " " },
            model.created_at.format("%m-%d %H:%M"),
            model.id,
            model.samples,
            view.rejected.contains(&model.id)
        ));
    }
    if let Some(model) = view.models.get(app.model_row) {
        text.push_str(&format!(
            "\n{} • cutoff {} • feature version {}\nFeature / points / evidence / confidence\n",
            model.version, model.cutoff, model.feature_version
        ));
        for (index, (feature, weight)) in model
            .weights
            .iter()
            .enumerate()
            .skip(app.feature_row.saturating_sub(5))
            .take(12)
        {
            text.push_str(&format!(
                "{} {} {:+.2} • n={} (+{}/-{}) • {:.0}%\n",
                if index == app.feature_row { ">" } else { " " },
                feature,
                weight.points,
                weight.evidence_count,
                weight.positive,
                weight.negative,
                weight.confidence * 100.0
            ));
        }
        text.push_str(&format!("\nBacktest: {} train / {} held-out jobs • baseline P@K {:?} • candidate P@K {:?}\n{}\n",model.backtest.training_samples,model.backtest.held_out_jobs,model.backtest.baseline_precision_at_k,model.backtest.candidate_precision_at_k,model.backtest.diagnostic));
    }
    if let Some(comparison) = &app.learning_review.comparison {
        text.push_str("\nShadow ranking (no activation)\n");
        for (index, row) in comparison.candidate.iter().take(10).enumerate() {
            let old = comparison
                .baseline
                .iter()
                .position(|old| old.job_id == row.job_id)
                .unwrap_or(index);
            text.push_str(&format!(
                "{}→{} {} • {:?} {:+.2} → {:?}\n",
                old + 1,
                index + 1,
                row.title,
                row.explanation.base_score,
                row.explanation.adjustment,
                row.explanation.final_score
            ));
        }
    }
    if let Some(calibration) = &app.learning_review.calibration {
        text.push_str(&format!("\nCalibration: {} explicit jobs / {} AI comparisons / {} disagreements / {} effort comparisons\n",calibration.explicit_jobs,calibration.ai_comparisons,calibration.ai_owner_disagreements,calibration.effort_comparisons));
        for suggestion in &calibration.suggestions {
            text.push_str(&format!("Review only: {suggestion}\n"));
        }
    }
    paragraph(
        frame,
        area,
        "Preference models / shadow / calibration",
        text,
        app.scroll,
    );
}
