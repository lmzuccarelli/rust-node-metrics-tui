use crate::config::load::Parameters;
use crate::handlers::process::{MetricsInterface, Service};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::Flex;
use ratatui::widgets::ListState;
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};
use std::{env, io};

#[derive(Debug, Clone)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        let mut st = ListState::default();
        // set first item as selected
        st.select(Some(0));
        Self { state: st, items }
    }

    pub fn next(&mut self) {
        if self.items.len() > 0 {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if self.items.len() > 0 {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }
}

/// set up the app state for the ui
// keep the schema and api in the same module
pub struct App {
    pub name: String,
    pub nodes: StatefulList<String>,
    pub cpu: Vec<String>,
    pub memory: Vec<String>,
    pub network: Vec<String>,
    pub disk: Vec<String>,
    pub info: Vec<String>,
    pub scrape_duration: u64,
    pub show_popup: bool,
}

impl App {
    pub fn new(name: String, params: Parameters) -> Self {
        let title = format!("[ {} ]", name);
        Self {
            name: title.clone(),
            nodes: StatefulList::with_items(params.servers),
            cpu: vec![],
            memory: vec![],
            network: vec![],
            disk: vec![],
            info: vec![],
            scrape_duration: params.scrape_duration,
            show_popup: false,
        }
    }
}

/// run the app (event loop)
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_secs(app.scrape_duration);
    let mut last_tick = Instant::now();
    let mut svc = Service::new();
    let mut changed = false;

    // get all metrics on startup
    let res_vec_metrics = svc.scrape(app.nodes.items[0].clone()).await;
    match res_vec_metrics {
        Ok(metrics) => {
            let res_data = svc.get_all_metrics(metrics);
            match res_data {
                Ok(data) => {
                    app.cpu = data.cpu;
                    app.memory = data.memory;
                    app.network = data.network;
                    app.disk = data.disk;
                    app.info = data.info;
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }

    loop {
        terminal.draw(|f| render_ui(f, app))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        // event handling
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => return Ok(()),
                        Down => {
                            app.nodes.next();
                            app.show_popup = false;
                            changed = true;
                        }
                        Up => {
                            app.nodes.previous();
                            app.show_popup = false;
                            changed = true;
                        }
                        Char('p') => {
                            app.show_popup = !app.show_popup;
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate || changed {
            let selected_id = app.nodes.state.selected().unwrap();
            let node = app.nodes.items[selected_id].to_string();
            let res_vec_metrics = svc.scrape(node.clone()).await;
            match res_vec_metrics {
                Ok(metrics) => {
                    let res_data = svc.get_all_metrics(metrics);
                    match res_data {
                        Ok(data) => {
                            app.cpu = data.cpu;
                            app.memory = data.memory;
                            app.network = data.network;
                            app.disk = data.disk;
                            app.info = data.info;
                        }
                        Err(_) => {}
                    }
                }
                Err(_) => {}
            }
            changed = false;
        }
        last_tick = Instant::now();
    }
}

/// ui rendering
pub fn render_ui(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(size);

    let title = Paragraph::new(app.name.as_str())
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("")
                .border_type(BorderType::Plain),
        );
    frame.render_widget(title, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    let (node, cpu, memory, network, disk) = render_complex_view(app);
    frame.render_stateful_widget(node, body[0], &mut app.nodes.state.clone());
    frame.render_widget(cpu, body[1]);
    frame.render_widget(memory, body[2]);
    frame.render_widget(network, body[3]);
    frame.render_widget(disk, body[4]);

    let version = env!["CARGO_PKG_VERSION"];
    let name = env!["CARGO_PKG_NAME"];
    let title = format!(
        "{} {} 2025 [ use ▲ ▼  to change node, p to toggle node details popup, q to quit ]",
        name, version
    );

    let copyright = Paragraph::new(title.clone())
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("info")
                .border_type(BorderType::Plain),
        );
    frame.render_widget(copyright, chunks[2]);

    // prepare popup rendering
    if app.show_popup {
        let index = app.nodes.state.selected().unwrap();
        let name = app.nodes.items[index].clone();
        let info = format!(
            "\n{}",
            app.info
                .join("\n")
                .to_string()
                .replace("{", " ")
                .replace("}", "")
        );
        let paragraph = Paragraph::new(info)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .style(Style::default().fg(Color::White))
                    .borders(Borders::ALL)
                    .title(format!("node info [{}] ", name))
                    .border_type(BorderType::Plain),
            );
        let area = popup_area(size, 35, 45);
        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }
}

/// render the complex view
fn render_complex_view<'a>(app: &mut App) -> (List<'a>, List<'a>, List<'a>, List<'a>, List<'a>) {
    let nodes = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("nodes")
        .border_type(BorderType::Plain);

    let cpu = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("cpu")
        .border_type(BorderType::Plain);

    let memory = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("memory")
        .border_type(BorderType::Plain);

    let network = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("network")
        .border_type(BorderType::Plain);

    let disk = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("disk")
        .border_type(BorderType::Plain);

    let mut node_list_items = vec![];
    for item in app.nodes.items.iter() {
        node_list_items.push(ListItem::new(Line::from(vec![Span::styled(
            format!(
                "{}",
                item.to_string().split("://").nth(1).unwrap_or("error")
            ),
            Style::default(),
        )])));
    }

    let mut cpu_list_items = vec![];
    for item in app.cpu.iter() {
        cpu_list_items.push(ListItem::new(Line::from(vec![Span::styled(
            item.to_string(),
            Style::default(),
        )])));
    }

    let mut memory_list_items = vec![];
    for item in app.memory.iter() {
        memory_list_items.push(ListItem::new(Line::from(vec![Span::styled(
            item.to_string(),
            Style::default(),
        )])));
    }

    let mut network_list_items = vec![];
    for item in app.network.iter() {
        network_list_items.push(ListItem::new(Line::from(vec![Span::styled(
            item.to_string(),
            Style::default(),
        )])));
    }

    let mut disk_list_items = vec![];
    for item in app.disk.iter() {
        disk_list_items.push(ListItem::new(Line::from(vec![Span::styled(
            item.to_string(),
            Style::default(),
        )])));
    }

    let node_list = List::new(node_list_items.clone())
        .block(nodes.clone())
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ");

    let cpu_list = List::new(cpu_list_items.clone())
        .block(cpu.clone())
        .style(Style::default().fg(Color::LightBlue))
        .highlight_symbol(" ");

    let memory_list = List::new(memory_list_items.clone())
        .block(memory.clone())
        .style(Style::default().fg(Color::LightBlue))
        .highlight_symbol(" ");

    let network_list = List::new(network_list_items.clone())
        .block(network.clone())
        .style(Style::default().fg(Color::LightBlue))
        .highlight_symbol(" ");

    let disk_list = List::new(disk_list_items.clone())
        .block(disk.clone())
        .style(Style::default().fg(Color::LightBlue))
        .highlight_symbol(" ");

    (node_list, cpu_list, memory_list, network_list, disk_list)
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
