use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "cat")]
struct CatCommand {
    #[arg(short = 'n', long = "number")]
    number: bool,
    #[arg(short = 'b', long = "number-non-blank")]
    number_nonblank: bool,
    #[arg(short = 's', long = "squeeze-blank")]
    squeeze_blank: bool,
    #[arg(short = 'e', long = "show-end")]
    show_end: bool,
    #[arg(short = 'T', long = "show-tab")]
    show_tab: bool,
    #[arg(short = 'v', long = "show-non-print")]
    show_nonprint: bool,
    #[arg(short = 'A', long = "show-all")]
    show_all: bool,
    files: Vec<String>,
}

impl CatCommand {
    fn process_command(&mut self) -> std::io::Result<()> {
        let n_file: usize = self.files.len();
        for file in &self.files {
            if n_file > 1 {
                print!("'{}'", file);
            }
            let lines = self.process_file_content(file)?;
            for line in lines {
                print!("{line}");
            }
        }
        Ok(())
    } 

    /*
    * Process_file_content:
    *   Process each line of a file with the given arguments.
    *
    *      Parameter:
    *          file: reference of the file name
    *
    *      Return:
    *          Result containing a Vector of resulting processed lines as String
    */
    fn process_file_content(&self, file: &str) -> std::io::Result<Vec<String>>{
        let content = std::fs::read_to_string(file)?;
        let mut result: Vec<String> = Vec::new();
        let mut line_number: usize = 1;

        for line in content.split_inclusive('\n') {
            let is_blank = line.trim_end_matches('\n').is_empty();
            result.push(self.process_line(line, line_number, is_blank));
            if !self.number_nonblank || !is_blank {
                line_number += 1;
            }
        }
        Ok(result)
    }

    /*
    * Process_line:
    *   Apply every arguments of the command on the given line.
    *
    *   Parameters:
    *       line: reference to the line that will be processed
    *       line_count: line number of the corresponding line in the file
    *       is_blank: boolean if the line is an empty line or not
    *
    *   Return:
    *       Return the result processed line as a String
    */
    fn process_line(
        &self,
        line: &str,
        line_count: usize,
        is_blank: bool) -> String {
        let mut result: String;
        let mut processed: String = String::from(line);

        if self.number || (self.number_nonblank && !is_blank) {
            result = String::from(&format!("{line_count:>6}   "));
        } else {
            result = String::new();
        }
        if self.show_tab {
            processed = processed.replace('\t', "^I");
        }
        if self.show_end {
            processed = processed.replace('\n', "$\n");
        }
        result.push_str(&processed);
        result
    }

    /*
     * Normalize_opt:
     *      Normalize option that groups multiple smaller option.
     *      Ex: -A which is -veT
     */
    fn normalize_opt(&mut self) {
        if self.show_all {
            self.show_nonprint = true;
            self.show_tab = true;
            self.show_end = true;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = CatCommand::parse();
    args.normalize_opt();
    args.process_command()?;
    Ok(())
}
