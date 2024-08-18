# Markdown to HTML Converter

A simple command-line tool written in Rust that converts Markdown files to HTML.

## Features

* Converts basic Markdown elements:
    * Headings
    * Paragraphs
    * Bold, italic, strikethrough, inline text
    * Links
    * Images
    * Lists (ordered, unordered and sub)
    * Code blocks
    * Blockquotes
    * Tables
    * Horizontal rules
* Handles nested lists correctly
* Escapes HTML entities to prevent XSS vulnerabilities
* Generates a table of contents based on headings
* Allows customization of CSS classes for headings and paragraphs
* Provides verbose output for debugging and monitoring