use regex::Regex;

fn main() {
    let system_regex = Regex::new(
        r"├──\s+([a-z_]+)\s*:\s*([\d.]+)ms\s+\((\d+)%\)"
    ).unwrap();
    
    let test_line = "├── ai_planner:      2.1ms ( 40%)";
    
    match system_regex.captures(test_line) {
        Some(caps) => {
            println!("Match found!");
            println!("  System: {}", caps.get(1).map(|m| m.as_str()).unwrap_or(""));
            println!("  MS: {}", caps.get(2).map(|m| m.as_str()).unwrap_or(""));
            println!("  %: {}", caps.get(3).map(|m| m.as_str()).unwrap_or(""));
        }
        None => println!("No match"),
    }
}
