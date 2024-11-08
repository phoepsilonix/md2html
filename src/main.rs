// Copyright 2024 Masato TOYOSHIMA <phoepsilonix@phoepsilonix.love>
// Copyright 2015 Google Inc. All rights reserved.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! Command line tool to exercise pulldown-cmark.

#![forbid(unsafe_code)]
use pulldown_cmark::{html, Options, Parser, Event, Tag, TagEnd, CodeBlockKind};
use pico_args::Arguments;
use std::io::{self, Read};
use std::fs::File;
use std::path::PathBuf;
//use std::mem;

fn perform_dry_run(text: &str, opts: Options) {
    let p = Parser::new_ext(text, opts);
    let count = p.count();
    println!("{} events", count);
}

fn print_events(text: &str, opts: Options) {
    let parser = Parser::new_ext(text, opts).into_offset_iter();
    for (event, range) in parser {
        println!("{:?}: {:?}", range, event);
    }
    println!("EOF");
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        println!("Usage: [options] [FILES...]");
        println!("Options:");
        println!("  -h, --help                 Print this help message");
        println!("  -d, --dry-run              Dry run, produce no output");
        println!("  -e, --events               Print event sequence instead of rendering");
        println!("  -T, --enable-tables        Enable GitHub-style tables");
        println!("  -F, --enable-footnotes     Enable GitHub-style footnotes");
        println!("  --enable-old-footnotes     Enable Hoedown-style footnotes");
        println!("  -S, --enable-strikethrough Enable GitHub-style strikethrough");
        println!("  -L, --enable-tasklists     Enable GitHub-style task lists");
        println!("  -P, --enable-smart-punctuation Enable smart punctuation");
        println!("  -H, --enable-heading-attributes Enable heading attributes");
        println!("  -M, --enable-metadata-blocks Enable metadata blocks");
        return Ok(());
    }

    let dry_run = args.contains(["-d", "--dry-run"]);
    let events = args.contains(["-e", "--events"]);

    let mut opts = Options::empty();
    if args.contains(["-T", "--enable-tables"]) {
        opts.insert(Options::ENABLE_TABLES);
    }
    if args.contains(["-F", "--enable-footnotes"]) {
        opts.insert(Options::ENABLE_FOOTNOTES);
    }
    if args.contains("--enable-old-footnotes") {
        opts.insert(Options::ENABLE_OLD_FOOTNOTES);
    }
    if args.contains(["-S", "--enable-strikethrough"]) {
        opts.insert(Options::ENABLE_STRIKETHROUGH);
    }
    if args.contains(["-L", "--enable-tasklists"]) {
        opts.insert(Options::ENABLE_TASKLISTS);
    }
    if args.contains(["-P", "--enable-smart-punctuation"]) {
        opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    }
    if args.contains(["-H", "--enable-heading-attributes"]) {
        opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    }
    if args.contains(["-M", "--enable-metadata-blocks"]) {
        opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        opts.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
    }

    // free() を使用して残りの引数を取得し、String に変換
    let mut files = Vec::new();
    while let Ok(file) = args.free_from_str::<String>() {
        files.push(file);
    }

    let mut input = String::new();
    if !files.is_empty() {
        for filename in files {
            let real_path = PathBuf::from(filename);
            let mut f = File::open(&real_path)?;
            f.read_to_string(&mut input)?;
            process_input(&input, dry_run, events, opts);
        }
    } else {
        io::stdin().lock().read_to_string(&mut input)?;
        process_input(&input, dry_run, events, opts);
    }
    Ok(())
}

fn process_input(input: &str, dry_run: bool, events: bool, opts: Options) {
    if events {
        print_events(input, opts);
    } else if dry_run {
        perform_dry_run(input, opts);
    } else {
        pulldown_cmark(input, opts);
    }
}

pub fn pulldown_cmark(input: &str, opts: Options) {
    let mut _in_code_block = false;
    let mut p = Vec::new();
    let mut code = String::new();
    let mut lang = String::new();

    for event in Parser::new_ext(input, opts) {
        match event {
            Event::Start(Tag::CodeBlock(info)) => {
                _in_code_block = true;
                if let CodeBlockKind::Fenced(info) = info {
                    lang = info.to_string();
                }
            }
            Event::Text(t) => {
                if _in_code_block {
                    code.push_str(&t);
                } else {
                    let replacer = gh_emoji::Replacer::new();
                    let s = replacer.replace_all(&t);
                    p.push(Event::Text(s.to_string().into()));
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                _in_code_block = false;
                let html = code;
                p.push(Event::Html(
                        format!("<pre><code class=\"language-{}\">{}</code></pre>", lang, html).into(),
                        ));
                code = String::new();
            }
            _ => {
                p.push(event);
            }
        }
    };
    let mut buffer = String::new();
    html::push_html(&mut buffer, &mut p.into_iter());
    print!("{}", buffer);

    //let stdout = std::io::stdout();
    //let handle = stdout.lock();
    //let _ = html::write_html_io(handle, &mut p.clone().into_iter());

    // Since the program will now terminate and the memory will be returned
    // to the operating system anyway, there is no point in tidely cleaning
    // up all the datastructures we have used. We shouldn't do this if we'd
    // do other things after this, because this is basically intentionally
    // leaking data. Skipping cleanup lets us return a bit (~5%) faster.
    //mem::forget(p);
}
