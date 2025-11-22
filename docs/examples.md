# Real-World Examples

This document provides real-world configuration examples for common use cases.

## Table of Contents

- [Code Review Documentation](#code-review-documentation)
- [Public Release Package](#public-release-package)
- [Internal Audit Trail](#internal-audit-trail)
- [API Documentation](#api-documentation)
- [Security Review](#security-review)
- [Open Source Contribution](#open-source-contribution)
- [Student Assignment Submission](#student-assignment-submission)
- [Technical Presentation](#technical-presentation)

## Code Review Documentation

Generate PDFs for code review meetings with team annotations.

**Use Case**: Preparing code for review meetings, printing for offline review

**config.yaml:**
```yaml
output:
  mode: single
  directory: ./code-reviews
  filename: review_sprint_42.pdf

files:
  - path: src/features/authentication.rs
    title: "Authentication Module"
  - path: src/features/user_management.rs
    title: "User Management"
  - path: src/api/handlers.rs
    title: "API Handlers"
  - path: tests/integration_tests.rs
    title: "Integration Tests"

syntax_highlighting:
  enabled: true
  theme: InspiredGitHub

page:
  size: Letter
  margins: { top: 2.5, bottom: 2.5, left: 3.0, right: 3.0 }
  font_size: 9
  line_numbers: true
  line_spacing: 1.4

header:
  enabled: true
  left: "Code Review - Sprint 42"
  right: "Page {page}/{total}"
  font_size: 8

footer:
  enabled: true
  left: "Engineering Team"
  center: "{filename}"
  right: "{date}"
  font_size: 8

metadata:
  title: "Sprint 42 Code Review"
  author: "Development Team"
  subject: "Code Review Documentation"
  keywords: ["code review", "sprint 42", "authentication"]
```

**Usage:**
```bash
papercut -c config.yaml -v
```

---

## Public Release Package

Prepare source code for public release with proper disclaimers.

**Use Case**: Government contracts, open-source releases, compliance requirements

**release_config.yaml:**
```yaml
output:
  mode: single
  directory: ./releases/2024
  filename: software_release_v1.0.pdf

files:
  - path: LICENSE
    title: "Software License Agreement"
  - path: README.md
    title: "Release Documentation"
  - path: CHANGELOG.md
    title: "Version History"
  - path: src/main.rs
    title: "Main Application"
  - path: src/lib.rs
    title: "Core Library"
  - path: src/utils.rs
    title: "Utility Functions"

syntax_highlighting:
  enabled: true
  theme: InspiredGitHub

page:
  size: Letter
  margins: { top: 2.5, bottom: 2.5, left: 2.5, right: 2.5 }
  font_size: 10
  line_numbers: true
  line_spacing: 1.3

header:
  enabled: true
  left: "APPROVED FOR PUBLIC RELEASE"
  center: "Software Version 1.0"
  right: "Page {page}/{total}"
  font_size: 8

footer:
  enabled: true
  left: "Distribution Unlimited"
  center: "{filename}"
  right: "Released: {date}"
  font_size: 8

styling:
  font_family: monospace
  background_color: "#ffffff"
  text_color: "#000000"
  line_number_color: "#888888"

metadata:
  title: "Software Source Code - Public Release v1.0"
  author: "Software Engineering Division"
  subject: "Public Release Documentation"
  keywords:
    - public release
    - source code
    - distribution unlimited
    - version 1.0
```

**Usage:**
```bash
papercut -c release_config.yaml
```

---

## Internal Audit Trail

Generate audit documentation for internal compliance.

**Use Case**: SOX compliance, internal audits, security reviews

**audit_config.yaml:**
```yaml
output:
  mode: single
  directory: ./audits/Q4-2024
  filename: security_audit_Q4.pdf

files:
  - path: src/security/authentication.rs
    title: "Authentication Implementation"
  - path: src/security/encryption.rs
    title: "Encryption Module"
  - path: src/security/access_control.rs
    title: "Access Control"
  - path: src/database/connection.rs
    title: "Database Security"
  - path: tests/security_tests.rs
    title: "Security Test Coverage"

syntax_highlighting:
  enabled: true
  theme: Solarized (light)

page:
  size: Legal
  margins: { top: 2.5, bottom: 2.5, left: 3.0, right: 3.0 }
  font_size: 9
  line_numbers: true
  line_spacing: 1.3

header:
  enabled: true
  left: "CONFIDENTIAL - INTERNAL AUDIT"
  center: "Security Review Q4 2024"
  right: "Page {page} of {total}"
  font_size: 8

footer:
  enabled: true
  left: "Compliance Team"
  center: "{filename}"
  right: "Generated: {date}"
  font_size: 8

metadata:
  title: "Q4 2024 Security Audit - Source Code Review"
  author: "Internal Audit Team"
  subject: "SOX Compliance - Security Controls"
  keywords:
    - audit
    - security
    - compliance
    - Q4 2024
    - confidential
```

**Usage:**
```bash
papercut -c audit_config.yaml -v
```

---

## API Documentation

Create PDF documentation of API implementation.

**Use Case**: API reference, implementation documentation, client onboarding

**api_docs_config.yaml:**
```yaml
output:
  mode: multiple
  directory: ./api-docs

files:
  - path: src/api/users.rs
    title: "User API Endpoints"
  - path: src/api/products.rs
    title: "Product API Endpoints"
  - path: src/api/orders.rs
    title: "Order API Endpoints"
  - path: src/api/auth.rs
    title: "Authentication API"
  - path: src/models/schemas.rs
    title: "Data Models & Schemas"

syntax_highlighting:
  enabled: true
  theme: InspiredGitHub

page:
  size: A4
  margins: { top: 2.0, bottom: 2.0, left: 2.0, right: 2.0 }
  font_size: 10
  line_numbers: true
  line_spacing: 1.2

header:
  enabled: true
  left: "API Documentation"
  right: "Page {page}/{total}"
  font_size: 8

footer:
  enabled: true
  left: "Version 2.0"
  center: "{filename}"
  right: "{date}"
  font_size: 8

metadata:
  title: "REST API Implementation Documentation"
  author: "API Team"
  subject: "API Reference"
  keywords: ["API", "REST", "documentation", "implementation"]
```

**Usage:**
```bash
papercut -c api_docs_config.yaml
# Generates: users.pdf, products.pdf, orders.pdf, auth.pdf, schemas.pdf
```

---

## Security Review

Prepare code for security team review with emphasis on security-critical code.

**Use Case**: Penetration testing preparation, security audits, vulnerability assessments

**security_review_config.yaml:**
```yaml
output:
  mode: single
  directory: ./security-reviews
  filename: security_review_2024-11.pdf

files:
  - path: src/security/auth.rs
    title: "Authentication & Authorization"
  - path: src/security/crypto.rs
    title: "Cryptographic Functions"
  - path: src/security/input_validation.rs
    title: "Input Validation & Sanitization"
  - path: src/database/queries.rs
    title: "Database Queries (SQL Injection Check)"
  - path: src/api/middleware.rs
    title: "Security Middleware"
  - path: .env.example
    title: "Environment Configuration Template"

syntax_highlighting:
  enabled: true
  theme: base16-ocean.dark

page:
  size: Letter
  margins: { top: 2.5, bottom: 2.5, left: 2.5, right: 2.5 }
  font_size: 9
  line_numbers: true
  line_spacing: 1.3

header:
  enabled: true
  left: "CONFIDENTIAL - SECURITY REVIEW"
  center: "Vulnerability Assessment"
  right: "Page {page}/{total}"
  font_size: 8

footer:
  enabled: true
  left: "Security Team Only"
  center: "{filename}"
  right: "{date}"
  font_size: 8

styling:
  font_family: monospace
  background_color: "#ffffff"
  text_color: "#000000"
  line_number_color: "#888888"

metadata:
  title: "Security Review - November 2024"
  author: "Security Engineering Team"
  subject: "Vulnerability Assessment & Code Review"
  keywords:
    - security
    - vulnerability
    - penetration testing
    - confidential
```

**Usage:**
```bash
papercut -c security_review_config.yaml -v
```

---

## Open Source Contribution

Document contributions to open-source projects.

**Use Case**: Contribution documentation, portfolio building, community submissions

**contribution_config.yaml:**
```yaml
output:
  mode: single
  directory: ./contributions
  filename: rust_contribution_2024.pdf

files:
  - path: CONTRIBUTING.md
    title: "Contribution Guidelines"
  - path: src/feature/new_parser.rs
    title: "New Parser Implementation"
  - path: src/feature/optimizer.rs
    title: "Performance Optimizer"
  - path: tests/parser_tests.rs
    title: "Parser Test Suite"
  - path: benches/benchmarks.rs
    title: "Performance Benchmarks"

syntax_highlighting:
  enabled: true
  theme: InspiredGitHub

page:
  size: A4
  margins: { top: 2.0, bottom: 2.0, left: 2.0, right: 2.0 }
  font_size: 10
  line_numbers: true
  line_spacing: 1.25

header:
  enabled: true
  left: "Open Source Contribution"
  right: "Page {page}/{total}"
  font_size: 8

footer:
  enabled: true
  left: "Your Name"
  center: "{filename}"
  right: "{date}"
  font_size: 8

metadata:
  title: "Rust Project Contribution - Parser & Optimizer"
  author: "Your Name"
  subject: "Open Source Contribution Documentation"
  keywords: ["open source", "rust", "parser", "contribution"]
```

**Usage:**
```bash
papercut -c contribution_config.yaml
```

---

## Student Assignment Submission

Submit programming assignments in PDF format for grading.

**Use Case**: University assignments, coding bootcamps, certification submissions

**assignment_config.yaml:**
```yaml
output:
  mode: single
  directory: ./submissions
  filename: CS401_Assignment3_StudentID.pdf

files:
  - path: README.md
    title: "Assignment Overview"
  - path: src/main.rs
    title: "Main Program"
  - path: src/algorithms/sorting.rs
    title: "Sorting Algorithms Implementation"
  - path: src/algorithms/searching.rs
    title: "Searching Algorithms Implementation"
  - path: tests/test_suite.rs
    title: "Test Cases & Results"
  - path: docs/analysis.md
    title: "Performance Analysis"

syntax_highlighting:
  enabled: true
  theme: InspiredGitHub

page:
  size: Letter
  margins: { top: 2.5, bottom: 2.5, left: 2.5, right: 2.5 }
  font_size: 10
  line_numbers: true
  line_spacing: 1.4

header:
  enabled: true
  left: "CS 401 - Algorithms"
  center: "Assignment 3: Sorting & Searching"
  right: "Page {page}/{total}"
  font_size: 8

footer:
  enabled: true
  left: "Student Name - ID: 12345678"
  center: "{filename}"
  right: "Submitted: {date}"
  font_size: 8

metadata:
  title: "CS 401 Assignment 3 - Algorithms Implementation"
  author: "Student Name (ID: 12345678)"
  subject: "Computer Science Assignment"
  keywords: ["algorithms", "sorting", "searching", "assignment"]
```

**Usage:**
```bash
papercut -c assignment_config.yaml
```

---

## Technical Presentation

Create code listings for technical presentations and talks.

**Use Case**: Conference talks, internal tech talks, training materials

**presentation_config.yaml:**
```yaml
output:
  mode: multiple
  directory: ./presentation-handouts

files:
  - path: examples/example1_basic.rs
    title: "Example 1: Basic Usage"
  - path: examples/example2_advanced.rs
    title: "Example 2: Advanced Features"
  - path: examples/example3_performance.rs
    title: "Example 3: Performance Optimization"

syntax_highlighting:
  enabled: true
  theme: Solarized (light)

page:
  size: Letter
  margins: { top: 3.0, bottom: 3.0, left: 3.0, right: 3.0 }
  font_size: 11
  line_numbers: false
  line_spacing: 1.5

header:
  enabled: true
  center: "Rust Performance Patterns - RustConf 2024"
  right: "{page}"
  font_size: 10

footer:
  enabled: true
  center: "https://github.com/username/rust-patterns"
  font_size: 9

styling:
  font_family: dejavu
  background_color: "#ffffff"
  text_color: "#000000"

metadata:
  title: "Rust Performance Patterns - Code Examples"
  author: "Speaker Name"
  subject: "RustConf 2024 Presentation"
  keywords: ["rust", "performance", "conference", "presentation"]
```

**Usage:**
```bash
papercut -c presentation_config.yaml
# Generates: example1_basic.pdf, example2_advanced.pdf, example3_performance.pdf
```

---

## Quick Reference

### Command Patterns

```bash
# Single PDF with verbose output
papercut -c config.yaml -v

# Multiple PDFs (quiet)
papercut -c config.yaml

# Check available themes first
papercut --list-themes

# Version info
papercut --version
```

### Common Configuration Patterns

```yaml
# Minimal configuration
output:
  mode: single
files:
  - path: src/main.rs

# Maximum readability
page:
  font_size: 12
  line_spacing: 1.5
  line_numbers: false

# Compact output
page:
  font_size: 8
  line_spacing: 1.0
  margins: { top: 1.5, bottom: 1.5, left: 1.5, right: 1.5 }

# Professional headers/footers
header:
  enabled: true
  left: "Document Title"
  right: "Page {page}/{total}"
footer:
  enabled: true
  left: "Company Name"
  center: "{filename}"
  right: "{date}"
```

## See Also

- [Configuration Reference](configuration.md) - Complete configuration options
- [Usage Guide](usage.md) - CLI usage instructions
- [Styling Guide](styling.md) - Advanced styling options
