# Project Report

This document demonstrates the markdown rendering capabilities of Papercut.

![Papercut Logo](../../assets/logos/papercut_logo.svg)

## Introduction

Papercut can now render markdown documents directly into PDF output alongside source code listings. This enables creating comprehensive technical documentation that combines narrative text with code.

## Features

### Text Formatting

Markdown supports various text formatting options:

- **Bold text** for emphasis
- *Italic text* for subtle emphasis
- `Inline code` for technical terms
- ~~Strikethrough~~ for deprecated content

### Lists

Ordered lists work well for step-by-step instructions:

1. First step
2. Second step
3. Third step

Unordered lists are great for features:

- Feature one
- Feature two
- Feature three
  - Nested item A
  - Nested item B

### Code Blocks

Code blocks are rendered with a monospace font and light background:

```rust
fn main() {
    println!("Hello from Papercut!");

    let numbers = vec![1, 2, 3, 4, 5];
    for n in numbers {
        println!("Number: {}", n);
    }
}
```

### Blockquotes

> Blockquotes are useful for highlighting important information
> or quoting external sources. They are rendered with a left border
> to visually distinguish them from regular text.

### Links

For more information, visit [Papercut on GitHub](https://github.com/chrislupp/papercut).

---

## Conclusion

The markdown report feature provides a powerful way to include documentation directly in your PDF output without requiring external tools or PDF merging.

### Future Extensions

Math support is designed as a future extension point. Currently, math expressions like $E = mc^2$ are rendered as inline code.

---
