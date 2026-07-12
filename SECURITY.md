# Security

RegexBench processes user-provided regex patterns and input strings.

## Input Validation
- All regex patterns are parsed with bounds checking
- Input strings are processed character by character
- No arbitrary code execution from patterns

## Resource Limits
- Patterns have a maximum complexity score
- Matching has a step limit to prevent infinite loops
- Benchmark iterations are configurable

## Dependencies
- All dependencies are pinned to specific versions
- No network access required for core functionality
- No file system access except for test file input

## Reporting
If you discover a security vulnerability, please report it responsibly.
