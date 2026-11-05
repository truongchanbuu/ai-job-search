use super::*;
use radar_application::profiles::{MODULES, ProfileUpdate, ProfileView};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use serde_json::{Value, json};

#[derive(Default)]
pub struct Editor {
    pub draft: Option<ProfileUpdate>,
    pub dirty: bool,
    pub section: usize,
    pub row: usize,
    pub preview: Option<EvidenceSnapshot>,
    form: Option<Form>,
}
struct Form {
    target: Target,
    fields: Vec<(String, String)>,
    cursor: usize,
}
enum Target {
    Skill(Option<String>),
    Fact(Option<usize>),
    Prose,
    Preferences,
    Confirm,
    RemoveSkill(String),
    RemoveFact(usize),
}
fn metadata(text: &str) -> Value {
    text.strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n"))
        .and_then(|(yaml, _)| serde_yaml::from_str(yaml).ok())
        .unwrap_or(json!({}))
}
fn prose(text: &str) -> &str {
    text.strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n"))
        .map_or(text, |(_, body)| body)
}
fn set_metadata(text: &str, value: &Value) -> Result<String, String> {
    Ok(format!(
        "---\n{}---\n{}",
        serde_yaml::to_string(value).map_err(|_| "Unable to serialize profile")?,
        prose(text)
    ))
}
impl Editor {
    pub fn editing(&self) -> bool {
        self.form.is_some()
    }
    pub fn load(&mut self, view: &ProfileView) {
        self.draft = Some(ProfileUpdate {
            expected_revision: view.revision,
            documents: view
                .profile
                .documents
                .iter()
                .map(|(k, v)| (k.clone(), v.replace("\r\n", "\n")))
                .collect(),
            confirmed_facts: false,
        });
        self.dirty = false;
        self.preview = None;
    }
    fn module(&self) -> &'static str {
        MODULES[self.section]
    }
    fn rows(&self) -> Vec<(String, String)> {
        let Some(draft) = &self.draft else {
            return vec![];
        };
        let meta = metadata(&draft.documents[self.module()]);
        match self.module() {
            "identity" => meta["skills"]
                .as_object()
                .map(|skills| {
                    skills
                        .iter()
                        .map(|(name, e)| {
                            (
                                name.clone(),
                                format!(
                                    "{} • {}",
                                    e["context"].as_str().unwrap_or("unknown"),
                                    e["proof"].as_str().unwrap_or("")
                                ),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
            "portfolio" => meta["evidence"]
                .as_array()
                .map(|facts| {
                    facts
                        .iter()
                        .map(|f| {
                            (
                                f["id"].as_str().unwrap_or("").into(),
                                format!(
                                    "{} • verified={} • shareable={}\n{}",
                                    f["context"].as_str().unwrap_or("unknown"),
                                    f["verified"],
                                    f["shareable"],
                                    f["statement"].as_str().unwrap_or("")
                                ),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
            _ => vec![],
        }
    }
    fn open(&mut self, new: bool) {
        let Some(draft) = &self.draft else { return };
        let meta = metadata(&draft.documents[self.module()]);
        let (target, fields) = match self.module() {
            "identity" => {
                let key = (!new)
                    .then(|| self.rows().get(self.row).map(|r| r.0.clone()))
                    .flatten();
                let entry = key.as_ref().and_then(|k| meta["skills"].get(k));
                (
                    Target::Skill(key.clone()),
                    vec![
                        ("Skill".into(), key.unwrap_or_default()),
                        (
                            "Proof / where used".into(),
                            entry.and_then(|v| v["proof"].as_str()).unwrap_or("").into(),
                        ),
                        (
                            "Context".into(),
                            entry
                                .and_then(|v| v["context"].as_str())
                                .unwrap_or("personal")
                                .into(),
                        ),
                    ],
                )
            }
            "portfolio" => {
                let index = (!new)
                    .then_some(self.row)
                    .filter(|i| meta["evidence"].get(*i).is_some());
                let entry = index
                    .and_then(|i| meta["evidence"].get(i))
                    .cloned()
                    .unwrap_or(json!({}));
                (
                    Target::Fact(index),
                    [
                        ("Evidence ID", "id", ""),
                        ("Section reference", "source", "portfolio.md#project"),
                        ("Context", "context", "personal"),
                        ("Safe factual statement", "statement", ""),
                        ("Verified", "verified", "false"),
                        ("Shareable", "shareable", "false"),
                        ("Public HTTPS URL (optional)", "portfolio_url", ""),
                    ]
                    .into_iter()
                    .map(|(label, key, default)| {
                        (
                            label.into(),
                            entry[key]
                                .as_str()
                                .map(str::to_owned)
                                .or_else(|| entry[key].as_bool().map(|b| b.to_string()))
                                .unwrap_or(default.into()),
                        )
                    })
                    .collect(),
                )
            }
            "job-preferences" => (
                Target::Preferences,
                vec![
                    (
                        "Preferred categories (comma separated)".into(),
                        meta["preferred_categories"]
                            .as_array()
                            .map(|v| {
                                v.iter()
                                    .filter_map(Value::as_str)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                            .unwrap_or_default(),
                    ),
                    (
                        "Blocked categories (comma separated)".into(),
                        meta["blocked_categories"]
                            .as_array()
                            .map(|v| {
                                v.iter()
                                    .filter_map(Value::as_str)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                            .unwrap_or_default(),
                    ),
                    (
                        "Minimum hourly USD".into(),
                        meta["minimum_hourly_usd"].to_string(),
                    ),
                ],
            ),
            _ => (
                Target::Prose,
                vec![(
                    "Content".into(),
                    prose(&draft.documents[self.module()]).into(),
                )],
            ),
        };
        self.form = Some(Form {
            target,
            fields,
            cursor: 0,
        });
    }
    fn apply(&mut self) -> Result<Option<UiRequest>, String> {
        let form = self.form.as_ref().ok_or("No form")?;
        let v: Vec<_> = form
            .fields
            .iter()
            .map(|(_, v)| v.trim().to_owned())
            .collect();
        let module = self.module();
        let draft = self.draft.as_mut().ok_or("Profile not loaded")?;
        if matches!(form.target, Target::Confirm) {
            if v[0] != "true" {
                return Err(
                    "Confirm your facts and sharing permissions with Space before saving".into(),
                );
            }
            let mut request = draft.clone();
            request.confirmed_facts = true;
            self.form = None;
            return Ok(Some(UiRequest::SaveProfile(request)));
        }
        let text = &draft.documents[module];
        let mut meta = metadata(text);
        match &form.target {
            Target::Skill(old) => {
                if v[0].is_empty() || v[1].is_empty() {
                    return Err("Skill and verified proof are required".into());
                }
                let key = v[0].to_lowercase();
                if old.as_ref() != Some(&key) && meta["skills"].get(&key).is_some() {
                    return Err("This skill already exists".into());
                }
                if let Some(old) = old {
                    meta["skills"]
                        .as_object_mut()
                        .ok_or("Invalid skills")?
                        .remove(old);
                }
                meta["skills"][&key] = json!({"proof":v[1],"context":v[2]});
                draft
                    .documents
                    .insert(module.into(), set_metadata(text, &meta)?);
            }
            Target::Fact(index) => {
                if v[0].is_empty() || v[3].is_empty() {
                    return Err("Evidence ID and statement are required".into());
                }
                if meta.get("evidence").is_none() {
                    meta["evidence"] = json!([]);
                }
                let facts = meta["evidence"]
                    .as_array_mut()
                    .ok_or("Invalid evidence catalog")?;
                if facts
                    .iter()
                    .enumerate()
                    .any(|(i, f)| Some(i) != *index && f["id"] == v[0])
                {
                    return Err("Evidence ID already exists".into());
                }
                let fact = json!({"id":v[0],"source":v[1],"context":v[2],"statement":v[3],"verified":v[4]=="true","shareable":v[5]=="true","portfolio_url":if v[6].is_empty(){Value::Null}else{json!(v[6])}});
                if let Some(i) = index {
                    facts[*i] = fact
                } else {
                    facts.push(fact)
                }
                draft
                    .documents
                    .insert(module.into(), set_metadata(text, &meta)?);
            }
            Target::Preferences => {
                let split = |s: &str| {
                    s.split(',')
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                };
                let rate = v[2]
                    .parse::<f64>()
                    .map_err(|_| "Enter a positive numeric hourly rate")?;
                if !rate.is_finite() || rate <= 0.0 {
                    return Err("Enter a positive numeric hourly rate".into());
                }
                meta = json!({"preferred_categories":split(&v[0]),"blocked_categories":split(&v[1]),"minimum_hourly_usd":rate});
                draft
                    .documents
                    .insert(module.into(), set_metadata(text, &meta)?);
            }
            Target::Prose => {
                let prefix = text.strip_suffix(prose(text)).unwrap_or("");
                draft
                    .documents
                    .insert(module.into(), format!("{prefix}{}", form.fields[0].1));
            }
            Target::RemoveSkill(key) => {
                if v[0] != "true" {
                    return Err("Confirm removal with Space".into());
                }
                let skills = meta["skills"].as_object_mut().ok_or("Invalid skills")?;
                if skills.len() <= 1 {
                    return Err("Keep at least one verified skill".into());
                }
                skills.remove(key);
                draft
                    .documents
                    .insert(module.into(), set_metadata(text, &meta)?);
            }
            Target::RemoveFact(index) => {
                if v[0] != "true" {
                    return Err("Confirm removal with Space".into());
                }
                let facts = meta["evidence"].as_array_mut().ok_or("Invalid evidence")?;
                if *index < facts.len() {
                    facts.remove(*index);
                }
                draft
                    .documents
                    .insert(module.into(), set_metadata(text, &meta)?);
            }
            Target::Confirm => {}
        }
        self.form = None;
        self.dirty = true;
        self.preview = None;
        Ok(None)
    }
}
impl App {
    pub fn paste_profile(&mut self, text: &str) {
        if self.screen != Screen::Profiles || self.busy {
            return;
        }
        if let Some(form) = &mut self.profile_editor.form {
            let (label, value) = &mut form.fields[form.cursor];
            if matches!(
                label.as_str(),
                "Context" | "Verified" | "Shareable" | "Confirm facts & sharing permissions"
            ) {
                return;
            }
            for c in text
                .chars()
                .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
            {
                if value.len() + c.len_utf8() > 32000 {
                    break;
                }
                value.push(c);
            }
        }
    }
    pub(super) fn profile_key(&mut self, key: KeyEvent) -> Option<Effect> {
        let editor = &mut self.profile_editor;
        if editor.form.is_some() {
            if key.code == KeyCode::Esc {
                editor.form = None;
                return Some(Effect::None);
            }
            if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Some(match editor.apply() {
                    Ok(Some(r)) => self.request(r),
                    Ok(None) => {
                        self.status =
                            "Profile draft updated locally; v previews, s reviews and saves".into();
                        Effect::None
                    }
                    Err(error) => {
                        self.status = error;
                        Effect::None
                    }
                });
            }
            let form = editor.form.as_mut().unwrap();
            match key.code {
                KeyCode::Tab | KeyCode::Down => form.cursor = (form.cursor + 1) % form.fields.len(),
                KeyCode::BackTab | KeyCode::Up => {
                    form.cursor = form.cursor.checked_sub(1).unwrap_or(form.fields.len() - 1)
                }
                code => {
                    let (label, value) = &mut form.fields[form.cursor];
                    if code == KeyCode::Char('u') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        value.clear();
                        return Some(Effect::None);
                    }
                    if matches!(
                        label.as_str(),
                        "Context"
                            | "Verified"
                            | "Shareable"
                            | "Confirm facts & sharing permissions"
                            | "Confirm removal"
                    ) {
                        if code == KeyCode::Char(' ') {
                            if label == "Context" {
                                let contexts =
                                    ["personal", "academic", "training", "employment", "contract"];
                                let i = contexts.iter().position(|c| *c == value).unwrap_or(0);
                                *value = contexts[(i + 1) % contexts.len()].into()
                            } else {
                                *value = if value == "true" { "false" } else { "true" }.into()
                            }
                        }
                    } else {
                        match code {
                            KeyCode::Backspace => {
                                value.pop();
                            }
                            KeyCode::Enter if matches!(form.target, Target::Prose) => {
                                if value.len() < 32000 {
                                    value.push('\n')
                                }
                            }
                            KeyCode::Char(c)
                                if !c.is_control()
                                    && !key.modifiers.contains(KeyModifiers::CONTROL)
                                    && value.len() + c.len_utf8() <= 32000 =>
                            {
                                value.push(c)
                            }
                            _ => {}
                        }
                    }
                }
            }
            return Some(Effect::None);
        }
        match key.code {
            KeyCode::Char('[') => {
                self.scroll = 0;
                editor.section = editor.section.saturating_sub(1);
                editor.row = 0;
                editor.preview = None;
            }
            KeyCode::Char(']') => {
                self.scroll = 0;
                editor.section = (editor.section + 1).min(MODULES.len() - 1);
                editor.row = 0;
                editor.preview = None;
            }
            KeyCode::Up => editor.row = editor.row.saturating_sub(1),
            KeyCode::Down => {
                editor.row = (editor.row + 1).min(editor.rows().len().saturating_sub(1))
            }
            KeyCode::Delete => {
                if let Some((key, _)) = editor.rows().get(editor.row) {
                    let target = if editor.module() == "identity" {
                        Target::RemoveSkill(key.clone())
                    } else {
                        Target::RemoveFact(editor.row)
                    };
                    editor.form = Some(Form {
                        target,
                        fields: vec![("Confirm removal".into(), "false".into())],
                        cursor: 0,
                    });
                }
            }
            KeyCode::Enter | KeyCode::Char('e') => editor.open(false),
            KeyCode::Char('n') => editor.open(true),
            KeyCode::Char('p') => {
                if let Some(draft) = &editor.draft {
                    editor.form = Some(Form {
                        target: Target::Prose,
                        fields: vec![(
                            "Content".into(),
                            prose(&draft.documents[editor.module()]).into(),
                        )],
                        cursor: 0,
                    })
                }
            }
            KeyCode::Char('s') => {
                editor.form = Some(Form {
                    target: Target::Confirm,
                    fields: vec![("Confirm facts & sharing permissions".into(), "false".into())],
                    cursor: 0,
                })
            }
            KeyCode::Char('v') => {
                if let Some(draft) = &editor.draft {
                    let request = UiRequest::PreviewProfile(draft.clone());
                    self.scroll = 0;
                    return Some(self.request(request));
                }
            }
            KeyCode::Char('d') => {
                if let Some(snapshot) = &self.snapshot {
                    editor.load(&snapshot.profile)
                }
            }
            _ => return None,
        }
        Some(Effect::None)
    }
}
fn panel(title: &str) -> Block<'_> {
    Block::default().borders(Borders::ALL).title(title).style(
        Style::default()
            .fg(Color::Rgb(211, 219, 230))
            .bg(Color::Rgb(16, 21, 29)),
    )
}
fn clean(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .collect()
}
pub(super) fn render(frame: &mut Frame, app: &App, area: Rect) {
    let editor = &app.profile_editor;
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(20)])
        .split(area);
    let names = [
        "CV & skills",
        "Mindset",
        "Availability",
        "Job preferences",
        "Portfolio evidence",
        "Writing style",
        "Lessons learned",
    ];
    let items = names
        .iter()
        .map(|name| ratatui::widgets::ListItem::new(format!("{name}\n")))
        .collect::<Vec<_>>();
    let navigation = ratatui::widgets::List::new(items)
        .block(panel("Profile sections • [ ]"))
        .highlight_symbol("› ")
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(31, 45, 62))
                .fg(Color::Rgb(133, 181, 233)),
        );
    let mut selection = ratatui::widgets::ListState::default().with_selected(Some(editor.section));
    frame.render_stateful_widget(navigation, columns[0], &mut selection);
    let mut content = String::new();
    if let Some(view) = app.snapshot.as_ref().map(|s| &s.profile) {
        content.push_str(&format!(
            "{} • version {} • {}\n\n",
            if view.source == "db-approved" {
                "Cloud profile"
            } else {
                "Private Markdown profile"
            },
            view.revision,
            if editor.dirty {
                "Unsaved draft"
            } else {
                "Saved"
            }
        ));
    }
    if let Some(preview) = &editor.preview {
        content.push_str(&format!("PREVIEW • {} hours/week from runtime settings • minimum ${}/hour\nOnly verified facts below enter AI evaluation. Only shareable facts enter proposals.\n\n",preview.weekly_hours,preview.minimum_hourly_usd));
        content.push_str(&format!(
            "Preferred: {}\nBlocked: {}\nWriting style supplied to AI:\n{}\n\n",
            preview.preferred_categories.join(", "),
            preview.blocked_categories.join(", "),
            preview.writing_style
        ));
        for fact in &preview.facts {
            content.push_str(&format!(
                "{} • {:?} • shareable={}\n{}\n\n",
                fact.id, fact.context, fact.shareable, fact.statement
            ));
        }
    } else {
        for (i, (title, body)) in editor.rows().iter().enumerate() {
            content.push_str(&format!(
                "{} {title}\n  {body}\n\n",
                if i == editor.row { "›" } else { " " }
            ));
        }
        if let Some(draft) = &editor.draft {
            content.push_str(prose(&draft.documents[editor.module()]));
        }
    }
    frame.render_widget(
        Paragraph::new(clean(&content))
            .block(panel(names[editor.section]))
            .wrap(Wrap { trim: false })
            .scroll((app.scroll, 0)),
        columns[1],
    );
    if let Some(form) = &editor.form {
        let rect = Rect::new(
            area.x + 2,
            area.y + 1,
            area.width.saturating_sub(4),
            area.height.saturating_sub(2),
        );
        frame.render_widget(Clear, rect);
        let mut text = String::new();
        let mut focus_line = 0usize;
        let width = rect.width.saturating_sub(2).max(1) as usize;
        for (i, (label, value)) in form.fields.iter().enumerate() {
            text.push_str(&format!(
                "{} {label}\n{}{}\n\n",
                if i == form.cursor { "›" } else { " " },
                value,
                if i == form.cursor { "▏" } else { "" }
            ));
            if i == form.cursor {
                focus_line = text
                    .lines()
                    .map(|line| line.chars().count().max(1).div_ceil(width))
                    .sum();
            }
        }
        let scroll = focus_line
            .saturating_sub(rect.height.saturating_sub(7) as usize)
            .min(u16::MAX as usize) as u16;
        text.push_str("Tab/Shift+Tab field • Ctrl+U clear • paste supported • Space toggles choices\nCtrl+S accept • Esc cancel. Saved profile applies to future evaluations.\nAvailability prose is descriptive; numeric capacity remains in Configuration.");
        frame.render_widget(
            Paragraph::new(clean(&text))
                .block(panel(&format!(
                    "Profile form • {}",
                    form.fields[form.cursor].0
                )))
                .wrap(Wrap { trim: false })
                .scroll((scroll, 0)),
            rect,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn editor() -> Editor {
        let root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/profiles");
        Editor {
            draft: Some(ProfileUpdate {
                expected_revision: 0,
                documents: MODULES
                    .iter()
                    .map(|name| {
                        (
                            name.to_string(),
                            std::fs::read_to_string(root.join(format!("{name}.md"))).unwrap(),
                        )
                    })
                    .collect(),
                confirmed_facts: false,
            }),
            ..Default::default()
        }
    }
    #[test]
    fn skill_and_portfolio_forms_preserve_prose_and_require_explicit_sharing() {
        let mut editor = editor();
        let original = prose(&editor.draft.as_ref().unwrap().documents["identity"]).to_owned();
        editor.open(true);
        let form = editor.form.as_mut().unwrap();
        form.fields[0].1 = "Python".into();
        form.fields[1].1 = "Synthetic personal API project".into();
        assert!(editor.apply().unwrap().is_none());
        assert!(editor.dirty);
        let text = &editor.draft.as_ref().unwrap().documents["identity"];
        assert_eq!(metadata(text)["skills"]["python"]["context"], "personal");
        assert_eq!(prose(text), original);
        editor.section = 4;
        editor.open(true);
        let form = editor.form.as_mut().unwrap();
        assert_eq!(form.fields[4].1, "false");
        assert_eq!(form.fields[5].1, "false");
        form.fields[0].1 = "synthetic-project".into();
        form.fields[3].1 = "Personal API training project".into();
        editor.apply().unwrap();
        let facts = metadata(&editor.draft.as_ref().unwrap().documents["portfolio"]);
        assert_eq!(
            facts["evidence"].as_array().unwrap().last().unwrap()["shareable"],
            false
        );
    }
    #[test]
    fn confirmation_and_invalid_rates_do_not_save_and_paste_is_bounded() {
        let mut editor = editor();
        editor.section = 3;
        editor.open(false);
        editor.form.as_mut().unwrap().fields[2].1 = "NaN".into();
        assert!(editor.apply().is_err());
        editor.form = Some(Form {
            target: Target::Confirm,
            fields: vec![("Confirm facts & sharing permissions".into(), "false".into())],
            cursor: 0,
        });
        assert!(editor.apply().is_err());
        editor.form.as_mut().unwrap().fields[0].1 = "true".into();
        assert!(matches!(
            editor.apply().unwrap(),
            Some(UiRequest::SaveProfile(ProfileUpdate {
                confirmed_facts: true,
                ..
            }))
        ));
        editor.section = 2;
        editor.open(false);
        let mut app = App {
            screen: Screen::Profiles,
            profile_editor: editor,
            ..Default::default()
        };
        app.paste_profile(&format!("\x1b{}", "é".repeat(40000)));
        let value = &app.profile_editor.form.as_ref().unwrap().fields[0].1;
        assert!(value.len() <= 32000);
        assert!(!value.contains('\x1b'));
        app.profile_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert!(
            app.profile_editor.form.as_ref().unwrap().fields[0]
                .1
                .is_empty()
        );
    }
}
