#[derive(Debug, PartialEq)]
pub struct AwkArgs {
    pub dump: bool,
    pub program: ProgramType,
    pub files: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum ProgramType {
    CLI(String),
    File(String),
}

impl ProgramType {
    pub fn load(&self) -> Result<String, String> {
        match self {
            ProgramType::CLI(s) => Ok(s.clone()),
            ProgramType::File(s) => {
                match std::fs::read_to_string(s) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(format!("Unable to load source program '{}'\nGot error: {}", s, e)),
                }
            }
        }
    }
}

fn print_help() {
    eprintln!("Usage: llawk [--dump] -f progfile file ...");
    eprintln!("Usage: llawk [--dump] 'program' file ...");
}

impl AwkArgs {
    pub fn new(args: Vec<String>) -> Result<Self, ()> {
        let mut dump = false;
        let mut program: Option<ProgramType> = None;
        let mut files: Vec<String> = vec![];

        let mut i = 1;
        while (i < args.len()) {
            match &args[i][..] {
                "--dump" => {
                    dump = true;
                    i += 1;
                }
                "-f" => {
                    if program != None {
                        print_help();
                        eprintln!("Cannot specify multiple programs!");
                        return Err(());
                    }
                    let next = match args.get(i + 1) {
                        None => {
                            print_help();
                            eprint!("-f must be followed by a file name\n");
                            return Err(());
                        }
                        Some(path) => path,
                    };
                    program = Some(ProgramType::File(next.to_string()));
                    i += 2;
                }
                _ => {
                    if program == None {
                        program = Some(ProgramType::CLI(args[i].clone()));
                    } else {
                        files.push(args[i].clone());
                    }
                    i += 1;
                }
            }
        }
        let program = match program {
            None => {
                print_help();
                eprintln!("You must specify a program either if -f file.awk or as an arg '$1 == 0 {{ print $1 }}'");
                return Err(());
            }
            Some(prog) => prog
        };
        let mut final_args = AwkArgs { dump, program, files };
        Ok(final_args)
    }
}