use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, Event, EventStream, KeyEventKind,
};
use futures_util::StreamExt;
use radar_tui::{
    client::{ApiClient, execute},
    ui::{App, Effect, NetworkResult, UiRequest, render},
};
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(about = "Personal Freelance Radar — authenticated terminal client")]
struct Cli {
    #[arg(long)]
    api_url: Option<String>,
    /// Development only: allow HTTP to a loopback address. Remote endpoints require HTTPS.
    #[arg(long)]
    allow_local_http: bool,
    /// Authenticate and read server state without entering the terminal UI.
    #[arg(long)]
    check: bool,
}
struct Restore;
impl Drop for Restore {
    fn drop(&mut self) {
        let _ = crossterm::execute!(std::io::stdout(), DisableBracketedPaste);
        ratatui::restore();
    }
}
#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("radar-tui: {error:#}");
        std::process::exit(1);
    }
}
fn start(
    client: ApiClient,
    app: &mut App,
    request: UiRequest,
    sender: mpsc::Sender<(UiRequest, NetworkResult)>,
) {
    app.busy = true;
    app.status = "Synchronizing with server…".into();
    let jobs = app.jobs.clone();
    let feedback = app.feedback.clone();
    let selected = app.selected.as_ref().map(|j| j.id);
    tokio::spawn(async move {
        let result = execute(&client, &request, &jobs, &feedback, selected).await;
        let _ = sender.send((request, result)).await;
    });
}
async fn run() -> Result<()> {
    let cli = Cli::parse();
    let url = cli
        .api_url
        .or_else(|| std::env::var("RADAR_API_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:8080".into());
    let token = std::env::var("RADAR_API_TOKEN").context("RADAR_API_TOKEN must be set")?;
    let client = ApiClient::new(&url, &token, cli.allow_local_http)?;
    if cli.check {
        let snapshot = client
            .snapshot(&Default::default(), &Default::default())
            .await?;
        println!(
            "Authenticated • jobs={} • feedback={} • config_revision={} • mode={:?}",
            snapshot.jobs.total,
            snapshot.feedback.total,
            snapshot.config.revision,
            snapshot.dashboard.mode
        );
        println!(
            "Scheduler: enabled={} active_workers={} next_scan={}",
            snapshot.scheduler.enabled,
            snapshot.scheduler.active_workers,
            snapshot
                .scheduler
                .next_scan
                .map(|at| at.to_rfc3339())
                .unwrap_or_else(|| "none".into())
        );
        println!(
            "Notification history: {} deliveries",
            snapshot.notifications.deliveries.len()
        );
        println!(
            "Upwork: enabled={} • {} • {} • fetched={} new={} duplicates={} accepted={}",
            snapshot.upwork.enabled,
            snapshot.upwork.execution_mode,
            snapshot.upwork.authentication,
            snapshot.upwork.fetched,
            snapshot.upwork.new_jobs,
            snapshot.upwork.duplicates,
            snapshot.upwork.accepted
        );
        for provider in &snapshot.notifications.providers {
            println!("{}: {}", provider.instance, provider.diagnostic);
        }
        return Ok(());
    }
    let mut terminal = ratatui::init();
    let _restore = Restore;
    crossterm::execute!(std::io::stdout(), EnableBracketedPaste)?;
    let mut app = App::default();
    let (sender, mut receiver) = mpsc::channel(4);
    let mut events = EventStream::new();
    start(client.clone(), &mut app, UiRequest::Refresh, sender.clone());
    loop {
        terminal.draw(|frame| render(frame, &app))?;
        let refresh = app
            .snapshot
            .as_ref()
            .and_then(|s| s.config.effective.radar.scheduler.as_ref())
            .map(|s| s.ui_refresh_seconds);
        tokio::select! {
            event=events.next()=>match event {
                Some(Ok(Event::Key(key))) if key.kind==KeyEventKind::Press=>match app.handle_key(key) {
                    Effect::Quit=>break,
                    Effect::Request(request)=>start(client.clone(),&mut app,request,sender.clone()),
                    Effect::Copy(text)=>{
                        use base64::Engine;
                        use std::io::Write;
                        let encoded=base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
                        write!(std::io::stdout(),"\x1b]52;c;{encoded}\x07")?;
                        std::io::stdout().flush()?;
                        app.status="Clipboard request sent; terminal support/permission required".into();
                    },
                    Effect::None=>{},
                },
                Some(Ok(Event::Paste(text)))=>app.paste_profile(&text),
                Some(Err(error))=>return Err(error.into()),None=>break,_=>{},
            },
            Some((request,result))=receiver.recv()=>app.complete(request,result),
            _=tokio::signal::ctrl_c()=>break,
            _=tokio::time::sleep(std::time::Duration::from_secs(refresh.unwrap_or(60))),if refresh.is_some()&&!app.busy&&app.input.is_none()&&!app.profile_editor.editing()=>start(client.clone(),&mut app,UiRequest::Refresh,sender.clone()),
        }
    }
    Ok(())
}
