/// Export regex patterns to other programming languages
///
/// Converts a regex pattern to equivalent code in various languages.
use serde::{Deserialize, Serialize};

/// Target language for export
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Python,
    JavaScript,
    Go,
    Rust,
    Java,
    CSharp,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Python => write!(f, "Python"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::Go => write!(f, "Go"),
            Language::Rust => write!(f, "Rust"),
            Language::Java => write!(f, "Java"),
            Language::CSharp => write!(f, "C#"),
        }
    }
}

/// Export a regex pattern to the specified language
pub fn export(pattern: &str, lang: Language) -> crate::Result<String> {
    match lang {
        Language::Python => export_python(pattern),
        Language::JavaScript => export_javascript(pattern),
        Language::Go => export_go(pattern),
        Language::Rust => export_rust(pattern),
        Language::Java => export_java(pattern),
        Language::CSharp => export_csharp(pattern),
    }
}

fn export_python(pattern: &str) -> crate::Result<String> {
    let escaped = escape_for_string(pattern);
    Ok(format!(
        r#"import re

pattern = re.compile(r"{escaped}")

# Check for match
if pattern.search(text):
    print("Match found!")

# Find all matches
matches = pattern.findall(text)

# Find match with groups
match = pattern.search(text)
if match:
    print(f"Full match: {{match.group()}}")
    print(f"Groups: {{match.groups()}}")"#
    ))
}

fn export_javascript(pattern: &str) -> crate::Result<String> {
    let escaped = pattern.replace('/', r"\/");
    Ok(format!(
        r#"const pattern = /{escaped}/;

// Check for match
if (pattern.test(text)) {{
    console.log("Match found!");
}}

// Find all matches
const matches = text.match(pattern);

// Find with groups
const match = pattern.exec(text);
if (match) {{
    console.log(`Full match: ${{match[0]}}`);
    console.log(`Groups: ${{match.slice(1)}}`);
}}"#
    ))
}

fn export_go(pattern: &str) -> crate::Result<String> {
    let _escaped = pattern.replace('\\', r"\\").replace('"', r#"\""#);
    Ok(format!(
        r#"package main

import (
    "fmt"
    "regexp"
)

func main() {{
    re := regexp.MustCompile(`{pattern}`)

    // Check for match
    if re.MatchString(text) {{
        fmt.Println("Match found!")
    }}

    // Find all matches
    matches := re.FindAllString(text, -1)

    // Find with groups
    match := re.FindStringSubmatch(text)
    if match != nil {{
        fmt.Printf("Full match: %s\\n", match[0])
        fmt.Printf("Groups: %v\\n", match[1:])
    }}
}}"#
    ))
}

fn export_rust(pattern: &str) -> crate::Result<String> {
    let _escaped = pattern.replace('\\', r"\\").replace('"', r#"\""#);
    Ok(format!(
        r#"use regex::Regex;

fn main() {{
    let re = Regex::new(r"{pattern}").unwrap();

    // Check for match
    if re.is_match("text") {{
        println!("Match found!");
    }}

    // Find all matches
    let matches: Vec<&str> = re.find_iter("text").map(|m| m.as_str()).collect();

    // Find with groups
    if let Some(caps) = re.captures("text") {{
        println!("Full match: {{}}", caps.get(0).unwrap().as_str());
        for i in 1..caps.len() {{
            println!("Group {{}}: {{}}", i, caps.get(i).unwrap().as_str());
        }}
    }}
}}"#
    ))
}

fn export_java(pattern: &str) -> crate::Result<String> {
    let escaped = pattern.replace('\\', r"\\").replace('"', r#"\""#);
    Ok(format!(
        r#"import java.util.regex.*;

public class RegexExample {{
    public static void main(String[] args) {{
        String text = "test string";
        Pattern pattern = Pattern.compile("{escaped}");

        // Check for match
        Matcher matcher = pattern.matcher(text);
        if (matcher.find()) {{
            System.out.println("Match found!");
        }}

        // Find all matches
        while (matcher.find()) {{
            System.out.println("Match: " + matcher.group());
        }}

        // Find with groups
        matcher.reset();
        if (matcher.find()) {{
            System.out.println("Full match: " + matcher.group(0));
            for (int i = 1; i <= matcher.groupCount(); i++) {{
                System.out.println("Group " + i + ": " + matcher.group(i));
            }}
        }}
    }}
}}"#
    ))
}

fn export_csharp(pattern: &str) -> crate::Result<String> {
    let escaped = pattern.replace('\\', r"\\").replace('"', r#"\""#);
    Ok(format!(
        r#"using System;
using System.Text.RegularExpressions;

class Program
{{
    static void Main()
    {{
        string text = "test string";
        var regex = new Regex(@"{escaped}");

        // Check for match
        if (regex.IsMatch(text))
        {{
            Console.WriteLine("Match found!");
        }}

        // Find all matches
        MatchCollection matches = regex.Matches(text);
        foreach (Match match in matches)
        {{
            Console.WriteLine($"Match: {{match.Value}}");
        }}

        // Find with groups
        Match m = regex.Match(text);
        if (m.Success)
        {{
            Console.WriteLine($"Full match: {{m.Groups[0].Value}}");
            for (int i = 1; i < m.Groups.Count; i++)
            {{
                Console.WriteLine($"Group {{i}}: {{m.Groups[i].Value}}");
            }}
        }}
    }}
}}"#
    ))
}

fn escape_for_string(s: &str) -> String {
    s.replace('\\', r"\\")
        .replace('"', r#"\""#)
        .replace('\n', r"\n")
        .replace('\r', r"\r")
        .replace('\t', r"\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_python() {
        let result = export(r"\d+", Language::Python).unwrap();
        assert!(result.contains("import re"));
        assert!(result.contains(r"\d+"));
    }

    #[test]
    fn test_export_javascript() {
        let result = export(r"\d+", Language::JavaScript).unwrap();
        assert!(result.contains("const pattern"));
        assert!(result.contains(r"\d+"));
    }

    #[test]
    fn test_export_go() {
        let result = export(r"\d+", Language::Go).unwrap();
        assert!(result.contains("regexp.MustCompile"));
    }

    #[test]
    fn test_export_rust() {
        let result = export(r"\d+", Language::Rust).unwrap();
        assert!(result.contains("Regex::new"));
    }
}
