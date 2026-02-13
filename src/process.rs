use crate::search_pattern::SearchPattern;
pub(crate) use crate::tree::Forest;
use crate::tree::Node;
use crate::utils::highlight_style;
use crate::utils::style_spans;
use crate::Args;
use num_format::Locale;
use num_format::ToFormattedString;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use std::fmt;
use std::ops::Range;
use std::path::Path;
use sysinfo::Pid;
use sysinfo::ProcessRefreshKind;
use sysinfo::ThreadKind;
use sysinfo::UpdateKind;

#[derive(Debug, Clone)]
pub(crate) enum Visible {
    Visible(Vec<Match>),
    NotVisible,
}

impl Default for Visible {
    fn default() -> Self {
        Visible::Visible(Vec::new())
    }
}

impl Visible {
    pub(crate) fn matches(&self) -> impl Iterator<Item = &Match> {
        match self {
            Visible::Visible(items) => items.iter(),
            Visible::NotVisible => [].iter(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Match {
    InPid(Range<usize>),
    InCommand(Range<usize>),
}

#[derive(Debug, Clone)]
pub(crate) struct Process {
    pid: Pid,
    pub(crate) name: String,
    pub(crate) arguments: Vec<String>,
    parent: Option<Pid>,
    cpu: f32,
    ram: u64,
    pub(crate) visible: Visible,
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        for argument in &self.arguments {
            write!(f, " {argument}")?;
        }
        Ok(())
    }
}

impl Node for Process {
    type Id = Pid;

    fn id(&self) -> Pid {
        self.pid
    }

    fn parent(&self) -> Option<Pid> {
        self.parent
    }

    fn accumulate_from(&mut self, other: &Self) {
        self.cpu += other.cpu;
        self.ram += other.ram;
    }
}

impl Process {
    fn from_sysinfo_process(process: &sysinfo::Process) -> Self {
        let mut command_words = process.cmd().to_vec().into_iter();
        Process {
            pid: process.pid(),
            name: match command_words.next() {
                Some(executable) => match Path::new(&executable).file_name() {
                    Some(file_name) => file_name.to_string_lossy().to_string(),
                    None => executable,
                },
                None => match process.exe() {
                    Some(exe) => match exe.file_name() {
                        Some(file_name) => file_name.to_string_lossy().to_string(),
                        None => exe.to_string_lossy().to_string(),
                    },
                    None => process.name().to_string(),
                },
            },
            arguments: command_words.collect(),
            parent: process.parent(),
            cpu: process.cpu_usage(),
            ram: process.memory(),
            visible: Visible::default(),
        }
    }

    pub(crate) fn compare(&self, other: &Process, sort_by: SortBy) -> std::cmp::Ordering {
        let ordering = match sort_by {
            SortBy::Pid => self.id().partial_cmp(&other.id()),
            SortBy::Cpu => other.cpu.partial_cmp(&self.cpu),
            SortBy::Ram => other.ram.partial_cmp(&self.ram),
        };
        match ordering {
            Some(std::cmp::Ordering::Equal) | None => self.pid.cmp(&other.pid),
            Some(ordering) => ordering,
        }
    }

    pub(crate) fn update_visible(&mut self, pattern: &SearchPattern, args: &Args) {
        self.visible = {
            if let SearchPattern::Empty = pattern {
                Visible::Visible(Vec::new())
            } else {
                let matches =
                    self.get_matches(pattern, sysinfo::Pid::from_u32(std::process::id()), args);
                if matches.is_empty() {
                    Visible::NotVisible
                } else {
                    Visible::Visible(matches)
                }
            }
        }
    }

    fn get_matches(&self, pattern: &SearchPattern, treetop_pid: Pid, args: &Args) -> Vec<Match> {
        let mut result = Vec::new();
        for range in pattern.find(&self.id().to_string()) {
            result.push(Match::InPid(range));
        }
        let mut command = self.name.clone();
        for argument in &self.arguments {
            command += " ";
            command += argument;
        }
        for range in pattern.find(&command) {
            if treetop_pid == self.id() && !args.dont_hide_self && range.end > self.name.len() {
                // hide treetop
            } else {
                result.push(Match::InCommand(range));
            }
        }
        result
    }

    pub(crate) fn render_header(area: Rect, sort_by: SortBy, buffer: &mut Buffer) -> u16 {
        let table_header = {
            let mut line = Line::default();
            for column in SortBy::all() {
                let leading_spaces = match column {
                    SortBy::Pid => 5,
                    SortBy::Cpu => 3,
                    SortBy::Ram => 7,
                };
                line.push_span(" ".repeat(leading_spaces));
                line.push_span(Span::styled(
                    format!("{column:?}").to_lowercase(),
                    if column == sort_by {
                        Style::new().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::new()
                    },
                ));
            }
            line.push_span(" ");
            line
        };
        buffer.set_line(area.x, area.y, &table_header, area.width);
        if let Ok(table_header_length) = table_header.width().try_into() {
            if let Some(cell) = buffer.cell_mut((table_header_length, area.y)) {
                cell.set_symbol("┃");
                cell.set_style(Style::new().dark_gray());
            }
            buffer.set_string(
                area.x + table_header_length + 2,
                area.y,
                "executable",
                Style::new(),
            );
            for x in (area.x)..(area.width) {
                if let Some(cell) = buffer.cell_mut((x, area.y + 1)) {
                    cell.set_symbol(if x == table_header_length {
                        "╋"
                    } else {
                        "━"
                    });
                    cell.set_style(Style::new().dark_gray());
                }
            }
        }
        2
    }

    pub(crate) fn table_data(&self) -> Vec<Span<'static>> {
        let mut result: Vec<Span> = Vec::new();
        let pid = self.pid.as_u32().to_string();
        result.push(" ".repeat(8 - pid.len()).into());
        let pid_spans = style_spans(
            vec![pid.into()],
            self.visible.matches().filter_map(|m| match m {
                Match::InPid(range) => Some(range.clone()),
                Match::InCommand(_) => None,
            }),
            highlight_style(),
        );
        result.extend(pid_spans);
        result.push(format!(" {:>4.0}%", self.cpu).into());
        result.push(
            format!(
                " {:>7}MB",
                (self.ram / 2_u64.pow(20)).to_formatted_string(&Locale::en)
            )
            .into(),
        );
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SortBy {
    Pid,
    Cpu,
    Ram,
}

#[allow(clippy::derivable_impls)]
impl Default for SortBy {
    fn default() -> SortBy {
        SortBy::Pid
    }
}

impl SortBy {
    pub(crate) fn next(self) -> SortBy {
        match self {
            SortBy::Pid => SortBy::Cpu,
            SortBy::Cpu => SortBy::Ram,
            SortBy::Ram => SortBy::Pid,
        }
    }

    fn all() -> impl Iterator<Item = SortBy> {
        vec![SortBy::Pid, SortBy::Cpu, SortBy::Ram].into_iter()
    }
}

#[derive(Debug)]
pub(crate) struct ProcessWatcher(ProcessWatcherInner);

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum ProcessWatcherInner {
    Production {
        system: sysinfo::System,
    },
    #[cfg(test)]
    TestWatcher {
        processes: Vec<Process>,
    },
}

impl ProcessWatcher {
    pub(crate) fn new(system: sysinfo::System) -> ProcessWatcher {
        ProcessWatcher(ProcessWatcherInner::Production { system })
    }

    pub(crate) fn refresh(&mut self) {
        match self {
            ProcessWatcher(ProcessWatcherInner::Production { system }) => system
                .refresh_processes_specifics(
                    ProcessRefreshKind::new()
                        .with_memory()
                        .with_cpu()
                        .with_cmd(UpdateKind::OnlyIfNotSet),
                ),
            #[cfg(test)]
            ProcessWatcher(ProcessWatcherInner::TestWatcher { .. }) => {}
        }
    }

    pub(crate) fn get_forest(&self) -> Forest<Process> {
        match self {
            ProcessWatcher(ProcessWatcherInner::Production { system }) => Forest::new_forest(
                system
                    .processes()
                    .values()
                    .filter(|process| process.thread_kind() != Some(ThreadKind::Userland))
                    .map(Process::from_sysinfo_process),
            ),
            #[cfg(test)]
            ProcessWatcher(ProcessWatcherInner::TestWatcher { processes }) => {
                Forest::new_forest(processes.iter().cloned())
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use std::string::ToString;

    impl Process {
        pub(crate) fn fake(pid: usize, cpu: f32, parent: Option<usize>) -> Process {
            Process {
                pid: pid.into(),
                name: crate::utils::test_utils::render_number(pid).to_string(),
                arguments: Vec::new(),
                parent: parent.map(From::from),
                cpu,
                ram: 0,
                visible: Visible::default(),
            }
        }

        pub(crate) fn set_name(mut self, name: &str) -> Self {
            self.name = name.to_string();
            self
        }

        pub(crate) fn set_arguments(mut self, arguments: Vec<&str>) -> Self {
            self.arguments = arguments.into_iter().map(ToString::to_string).collect();
            self
        }
    }

    impl Default for Process {
        fn default() -> Self {
            Process {
                pid: 42.into(),
                name: "name".to_string(),
                arguments: vec![],
                parent: None,
                cpu: 0.0,
                ram: 0,
                visible: Visible::default(),
            }
        }
    }

    impl ProcessWatcher {
        pub(crate) fn fake(processes: Vec<Process>) -> ProcessWatcher {
            ProcessWatcher(ProcessWatcherInner::TestWatcher { processes })
        }
    }

    mod is_match {
        use super::*;

        #[test]
        fn is_match_considers_arguments() {
            assert!(!Process::default()
                .set_arguments(vec!["foo"])
                .get_matches(
                    &SearchPattern::from_string("foo"),
                    0.into(),
                    &Args::default()
                )
                .is_empty());
            assert!(Process::default()
                .set_arguments(vec!["foo"])
                .get_matches(
                    &SearchPattern::from_string("bar"),
                    0.into(),
                    &Args::default()
                )
                .is_empty());
            assert!(!Process::default()
                .set_arguments(vec!["foobarbaz"])
                .get_matches(
                    &SearchPattern::from_string("bar"),
                    0.into(),
                    &Args::default()
                )
                .is_empty());
        }

        #[test]
        fn filtering_by_matching_on_multiple_process_arguments() {
            assert!(!Process::default()
                .set_arguments(vec!["foo", "bar"])
                .get_matches(
                    &SearchPattern::from_string("fo.*ar"),
                    0.into(),
                    &Args::default()
                )
                .is_empty());
            assert!(!Process::default()
                .set_arguments(vec!["foo", "bar"])
                .get_matches(
                    &SearchPattern::from_string("foo bar"),
                    0.into(),
                    &Args::default()
                )
                .is_empty());
            assert!(!Process::default()
                .set_name("foo")
                .set_arguments(vec!["bar"])
                .get_matches(
                    &SearchPattern::from_string("foo bar"),
                    0.into(),
                    &Args::default()
                )
                .is_empty());
        }

        #[test]
        fn is_match_hides_treetop_for_arguments() {
            let process = Process {
                pid: 42.into(),
                name: "treetop".to_string(),
                arguments: vec!["foo".to_string()],
                ..Process::default()
            };
            assert!(process
                .get_matches(
                    &SearchPattern::from_string("foo"),
                    42.into(),
                    &Args::default()
                )
                .is_empty());
            assert!(!process
                .get_matches(
                    &SearchPattern::from_string("foo"),
                    43.into(),
                    &Args::default()
                )
                .is_empty());
            assert!(!process
                .get_matches(
                    &SearchPattern::from_string("treetop"),
                    42.into(),
                    &Args::default()
                )
                .is_empty());
            assert!(!process
                .get_matches(
                    &SearchPattern::from_string("42"),
                    42.into(),
                    &Args::default()
                )
                .is_empty());
        }

        #[test]
        fn is_match_shows_treetop_when_asked_to() {
            let process = Process {
                pid: 42.into(),
                name: "treetop".to_string(),
                arguments: vec!["foo".to_string()],
                ..Process::default()
            };
            assert!(!process
                .get_matches(
                    &SearchPattern::from_string("foo"),
                    42.into(),
                    &Args {
                        dont_hide_self: true,
                        ..Args::default()
                    }
                )
                .is_empty());
        }
    }
}
