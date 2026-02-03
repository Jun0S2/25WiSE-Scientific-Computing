use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Debug)]
struct BasicInterpreter {
    variables: HashMap<String, f64>,      // Variable storage (name -> value)
    line_numbers: Vec<i32>,               // Sorted list of line numbers
    program: HashMap<i32, String>,        // Program lines (line number -> code)
    current_line: i32,                    // Current line being executed
    return_stack: Vec<i32>,               // Stack for GOSUB/RETURN
    running: bool,                        // Program execution flag
}

impl BasicInterpreter {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            line_numbers: Vec::new(),
            program: HashMap::new(),
            current_line: 0,
            return_stack: Vec::new(),
            running: false,
        }
    }

    // Load program from string (parses line numbers and code)
    fn load_program(&mut self, program: &str) {
        self.program.clear();
        self.line_numbers.clear();

        for line in program.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse line number and code
            if let Some(space_pos) = line.find(' ') {
                if let Ok(line_num) = line[..space_pos].parse::<i32>() {
                    self.program.insert(line_num, line[space_pos + 1..].to_string());
                    self.line_numbers.push(line_num);
                }
            }
        }

        self.line_numbers.sort();  // Sort line numbers for sequential execution
    }

    // Main execution loop
    fn run(&mut self) {
        self.running = true;
        self.current_line = *self.line_numbers.first().unwrap_or(&0);

        while self.running && self.current_line > 0 {
            let line_content = if let Some(line) = self.program.get(&self.current_line) {
                line.clone()
            } else {
                self.running = false;
                continue;
            };

            self.execute_line(&line_content);
        }
    }

    // Execute a single line of BASIC code
    fn execute_line(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            self.goto_next_line();
            return;
        }

        let mut parts = line.split_whitespace();
        if let Some(cmd) = parts.next() {
            match cmd {
                "REM" => self.goto_next_line(),  // Comment - skip to next line
                
                "PRINT" => {
                    let output = line[5..].trim_start();
                    let evaluated = self.evaluate_print_output(output);
                    println!("{}", evaluated);
                    self.goto_next_line();
                }
                
                "LET" => {
                    if let Some(eq_pos) = line.find('=') {
                        let left = line[3..eq_pos].trim().to_string();
                        let right = line[eq_pos + 1..].trim();
                        let value = self.evaluate_expression(right);
                        
                        // Only clamp height (H) to non-negative, velocity (V) can be negative
                        let final_value = if left == "H" {
                            value.max(0.0)
                        } else {
                            value
                        };
                        
                        self.variables.insert(left, final_value);
                    }
                    self.goto_next_line();
                }
                
                "GOTO" => {
                    if let Some(token) = parts.next() {
                        if let Ok(line_num) = token.parse::<i32>() {
                            self.current_line = line_num;
                            return;
                        }
                    }
                    self.goto_next_line();
                }
                
                "GOSUB" => {
                    if let Some(token) = parts.next() {
                        if let Ok(line_num) = token.parse::<i32>() {
                            // Push return address (next line) to stack
                            if let Some(pos) = self.line_numbers.iter().position(|&x| x == self.current_line) {
                                if pos + 1 < self.line_numbers.len() {
                                    self.return_stack.push(self.line_numbers[pos + 1]);
                                } else {
                                    self.return_stack.push(0);
                                }
                            }
                            self.current_line = line_num;
                            return;
                        }
                    }
                    self.goto_next_line();
                }
                
                "RETURN" => {
                    if let Some(return_line) = self.return_stack.pop() {
                        if return_line == 0 {
                            self.running = false;
                        } else {
                            self.current_line = return_line;
                        }
                    } else {
                        self.running = false;
                    }
                }
                
                "IF" => {
                    // Parse condition and THEN part
                    let if_part = &line[2..].trim();
                    
                    if let Some(then_pos) = if_part.find("THEN") {
                        let condition = &if_part[..then_pos].trim();
                        let then_part = &if_part[then_pos + 4..].trim();
                        
                        if self.evaluate_condition(condition) {
                            if let Ok(line_num) = then_part.parse::<i32>() {
                                self.current_line = line_num;
                                return;
                            }
                        }
                    }
                    self.goto_next_line();
                }
                
                "INPUT" => {
                    let rest = line[5..].trim_start();
                    self.handle_input(rest);
                    self.goto_next_line();
                    return;
                }
                
                "END" => {
                    self.running = false;
                }
                
                _ => {
                    // Implicit LET for assignments without LET keyword
                    if line.contains('=') {
                        self.execute_line(&format!("LET {}", line));
                    } else {
                        self.goto_next_line();
                    }
                }
            }
        } else {
            self.goto_next_line();
        }
    }

    // Handle INPUT statement with optional prompt
    fn handle_input(&mut self, input_spec: &str) {
        let spec = input_spec.trim();
        let var_name: String;
        
        if spec.starts_with('"') {
            // INPUT with prompt: INPUT "prompt"; variable
            if let Some(end_quote) = spec[1..].find('"') {
                let prompt = &spec[1..1 + end_quote];
                let after = spec[1 + end_quote + 1..].trim_start();
                var_name = if after.starts_with(';') { 
                    after[1..].to_string() 
                } else { 
                    after.to_string() 
                };
                print!("{}", prompt);
            } else {
                var_name = spec.to_string();
                print!("? ");
            }
        } else {
            // INPUT without prompt: INPUT variable
            var_name = spec.to_string();
            print!("? ");
        }
    
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    
        if let Ok(num) = input.trim().parse::<f64>() {
            self.variables.insert(var_name.clone(), num);
        }
    
        // Fuel limit check (custom extension)
        if var_name == "U" {
            let f_val = *self.variables.get("F").unwrap_or(&0.0);
            if let Some(u) = self.variables.get_mut("U") {
                if *u > f_val {
                    println!("Not enough fuel, using remaining fuel: {}", f_val);
                    *u = f_val;
                }
            }
        }
    }

    // Move to next line in program
    fn goto_next_line(&mut self) {
        if let Some(pos) = self.line_numbers.iter().position(|&x| x == self.current_line) {
            if pos + 1 < self.line_numbers.len() {
                self.current_line = self.line_numbers[pos + 1];
            } else {
                self.running = false;
            }
        } else {
            self.running = false;
        }
    }

    // Evaluate PRINT statement output (handles strings and variables)
    fn evaluate_print_output(&self, output: &str) -> String {
        let mut result = String::new();
        let mut chars = output.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                '"' => {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        if ch == '"' { chars.next(); break; }
                        result.push(ch);
                        chars.next();
                    }
                }
                ';' => { chars.next(); }  // Semicolon for concatenation
                _ => {
                    let mut token = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == ';' || ch == '"' { break; }
                        token.push(ch);
                        chars.next();
                    }
                    let token = token.trim();
                    if !token.is_empty() {
                        if let Some(&v) = self.variables.get(token) {
                            // Format integer values without decimal
                            if (v - v.trunc()).abs() < 1e-9 {
                                result.push_str(&format!("{}", v as i64));
                            } else {
                                result.push_str(&format!("{}", v));
                            }
                        } else {
                            result.push_str(token);
                        }
                    }
                }
            }
        }

        result
    }

    // Evaluate arithmetic expression
    fn evaluate_expression(&self, expr: &str) -> f64 {
        let mut parser = ExprParser::new(expr, &self.variables);
        parser.parse_expression()
    }

    // Evaluate conditional expression (for IF statements)
    fn evaluate_condition(&self, condition: &str) -> bool {
        let cond_no_spaces = condition.replace(' ', "");
        
        // Parse comparison operators
        if let Some(pos) = cond_no_spaces.find("<>") {  // Not equal
            let left = &cond_no_spaces[..pos];
            let right = &cond_no_spaces[pos + 2..];
            let left_val = self.evaluate_expression(left);
            let right_val = self.evaluate_expression(right);
            return (left_val - right_val).abs() > 1e-9;
        }
        
        if let Some(pos) = cond_no_spaces.find("<=") {  // Less than or equal
            let left = &cond_no_spaces[..pos];
            let right = &cond_no_spaces[pos + 2..];
            let left_val = self.evaluate_expression(left);
            let right_val = self.evaluate_expression(right);
            return left_val <= right_val;
        }
        
        if let Some(pos) = cond_no_spaces.find('<') {  // Less than
            // Check it's not part of <=
            if pos == 0 || cond_no_spaces.chars().nth(pos - 1) != Some('=') {
                let left = &cond_no_spaces[..pos];
                let right = &cond_no_spaces[pos + 1..];
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                return left_val < right_val;
            }
        }
        
        if let Some(pos) = cond_no_spaces.find(">=") {  // Greater than or equal
            let left = &cond_no_spaces[..pos];
            let right = &cond_no_spaces[pos + 2..];
            let left_val = self.evaluate_expression(left);
            let right_val = self.evaluate_expression(right);
            return left_val >= right_val;
        }
        
        if let Some(pos) = cond_no_spaces.find('>') {  // Greater than
            // Check it's not part of >=
            if pos == 0 || cond_no_spaces.chars().nth(pos - 1) != Some('=') {
                let left = &cond_no_spaces[..pos];
                let right = &cond_no_spaces[pos + 1..];
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                return left_val > right_val;
            }
        }
        
        if let Some(pos) = cond_no_spaces.find('=') {  // Equal
            // Check it's not part of <=, >=, or <>
            if pos > 0 && cond_no_spaces.chars().nth(pos - 1) != Some('<') 
                && cond_no_spaces.chars().nth(pos - 1) != Some('>') {
                let left = &cond_no_spaces[..pos];
                let right = &cond_no_spaces[pos + 1..];
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                return (left_val - right_val).abs() < 1e-9;
            }
        }
        
        false
    }
}

// Parser for arithmetic expressions
struct ExprParser<'a> {
    s: &'a str,                          // Input string
    pos: usize,                          // Current position in string
    vars: &'a HashMap<String, f64>,      // Variables for lookup
}

impl<'a> ExprParser<'a> {
    fn new(s: &'a str, vars: &'a HashMap<String, f64>) -> Self {
        Self { s, pos: 0, vars }
    }

    // Peek at next character
    fn peek(&self) -> Option<char> {
        self.s[self.pos..].chars().next()
    }

    // Skip whitespace
    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() { self.pos += c.len_utf8(); } else { break; }
        }
    }

    // Parse expression (addition/subtraction)
    fn parse_expression(&mut self) -> f64 {
        let mut value = self.parse_term();
        loop {
            self.consume_whitespace();
            if let Some(op) = self.peek() {
                if op == '+' || op == '-' {
                    self.pos += op.len_utf8();
                    let rhs = self.parse_term();
                    if op == '+' { value += rhs; } else { value -= rhs; }
                    continue;
                }
            }
            break;
        }
        value
    }

    // Parse term (multiplication/division)
    fn parse_term(&mut self) -> f64 {
        let mut value = self.parse_factor();
        loop {
            self.consume_whitespace();
            if let Some(op) = self.peek() {
                if op == '*' || op == '/' {
                    self.pos += op.len_utf8();
                    let rhs = self.parse_factor();
                    if op == '*' { value *= rhs; } else { value /= rhs; }
                    continue;
                }
            }
            break;
        }
        value
    }

    // Parse factor (numbers, variables, parentheses, unary minus)
    fn parse_factor(&mut self) -> f64 {
        self.consume_whitespace();
        if let Some(c) = self.peek() {
            if c == '(' {
                self.pos += 1;
                let v = self.parse_expression();
                self.consume_whitespace();
                if self.peek() == Some(')') { self.pos += 1; }
                return v;
            }
            if c == '-' {
                self.pos += 1;
                return -self.parse_factor();
            }

            // Parse number or variable name
            let start = self.pos;
            while let Some(ch) = self.peek() {
                if ch.is_alphanumeric() || ch == '.' || ch == '_' { 
                    self.pos += ch.len_utf8(); 
                } else { 
                    break; 
                }
            }
            let token_str = &self.s[start..self.pos].trim();
            if token_str.is_empty() { return 0.0; }

            // Try parsing as number
            if let Ok(n) = token_str.parse::<f64>() { 
                return n; 
            }

            // Look up as variable
            let token = token_str.to_string();
            if let Some(v) = self.vars.get(&token) { 
                return *v; 
            }

            0.0  // Default value if variable not found
        } else { 
            0.0 
        }
    }
}

fn main() {
    // Original Lunar Lander program (1979) adapted for our interpreter
    let lunar_lander_program = "
10 REM LUNAR LANDER
20 REM Based on the 1979 BASIC version
30 REM Adapted for this interpreter
40 REM
50 REM Game variables:
60 REM   H = Height (meters)
70 REM   V = Velocity (m/sec, positive = downward)
80 REM   F = Fuel remaining (gallons)
90 REM   G = Gravity (2 m/sec^2 on the Moon)
100 REM   U = Fuel usage this turn
110 REM
120 PRINT \"LUNAR LANDER\"
130 PRINT \"============\"
140 PRINT
150 PRINT \"You are aboard the Lunar Lander.\"
160 PRINT \"Try to land with velocity less than 5 m/sec.\"
170 PRINT
180 REM Initialize game state
190 LET V = 70      // Initial velocity (downward)
200 LET F = 500     // Initial fuel
210 LET H = 1000    // Initial height
220 LET G = 2       // Moon gravity
230 REM Main game loop
240 PRINT \"Meter readings:\"
250 PRINT \"--------------\"
260 PRINT \"Fuel (gal):\"; F
270 PRINT \"Velocity (m/sec):\"; V
280 PRINT \"Height (m):\"; H
290 PRINT
300 PRINT \"How much fuel will you use?\"
310 INPUT U
320 REM Validate fuel input
330 IF U < 0 THEN 390      // Negative fuel not allowed
340 IF U > F THEN LET U = F  // Can't use more than available
350 REM Update game state
360 LET F = F - U
370 LET V = V + G - U * 0.2  // Physics: gravity + engine thrust
380 LET H = H - V            // Update height based on velocity
390 REM Check if still in flight
400 IF H > 0 THEN 240
410 REM Landing sequence
420 LET H = 0                // Clamp height to 0
430 PRINT
440 PRINT \"LANDING SEQUENCE\"
450 PRINT \"----------------\"
460 PRINT \"Final readings:\"
470 PRINT \"Fuel (gal):\"; F
480 PRINT \"Landing velocity (m/sec):\"; V
490 PRINT \"Height (m):\"; H
500 PRINT
510 REM Determine landing outcome
520 IF V > 5 THEN 560       // Crashed if velocity > 5 m/sec
530 PRINT \"Congratulations! This was a very good landing.\"
540 GOTO 580
560 PRINT \"You have crashed! Landing velocity was\"; V; \"m/sec.\"
580 PRINT
590 REM Play again option
600 PRINT \"Do you want to play again? (1=yes, 0=no)\"
610 INPUT A
620 IF A = 1 THEN 190       // Restart game
630 PRINT \"Thanks for playing!\"
640 PRINT \"Goodbye!\"
650 END
";

    let mut interpreter = BasicInterpreter::new();
    interpreter.load_program(lunar_lander_program);
    interpreter.run();
}