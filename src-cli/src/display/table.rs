use console::{style, Style, Term};
use std::io;

/// Table column definition
pub struct TableColumn {
    /// Column title
    pub title: String,
    
    /// Column width
    pub width: usize,
    
    /// Column style
    pub style: Option<Style>,
}

/// Print a table to the terminal
pub fn print_table(columns: &[TableColumn], rows: &[Vec<String>]) -> io::Result<()> {
    let term = Term::stdout();
    
    // Print header
    let mut header_line = String::new();
    
    for (i, column) in columns.iter().enumerate() {
        let title = format!("{:<width$}", column.title, width = column.width);
        
        if let Some(ref style) = column.style {
            header_line.push_str(&format!("{}", style.apply_to(title)));
        } else {
            header_line.push_str(&title);
        }
        
        if i < columns.len() - 1 {
            header_line.push_str("  ");
        }
    }
    
    term.write_line(&header_line)?;
    
    // Print separator
    let sep_line: String = header_line
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { '─' })
        .collect();
    
    term.write_line(&sep_line)?;
    
    // Print rows
    for row in rows {
        let mut row_line = String::new();
        
        for (i, (column, value)) in columns.iter().zip(row.iter()).enumerate() {
            // Truncate value if it's too long
            let truncated_value = if value.len() > column.width {
                format!("{}…", &value[0..column.width - 1])
            } else {
                value.clone()
            };
            
            let padded_value = format!("{:<width$}", truncated_value, width = column.width);
            
            // Apply style if provided
            if let Some(ref style) = column.style {
                row_line.push_str(&format!("{}", style.apply_to(padded_value)));
            } else {
                row_line.push_str(&padded_value);
            }
            
            if i < columns.len() - 1 {
                row_line.push_str("  ");
            }
        }
        
        term.write_line(&row_line)?;
    }
    
    Ok(())
}
