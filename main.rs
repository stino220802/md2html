use std::fs;
use std::io::{self, Write};
use pulldown_cmark::{Parser as MarkdownParser, Options, Event, Tag};
use clap::Parser;
use v_htmlescape::escape;
use pulldown_cmark::HeadingLevel;

struct CustomClasses {
    heading: Option<String>,
    paragraph: Option<String>,

}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    #[arg(short, long)]
    input_file: String,

    #[arg(short, long)]
    output_file: Option<String>,

    #[arg(long)]
    heading_class: Option<String>,


    #[arg(long)]
    paragraph_class: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::try_parse()?; 
    let custom_classes = CustomClasses {
        heading: args.heading_class,
        paragraph: args.paragraph_class,
        
    };

    let markdown_input = fs::read_to_string(args.input_file.clone())?;
    if args.verbose {
        println!("Input file: {}", args.input_file);
        if let Some(output_file) = &args.output_file {
            println!("Output file: {}", output_file);
        } else {
            println!("Output to stdout");
        }
        
    }
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let _parser = MarkdownParser::new_ext(&markdown_input, options); 

    let mut html_output = String::new();
    let mut list_depth = 0;
    let mut table_state = None;
    let mut list_stack = Vec::new();
    let mut headings = Vec::new(); 
    let mut is_heading = false;
    for event in MarkdownParser::new_ext(&markdown_input, options) {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading(level, id, _) => {
                        let class_attr = custom_classes.heading
                            .as_ref()
                            .map(|class| format!(" class=\"{}\"", class))
                            .unwrap_or_default();
                        html_output.push_str(&format!("<{}{}>", level, class_attr));
                        headings.push((level, id.clone().unwrap_or_default().to_string(), String::new()));
                        is_heading = true;
                    }
                    Tag::Paragraph => {
                        let class_attr = custom_classes.paragraph
                        .as_ref()
                        .map(|class| format!(" class=\"{}\"", class))
                        .unwrap_or_default();
                    html_output.push_str(&format!("<p{}>", class_attr));
                    }
                    Tag::Strong => {
                        html_output.push_str("<strong>");
                    }
                    Tag::Emphasis => {
                        html_output.push_str("<em>");
                    }
                    Tag::Link(_link_type, url, _title) => {
                        html_output.push_str(&format!("<a href=\"{}\">", url));
                    }
                    Tag::Image(_link_type, url, alt) => {
                        let title_attr = if alt.is_empty() {
                            String::new()
                        } else {
                            format!(" title=\"{}\"", alt)
                        };
    
                        html_output.push_str(&format!("<img src=\"{}\" alt=\"{}\"{}>\n", url, alt, title_attr));
                    }
                    Tag::List(Some(start_num)) => {
                        html_output.push_str(&format!("<ol start=\"{}\">\n", start_num));
                        list_stack.push("ol"); 
                    }
                    Tag::List(None) => {
                        html_output.push_str("<ul>\n");
                        list_stack.push("ul");
                    }
                    Tag::Item => {
                        html_output.push_str("<li>");
                    }
                    Tag::CodeBlock(code_block_kind) => {
                        let lang = match code_block_kind {
                            pulldown_cmark::CodeBlockKind::Indented => "".to_string(),
                            pulldown_cmark::CodeBlockKind::Fenced(info) => info.to_string(),
                        };
                        html_output.push_str(&format!("<pre><code class=\"language-{}\">", lang));
                    }
                    Tag::BlockQuote => {
                        html_output.push_str("<blockquote>\n");
                    }
                    Tag::Table(alignment) => {
                        html_output.push_str("<table>\n");
                        table_state = Some(alignment);
                    }
                    Tag::TableHead => {
                        html_output.push_str("<thead>\n<tr>");
                    }
                    Tag::TableRow => {
                        html_output.push_str("<tr>");
                    }
                    Tag::TableCell => {
                        let tag = match &table_state {
                            Some(alignment) if alignment.iter().any(|&a| a == pulldown_cmark::Alignment::Right) => "th",
                            _ => "td",
                        };
                        html_output.push_str(&format!("<{}>", tag));
                    }
                    Tag::Strikethrough => {
                        html_output.push_str("<s>");
                    } 
                    _ => {} 
                }
            }
            Event::End(tag) => {
                match tag {
                    Tag::Heading(level, _, _) => {
                        html_output.push_str(&format!("</{}>\n", level)); 
                        is_heading = false;
                    }
                    Tag::Paragraph => {
                        html_output.push_str("</p>\n"); 
                    }
                    Tag::Strong => {
                        html_output.push_str("</strong>");
                    }
                    Tag::Emphasis => {
                        html_output.push_str("</em>");
                    }
                    Tag::Link(..) => { 
                        html_output.push_str("</a>");
                    }
                    Tag::List(_) => {
                        if let Some(list_type) = list_stack.pop() {
                            html_output.push_str(&format!("</{}>\n", list_type)); 
                        }
                    }
                    Tag::Item => {
                        html_output.push_str("</li>\n");
                    }
                    Tag::CodeBlock(_) => {
                        html_output.push_str("</code></pre>\n");
                    }
                    Tag::BlockQuote => {
                        html_output.push_str("</blockquote>\n");
                    }
                    Tag::Table(_) => {
                        html_output.push_str("</table>\n");
                        table_state = None;
                    }
                    Tag::TableHead => {
                        html_output.push_str("</tr></thead>\n");
                    }
                    Tag::TableRow => {
                        html_output.push_str("</tr>\n");
                    }
                    Tag::TableCell => {
                        let tag = match &table_state {
                            Some(alignment) if alignment.iter().any(|&a| a == pulldown_cmark::Alignment::Right) => "th",
                            _ => "td",
                        };
                        html_output.push_str(&format!("</{}>", tag));
                    }
                    Tag::Strikethrough => {
                        html_output.push_str("</s>");
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                let text_str = escape(&text.clone().into_string()).to_string();
                if is_heading {
                   
                    if let Some((_, _, ref mut heading_content)) = headings.last_mut() {
                        heading_content.push_str(&text.clone().into_string());
                    }
                } 
                    
                    html_output.push_str(&escape(&text.into_string()).to_string());
                
            }
            Event::Code(text) => {
                html_output.push_str(&format!("<code>{}</code>", escape(&text.into_string()).to_string()));
            }
            Event::Rule => {
                html_output.push_str("<hr>\n");
            }
            _ => {} 
        }
    }
    let toc = generate_toc(&headings[..]);

    html_output.insert_str(0, &toc);
    match args.output_file {
        Some(file_path) => fs::write(file_path, html_output.clone())?,
        None => io::stdout().write_all(html_output.as_bytes())?,
    }

    
    Ok(())
}

fn generate_toc(headings: &[(HeadingLevel, String, String)]) -> String {
    let mut toc = String::from("<nav>\n<h2>Table of Contents</h2>\n<ul>\n");
    let mut current_level = HeadingLevel::H1; 

    for (level, id, content) in headings {
        while *level > current_level {
            toc.push_str("<ul>\n");
            current_level = next_level(&current_level);
        }
        while *level < current_level {
            toc.push_str("</ul>\n");
            current_level = prev_level(&current_level);
        }

        toc.push_str(&format!(
            "<li><a href=\"#{}\">{}</a></li>\n",
            id,
            escape(&content).to_string()
        ));
    }

    while current_level > HeadingLevel::H1 {
        toc.push_str("</ul>\n");
        current_level = prev_level(&current_level);
    }

    toc.push_str("</ul>\n</nav>\n");
    toc
}

fn next_level(level: &HeadingLevel) -> HeadingLevel {
    match level {
        HeadingLevel::H1 => HeadingLevel::H2,
        HeadingLevel::H2 => HeadingLevel::H3,
        HeadingLevel::H3 => HeadingLevel::H4,
        HeadingLevel::H4 => HeadingLevel::H5,
        HeadingLevel::H5 => HeadingLevel::H6,
        HeadingLevel::H6 => HeadingLevel::H6, 
    }
}

fn prev_level(level: &HeadingLevel) -> HeadingLevel {
    match level {
        HeadingLevel::H2 => HeadingLevel::H1,
        HeadingLevel::H3 => HeadingLevel::H2,
        HeadingLevel::H4 => HeadingLevel::H3,
        HeadingLevel::H5 => HeadingLevel::H4,
        HeadingLevel::H6 => HeadingLevel::H5,
        HeadingLevel::H1 => HeadingLevel::H1, 
    }
}
