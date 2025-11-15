use std::fs::File;

use mdbook::book::{Book, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::TagEnd;
use serde_json::Value;
use tempfile::tempdir;

pub struct NixEval;

impl Default for NixEval {
    fn default() -> Self {
        Self::new()
    }
}

impl NixEval {
    pub fn new() -> NixEval {
        NixEval
    }
}

struct NixConfig {
    eval_command: String,
    eval_args: String,
}

impl Preprocessor for NixEval {
    fn name(&self) -> &str {
        "nix-eval"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let mut nix_config = NixConfig {
            eval_command: "nix-instantiate".to_owned(),
            eval_args: "".to_owned(),
        };
        if let Some(config) = _ctx.config.get_preprocessor("nix-eval") {
            if let Some(v) = config.get("eval_command") {
                nix_config.eval_command = v.as_str().expect("Not a string").to_owned();
            }
            if let Some(v) = config.get("eval_args") {
                nix_config.eval_args = v.as_str().expect("Not a string").to_owned();
            }
        }
        book.for_each_mut(|book| {
            if let mdbook::BookItem::Chapter(chapter) = book {
                // TODO: better error handling...
                if let Err(e) = nix_eval(&nix_config, chapter) {
                    eprintln!("nix-eval error: {:?}", e);
                }
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

fn nix_eval(config: &NixConfig, chapter: &mut Chapter) -> Result<(), Error> {
    use pulldown_cmark::{CodeBlockKind, CowStr, Event, Parser, Tag};

    let chapter_temp_dir = tempdir()?;

    // mini state machine for the current nix-eval tag
    let mut nix: Option<String> = None;
    let mut nix_file_name = String::new();
    let events = Parser::new(&chapter.content).filter_map(|event| {
        match &event {
            // a code block for the `nix-eval` language was started
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                if !(info.ends_with(".nix") || info.to_string() == "nix") {
                    return Some(event);
                }
                nix = Some("".to_owned());
                let is_file = info.ends_with(".nix");
                let is_eval = info.to_string() == "nix";
                if !(is_file || is_eval) {
                    return Some(event);
                }

                nix_file_name = match is_file {
                    true => info.to_string(),
                    false => "eval.nix".to_owned(),
                };
                None
            }
            // a code block for the `nix-eval` language was ended
            Event::End(TagEnd::CodeBlock) => {
                if nix.is_none() {
                    return Some(event);
                }
                let nix_file_path = chapter_temp_dir.path().join(&nix_file_name);

                // extract the contents of the diagram
                let nix_src = nix.take().expect("nix was started");

                let mut out_file = File::create(nix_file_path.as_path()).expect("nix file created");
                out_file
                    .write_all(nix_src.as_ref())
                    .expect("wrote temp file");

                // eprintln!("writing temp file: {:?}", nix_file_path.as_path());

                // evaluate the nix expression
                use std::io::Write;
                use std::process::{Command, Stdio};

                let quick_eval = match Command::new(&config.eval_command)
                    .current_dir(chapter_temp_dir.path())
                    .arg("--eval")
                    .arg(nix_file_path.as_path())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!(
                            "failed to launch nix-eval, not rendering nix-eval block: {:?}",
                            e
                        );
                        return None;
                    }
                };
                let quick_eval_out = quick_eval.wait_with_output().expect("can launch nix-eval");
                let quick_eval_str = String::from_utf8(quick_eval_out.stdout).expect("valid utf-8");
                let wrap_lambda = quick_eval_str.trim() == "<LAMBDA>";

                let mut c_init = Command::new(&config.eval_command);
                let mut command = c_init
                    .current_dir(chapter_temp_dir.path())
                    .arg("--json")
                    .arg("--eval")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                command = command.arg("--strict");

                if !config.eval_args.is_empty() {
                    command = command.args(config.eval_args.split(" "));
                }

                if wrap_lambda {
                    command = command.arg("-E");
                    command = command.arg(format!(
                        "import {} {}",
                        nix_file_path.as_path().to_str().expect("invalid path"),
                        "{}"
                    ));
                } else {
                    command = command.arg(nix_file_path.as_path());
                }

                // eprintln!("{:?}", command);
                let child = match command.spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!(
                            "failed to launch nix-eval, not rendering nix-eval block: {:?}",
                            e
                        );
                        return None;
                    }
                };

                let cmd_output = child.wait_with_output().expect("can launch nix-eval");

                let output: String = String::from_utf8(cmd_output.stdout).expect("valid utf-8");
                let trimmed_output = output.trim();
                let mut nix_eval_output = "".to_owned();
                if !cmd_output.status.success() {
                    nix_eval_output = String::from_utf8(cmd_output.stderr).expect("valid utf-8");
                } else if !trimmed_output.is_empty() {
                    match serde_json::from_str(trimmed_output) {
                        Ok(v) => {
                            let line = match v {
                                Value::String(s) => {
                                    let trimmed = s.trim();
                                    if trimmed.contains("\n") {
                                        format!("\"\n{}\n\"", trimmed)
                                    } else {
                                        format!("\"{}\"", trimmed)
                                    }
                                }
                                Value::Bool(b) => serde_json::to_string_pretty(&b).unwrap(),
                                Value::Null => "null".to_owned(),
                                Value::Number(n) => format!("{}", n),
                                Value::Array(a) => serde_json::to_string_pretty(&a).unwrap(),
                                Value::Object(o) => serde_json::to_string_pretty(&o).unwrap(),
                            };
                            nix_eval_output.push_str(line.as_str())
                        }
                        Err(_e) => nix_eval_output.push_str(trimmed_output),
                    };
                } else {
                    nix_eval_output.push_str("<< no output >>")
                }

                let input_header = format!("**{}**\n", nix_file_name);

                let input = format!("\n```nix\n{}\n```\n", nix_src.trim());
                let output = format!("\n```json\n{}\n```\n", nix_eval_output.trim());

                nix = None;
                nix_file_name = String::new();

                Some(Event::Text(CowStr::from(format!(
                    "\n{}\n<div style='border-left: 2px solid;'>\n{}\n\n{}\n</div>\n\n",
                    input_header, input, output
                ))))
            }
            // intercept text events if we're currently in the code block state
            Event::Text(txt) => {
                if let Some(nix) = nix.as_mut() {
                    nix.push_str(txt);
                    None
                } else {
                    Some(event)
                }
            }
            // don't touch other events
            _ => Some(event),
        }
    });

    let mut buf = String::with_capacity(chapter.content.len());
    pulldown_cmark_to_cmark::cmark(events, &mut buf).expect("can re-render cmark");
    chapter.content = buf;

    Ok(())
}
