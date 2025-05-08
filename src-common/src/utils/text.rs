use regex::Regex;

/// Wrap text to a specific width
pub fn wrap_text(text: &str, width: usize) -> String {
    let mut result = String::new();
    let mut current_width = 0;
    
    for word in text.split_whitespace() {
        if current_width + word.len() + 1 > width {
            // Add a newline and start a new line
            result.push('\n');
            result.push_str(word);
            current_width = word.len();
        } else if current_width == 0 {
            // First word on the line
            result.push_str(word);
            current_width = word.len();
        } else {
            // Add word with a space
            result.push(' ');
            result.push_str(word);
            current_width += word.len() + 1;
        }
    }
    
    result
}

/// Truncate string to a maximum length with ellipsis
pub fn truncate(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        // Find a good place to truncate - preferably at a word boundary
        let truncate_pos = text[..max_length].rfind(' ').unwrap_or(max_length);
        format!("{}...", &text[..truncate_pos])
    }
}

/// Extract the first line of text
pub fn first_line(text: &str) -> String {
    text.lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

/// Clean text by removing extra whitespace
pub fn clean_text(text: &str) -> String {
    let whitespace_regex = Regex::new(r"[ \t]+").unwrap();
    let newline_regex = Regex::new(r"\n{3,}").unwrap();
    
    let text = whitespace_regex.replace_all(text, " ");
    let text = newline_regex.replace_all(&text, "\n\n");
    
    text.trim().to_string()
}

/// Convert markdown to plain text
pub fn markdown_to_plain(markdown: &str) -> String {
    // This is a very simplified markdown to plain text converter
    let heading_regex = Regex::new(r"^(#+)\s+(.*)$").unwrap();
    let bullet_regex = Regex::new(r"^[-*+]\s+(.*)$").unwrap();
    let numbered_regex = Regex::new(r"^\d+\.\s+(.*)$").unwrap();
    let link_regex = Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap();
    let emphasis_regex = Regex::new(r"(\*\*|__)(.*?)\1").unwrap();
    let italic_regex = Regex::new(r"(\*|_)(.*?)\1").unwrap();
    let code_regex = Regex::new(r"`([^`]+)`").unwrap();
    
    let mut lines: Vec<String> = Vec::new();
    
    for line in markdown.lines() {
        // Process headings
        let line = heading_regex.replace(line, "$2");
        
        // Process bullet points
        let line = bullet_regex.replace(&line, "• $1");
        
        // Process numbered lists
        let line = numbered_regex.replace(&line, "$1");
        
        // Process links
        let line = link_regex.replace_all(&line, "$1");
        
        // Process bold text
        let line = emphasis_regex.replace_all(&line, "$2");
        
        // Process italic text
        let line = italic_regex.replace_all(&line, "$2");
        
        // Process code
        let line = code_regex.replace_all(&line, "$1");
        
        lines.push(line.to_string());
    }
    
    lines.join("\n")
}
