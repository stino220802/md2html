# Markdown to HTML Converter

A versatile tool written in Rust that converts Markdown files to HTML, with both a command-line interface and a graphical user interface (GUI) for added convenience.

## Features

**Markdown Conversion**

*   Converts basic Markdown elements:
    *   Headings
    *   Paragraphs
    *   Bold, italic, strikethrough, inline text
    *   Links
    *   Images
    *   Lists (ordered, unordered, and nested)
    *   Code blocks
    *   Blockquotes
    *   Tables
    *   Horizontal rules
*   Handles nested lists correctly
*   Escapes HTML entities to prevent XSS vulnerabilities

**Table of Contents**

*   Generates a table of contents based on headings
*   Allows customization of the maximum depth of headings included in the TOC

**Customization**

*   Allows customization of CSS classes for headings and paragraphs
*   Provides verbose output for debugging and monitoring

**Linting**

*   Integrates Markdown linting using the `remark` crate to identify potential errors or inconsistencies in your Markdown.

**Link Validation**

*   Checks links in your Markdown documents using the `ureq` crate and reports any potentially broken links.

**Batch Conversion**

*   Supports converting multiple Markdown files at once, streamlining your workflow.

**GUI**

*   Provides a user-friendly graphical interface for converting Markdown to HTML.
*   Allows editing Markdown directly within the GUI and viewing the live HTML output.
*   Includes options to save the generated HTML to a file.

**Additional**

*   Handles file overwrites gracefully, prompting for confirmation before replacing existing files.

## Usage

**Command-Line**

```bash
cargo run -- -i <input_file> [-o <output_file>] [-v] [--heading-class <class_name>] [--paragraph-class <class_name>] [--toc-depth <depth>]