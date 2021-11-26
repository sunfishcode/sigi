use crate::data;
use crate::effects::{EffectNames, StackEffect};
use crate::output::OutputFormat;

// ===== Peek =====

pub struct Peek {
    pub stack: String,
}

impl StackEffect for Peek {
    fn names<'a>() -> EffectNames<'a> {
        EffectNames {
            name: "peek",
            description: "Show the current item",
            aliases: &["show"],
        }
    }

    fn run(&self, output: OutputFormat) {
        if let Ok(items) = data::load(&self.stack) {
            let top_item = &items.last().unwrap().contents;
            if !items.is_empty() {
                output.log(vec!["position", "item"], vec![vec!["Now", top_item]]);
            }
        }
    }
}

// ===== Some help for doing ListAll/Head/Tail =====

trait Listable {
    fn range<'a>(&'a self) -> ListRange<'a>;
}

struct ListRange<'a> {
    stack: &'a str,
    // Ignored if starting "from_end".
    start: usize,
    limit: Option<usize>,
    from_end: bool,
}

fn list_range(listable: &impl Listable, output: OutputFormat) {
    if let OutputFormat::Silent = output {
        return;
    }

    let range = listable.range();

    if let Ok(items) = data::load(range.stack) {
        let limit = match range.limit {
            Some(n) => n,
            None => items.len(),
        };

        let start = if range.from_end {
            if limit <= items.len() {
                items.len() - limit
            } else {
                0
            }
        } else {
            range.start
        };

        let lines = items
            .into_iter()
            .rev()
            .enumerate()
            .skip(start)
            .take(limit)
            .map(|(i, item)| {
                let position = match output {
                    // Pad human output numbers to line up nicely with "Now".
                    OutputFormat::Human(_) => match i {
                        0 => "Now".to_string(),
                        1..=9 => format!("  {}", i),
                        10..=099 => format!(" {}", i),
                        _ => i.to_string(),
                    },
                    _ => i.to_string(),
                };

                let created = item
                    .history
                    .iter()
                    .find(|(status, _)| status == "created")
                    .map(|(_, dt)| output.format_time(*dt))
                    .unwrap_or("unknown".to_string());

                vec![position, item.contents, created]
            })
            .collect::<Vec<_>>();

        // Get the lines into a "borrow" state (&str instead of String) to make log happy.
        let lines = lines
            .iter()
            .map(|line| line.iter().map(|s| s.as_str()).collect())
            .collect();

        output.log(vec!["position", "item", "created"], lines);
    }
}

// ===== ListAll =====

pub struct ListAll {
    pub stack: String,
}

impl Listable for ListAll {
    fn range<'a>(&'a self) -> ListRange<'a> {
        ListRange {
            stack: &self.stack,
            start: 0,
            limit: None,
            from_end: false,
        }
    }
}

impl StackEffect for ListAll {
    fn names<'a>() -> EffectNames<'a> {
        EffectNames {
            name: "list",
            description: "List all items",
            aliases: &["ls", "snoop", "show", "all"],
        }
    }

    fn run(&self, output: OutputFormat) {
        list_range(self, output);
    }
}

// ===== Head =====

const HEAD_DEFAULT_LIMIT: usize = 10;

pub struct Head {
    pub stack: String,
    pub n: Option<usize>,
}

impl Listable for Head {
    fn range<'a>(&'a self) -> ListRange<'a> {
        ListRange {
            stack: &self.stack,
            start: 0,
            limit: Some(self.n.unwrap_or(HEAD_DEFAULT_LIMIT)),
            from_end: false,
        }
    }
}

impl StackEffect for Head {
    fn names<'a>() -> EffectNames<'a> {
        EffectNames {
            name: "head",
            description: "List the first N items",
            aliases: &["top", "first"],
        }
    }

    fn run(&self, output: OutputFormat) {
        list_range(self, output);
    }
}

// ===== Tail =====

const TAIL_DEFAULT_LIMIT: usize = 10;

pub struct Tail {
    pub stack: String,
    pub n: Option<usize>,
}

impl Listable for Tail {
    fn range<'a>(&'a self) -> ListRange<'a> {
        ListRange {
            stack: &self.stack,
            start: 0,
            limit: Some(self.n.unwrap_or(TAIL_DEFAULT_LIMIT)),
            from_end: true,
        }
    }
}

impl StackEffect for Tail {
    fn names<'a>() -> EffectNames<'a> {
        EffectNames {
            name: "tail",
            description: "List the last N items",
            aliases: &["bottom", "last"],
        }
    }

    fn run(&self, output: OutputFormat) {
        list_range(self, output);
    }
}

// ===== Count =====

pub struct Count {
    pub stack: String,
}

impl StackEffect for Count {
    fn names<'a>() -> EffectNames<'a> {
        EffectNames {
            name: "count",
            description: "Print the total number of items in the stack",
            aliases: &["size", "length"],
        }
    }

    fn run(&self, output: OutputFormat) {
        if let Ok(items) = data::load(&self.stack) {
            let len = items.len().to_string();
            output.log(vec!["items"], vec![vec![&len]])
        }
    }
}

// ===== IsEmpty =====

pub struct IsEmpty {
    pub stack: String,
}

impl StackEffect for IsEmpty {
    fn names<'a>() -> EffectNames<'a> {
        EffectNames {
            name: "is-empty",
            description: "\"true\" if stack has zero items, \"false\" (and nonzero exit code) if the stack does have items",
            aliases: &["empty"],
        }
    }

    fn run(&self, output: OutputFormat) {
        if let Ok(items) = data::load(&self.stack) {
            if !items.is_empty() {
                output.log(vec!["empty"], vec![vec!["false"]]);
                // Exit with a failure (nonzero status) when not empty.
                // This helps people who do shell scripting do something like:
                //     while ! sigi -t $stack is-empty ; do <ETC> ; done
                // TODO: It would be better modeled as an error, if anyone uses as a lib this will surprise.
                std::process::exit(1);
            }
        }
        output.log(vec!["empty"], vec![vec!["true"]]);
    }
}
