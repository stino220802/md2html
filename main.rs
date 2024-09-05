use std::fs;
use std::io::{self, Write};
use pulldown_cmark::{Parser as MarkdownParser, Options, Event, Tag};
use clap::Parser;
use v_htmlescape::escape;
use pulldown_cmark::HeadingLevel;

use rfd::{FileDialog};

use dialoguer::{Confirm};

use egui::{CentralPanel, Context, RichText, ScrollArea, TextEdit, TextStyle, Vec2, Button};
use eframe::{egui, run_native, NativeOptions, App};


struct CustomClasses {
    heading: Option<String>,
    paragraph: Option<String>,

}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    #[arg(short, long)]
    input_files: Vec<String>,

    #[arg(short, long)]
    output_file: Option<String>,

    #[arg(long)]
    heading_class: Option<String>,


    #[arg(long)]
    paragraph_class: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

struct MyApp {
    markdown_input: String,
    html_output: String,
    args: Args, 
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            markdown_input: String::new(),
            html_output: String::new(),
            args: Args::parse(),
        }
    }

    fn convert_markdown(&mut self) {
        self.html_output = convert_markdown_to_html(&self.markdown_input, &self.args)
    }

    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output_file = if self.args.output_file.is_some() {
            self.args.output_file.clone().unwrap() 
        } else {
            if let Some(file_path) = FileDialog::new()
                .add_filter("HTML Files", &["html"])
                .save_file()
            {
                file_path.to_str().unwrap().to_string()
            } else {
                println!("Save cancelled. Exiting.");
                return Ok(());
            }
        };

        if std::path::Path::new(&output_file).exists() {
            if !Confirm::new()
                .with_prompt(format!("File '{}' already exists. Overwrite?", output_file))
                .interact()? 
            {
                println!("Overwrite cancelled. Exiting.");
                return Ok(());
            }
        }

        fs::write(output_file.clone(), self.html_output.clone())?; 
        println!("Conversion complete! Output saved to: {}", output_file);
        Ok(())
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Markdown to HTML Converter");

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label("Markdown Input:");
                ui.add_space(5.0);
                ui.add(TextEdit::multiline(&mut self.markdown_input)
                    .font(TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(10)); 
            });

            ui.add_space(10.0);

            
            ui.horizontal(|ui| {
                if ui.button("Convert").clicked() {
                    self.convert_markdown();
                }

                if ui.button("Save to File").clicked() {
                    if let Err(err) = self.save_to_file() {
                        eprintln!("Error saving to file: {}", err);
                    }
                }
            });

            ui.add_space(10.0);

            ui.separator();

            ui.add_space(10.0);


            ui.group(|ui| {
                ui.label("HTML Output:");
                ui.add_space(5.0); 
                ui.add(TextEdit::multiline(&mut self.html_output)
                    .font(TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(10));
            });
        });
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.input_files.is_empty() {
        let native_options = NativeOptions::default();
        run_native(
            "Markdown to HTML Converter",
            native_options,
            Box::new(|cc| {
                Ok(Box::new(MyApp::new(cc)) as Box<dyn App>) 
            }),
        )
        .map_err(|e| e.to_string())?;
    } else {
        for input_file in &args.input_files {
            
            let markdown_input = fs::read_to_string(input_file.clone())?;
       
            let markdown_input = fs::read_to_string(input_file.clone())?;
            if args.verbose {
                println!("Input file: {}", input_file);
                if let Some(output_file) = &args.output_file {
                    println!("Output file: {}", output_file);
                } else {
                    println!("Output to stdout");
                }
            }
            let html_output = convert_markdown_to_html(&markdown_input, &args);

            let output_file = if args.output_file.is_some() {
                args.output_file.clone().unwrap() 
            } else {
                let file_stem = std::path::Path::new(input_file)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap();
                format!("{}.html", file_stem)
            };

            if std::path::Path::new(&output_file).exists() {
                if !Confirm::new()
                    .with_prompt(format!("File '{}' already exists. Overwrite?", output_file))
                    .interact()? 
                {
                    println!("Overwrite cancelled for {}. Skipping.", input_file);
                    continue;
                }
            }

            fs::write(output_file.clone(), html_output.clone())?; 

            println!("Conversion complete! Output saved to: {}", output_file);
        }
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

fn convert_markdown_to_html(markdown_input: &str, args: &Args) -> String {
    let custom_classes = CustomClasses {
        heading: args.heading_class.clone(),
        paragraph: args.paragraph_class.clone(),
    };

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

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

    html_output 
}