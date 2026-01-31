// Sample Rust file for Papercut markdown report example
//
// This file demonstrates how source code is rendered alongside
// the markdown report in the final PDF output.

fn main() {
    println!("Hello from Papercut!");

    let message = create_greeting("World");
    println!("{}", message);

    demonstrate_features();
}

/// Creates a greeting message for the given name
fn create_greeting(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Demonstrates various Rust features for syntax highlighting
fn demonstrate_features() {
    // Variables and types
    let count: i32 = 42;
    let pi: f64 = 3.14159;
    let is_active: bool = true;

    // Collections
    let numbers = vec![1, 2, 3, 4, 5];
    let mut map = std::collections::HashMap::new();
    map.insert("key", "value");

    // Control flow
    for (i, num) in numbers.iter().enumerate() {
        if *num % 2 == 0 {
            println!("Index {}: {} is even", i, num);
        } else {
            println!("Index {}: {} is odd", i, num);
        }
    }

    // Pattern matching
    match count {
        0 => println!("Zero"),
        1..=10 => println!("Small number"),
        _ => println!("Large number: {}", count),
    }

    println!("Pi: {}, Active: {}", pi, is_active);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeting() {
        let result = create_greeting("Test");
        assert_eq!(result, "Hello, Test!");
    }
}
