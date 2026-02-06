use std::{
    cmp,
    cmp::Reverse,
    fs,
};

use eyre::Result;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use ratatui::{
    DefaultTerminal,
    Frame,
    crossterm::{
        event,
        event::{
            Event,
            KeyCode,
            KeyEvent,
            KeyEventKind,
        },
    },
    prelude::*,
    widgets::*,
};
use shinden_to_anilist_core::{
    converter::{
        common::AnimeId,
        database,
        database::AnimeDatabase,
        searcher::{
            DefaultSearcher,
            Search,
            Searcher,
        },
    },
    utils::normalize_str,
};
use strsim::jaro_winkler;
use unicode_normalization::UnicodeNormalization;
use wana_kana::ConvertJapanese;

fn main() -> Result<()> {
    simple_eyre::install()?;
    ratatui::run(|terminal| App::new().run(terminal))?;
    Ok(())
}

#[derive(Debug)]
struct App {
    database: AnimeDatabase,
    searcher: DefaultSearcher,
    search_results: Vec<(AnimeId, f32)>,
    selected_id: Option<AnimeId>,
    entry_box_state: ListState,
    input: String,
    exit: bool,
}

impl App {
    fn new() -> Self {
        let db_path = [
            "anime-offline-database.jsonl",
            "shinden-to-anilist-core/anime-offline-database.jsonl",
        ]
        .iter()
        .find(|&&x| fs::exists(x).is_ok_and(|t| t));
        let db_path = *db_path.unwrap();

        let database = database::get_from_mmap(db_path).unwrap();
        let searcher = DefaultSearcher::new(&database);
        Self {
            database,
            searcher,
            search_results: Vec::new(),
            selected_id: None,
            entry_box_state: ListState::default(),
            input: String::new(),
            exit: false,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            },
            _ => {},
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exit(),
            KeyCode::Up => self.up(),
            KeyCode::Down => self.down(),
            KeyCode::Left => {
                self.selected_id = None;
                self.entry_box_state.select(None);
            },
            any => self.handle_typing(any),
        }
    }

    fn handle_typing(&mut self, key: KeyCode) {
        let prev = self.input.len();
        match key {
            KeyCode::Backspace => {
                self.input.pop();
            },
            KeyCode::Char(c) => {
                self.input.push(c);
            },
            _ => {},
        }

        if self.input.is_empty() {
            self.search_results = Vec::new();
            return;
        }

        if prev != self.input.len() {
            let results = self
                .searcher
                .search(&self.input, Search::options().limit(100).threshold(0.10).fuzzy().build());

            if let Some(id) = self.selected_id {
                let pos = results.iter().position(|&(i, _)| i == id);
                match pos {
                    None => {
                        self.entry_box_state.select(None);
                    },
                    Some(i) => {
                        self.entry_box_state.select(Some(i));
                    },
                }
            }

            self.search_results = results;
        }
    }

    fn up(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        let i = match self.entry_box_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.search_results.len() - 1
                } else {
                    i - 1
                }
            },
            None => 0,
        };
        self.selected_id = Some(self.search_results[i].0);
        self.entry_box_state.select(Some(i));
    }

    fn down(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        let i = match self.entry_box_state.selected() {
            Some(i) => (i + 1) % self.search_results.len(),
            None => 0,
        };
        self.selected_id = Some(self.search_results[i].0);
        self.entry_box_state.select(Some(i));
    }

    fn exit(&mut self) { self.exit = true; }

    fn draw(&mut self, frame: &mut Frame) {
        let content_chunks =
            Layout::horizontal([Constraint::Min(0), Constraint::Max(50)]).split(frame.area());

        let left_chunks =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(content_chunks[0]);

        let items = self
            .search_results
            .iter()
            .map(|&(id, score)| (&self.database[id], score))
            .collect::<Vec<_>>();

        frame.render_widget(SearchBox { input: &self.input }, left_chunks[0]);
        frame.render_stateful_widget(
            EntryBox { entries: &items },
            left_chunks[1],
            &mut self.entry_box_state,
        );

        if let Some(entry_id) = self.selected_id {
            let entry = &self.database[entry_id];
            frame.render_widget(DetailsBox { query: &self.input, entry }, content_chunks[1]);
        }
    }
}

struct SearchBox<'a> {
    input: &'a str,
}

impl Widget for SearchBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title("<Search>");

        let text = Paragraph::new(format!("{}_", self.input)).block(block).style(Style::default());

        text.render(area, buf);
    }
}

#[derive(Debug)]
struct EntryBox<'a> {
    entries: &'a [(&'a database::AnimeEntry, f32)],
}

impl StatefulWidget for EntryBox<'_> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items = self
            .entries
            .iter()
            .map(|&(a, s)| {
                let content = format!("{:.2} {}", s, a.title());
                ListItem::new(content).style(Style::default().fg(calc_color(s)))
            })
            .collect::<Vec<_>>();

        let highlighted = state
            .selected()
            .and_then(|i| self.entries.get(i).map(|&(_, s)| calc_color(s)))
            .unwrap_or(Color::White);

        let list = List::new(items)
            .block(Block::bordered().title("<Entries>"))
            .highlight_style(Style::default().bg(highlighted).fg(Color::Black).bold());

        StatefulWidget::render(list, area, buf, state)
    }
}

struct DetailsBox<'a> {
    query: &'a str,
    entry: &'a database::AnimeEntry,
}

impl Widget for DetailsBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let v = self.entry;

        let mut synonyms_scored = v
            .synonyms()
            .iter()
            .filter_map(|t| {
                let normalized = normalize_str(t);
                (!normalized.is_empty())
                    .then_some(
                        t.nfd()
                            .filter(|c| c.is_ascii())
                            .collect::<String>()
                            .to_romaji()
                            .is_ascii()
                            .then_some((t, normalized)),
                    )
                    .flatten()
            })
            .dedup()
            .map(|(t, s)| {
                let score = jaro_winkler(&normalize_str(self.query), &s);
                (t.to_string(), score)
            })
            .collect::<Vec<_>>();

        synonyms_scored.sort_by_key(|&(_, s)| cmp::Reverse(OrderedFloat(s)));

        let spacer = "-".repeat((area.width as i32 - 2).max(0) as usize);
        let mut text = vec![
            Line::from(format!("{} ({})", v.title(), v.id()))
                .style(Style::default().fg(Color::Magenta).bold()),
            Line::from(
                synonyms_scored
                    .iter()
                    .map(|(s, sc)| Span::from(s).style(Style::default().fg(calc_color(*sc as f32))))
                    .take(15)
                    .intersperse(Span::from(", "))
                    .collect::<Vec<_>>(),
            )
            .style(Style::default().italic()),
            Line::from(spacer.as_str()),
        ];

        let mut metadata_list = vec![v.metadata()];
        metadata_list.extend(v.synonyms_metadata());

        metadata_list.sort_by_key(|m| {
            let mut n = 0;
            if m.has_season_keyword() {
                n += 1;
            }
            if m.has_episode_keyword() {
                n += 1;
            }
            if m.has_part_keyword() {
                n += 1;
            }
            Reverse(n)
        });
        let m = metadata_list[0];

        let m_s = if m.season().is_some_and(|x| x == 99.0) {
            metadata_list.iter().find_map(|&x| x.season().filter(|&x| x != 99.0))
        } else {
            None
        };

        let m_p = if m.part().is_some_and(|x| x == 99.0) {
            metadata_list.iter().find_map(|&x| x.part().filter(|&x| x != 99.0))
        } else {
            None
        };

        let m_e = if m.episode().is_some_and(|x| x == 99.0) {
            metadata_list.iter().find_map(|&x| x.episode().filter(|&x| x != 99.0))
        } else {
            None
        };

        text.push(
            format!(
                "season:   {:?}{}",
                m.season(),
                m_s.map(|x| format!(", ({})", x)).unwrap_or_default()
            )
            .into(),
        );
        text.push(
            format!(
                "part:     {:?}{}",
                m.part(),
                m_p.map(|x| format!(", ({})", x)).unwrap_or_default()
            )
            .into(),
        );
        text.push(
            format!(
                "episode:  {:?}{}",
                m.episode(),
                m_e.map(|x| format!(", ({})", x)).unwrap_or_default()
            )
            .into(),
        );
        text.push(format!("tokens:   {:?}", m.tokens()).into());
        if let Some(x) = v.anime_type() {
            text.push(format!("Type:     {x:?}").into());
        }
        if let Some(x) = v.year() {
            text.push(format!("Year:     {x}").into());
        }
        if let Some(x) = v.season() {
            text.push(format!("Season:   {x:?}").into());
        }
        if let Some(x) = v.status() {
            text.push(format!("Status:   {x:?}").into());
        }
        text.push(format!("Episodes: {}", v.episodes()).into());

        Paragraph::new(text)
            .block(Block::bordered().title("<Details>"))
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

fn calc_color(s: f32) -> Color {
    let g = s.powf(1.75) * 255.0;
    let r = 255.0 - g;
    Color::Rgb(r as u8, g as u8, 50)
}
