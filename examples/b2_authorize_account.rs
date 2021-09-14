use backblaze_b2_async_plain::v2::b2_authorize_account;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "b2_authorize_account", about = "Calls b2_authorize_account")]
struct Params {
    #[structopt(short, long, env = "B2_APPLICATION_KEY_ID")]
    application_key_id: Option<String>,
    //to avoid storing it in the shell history there is intentionally no field for application_key here
    #[structopt(long)]
    /// save the received authentication data for later usage to file specified by save_file (default: ~/.b2_auth.yaml)
    save: bool,

    #[structopt(long)]
    /// file to store the authentication data into, requires --save otherwise nothing will be saved
    save_file: Option<String>,
}

/// reads a single line, fails with error messages if that does not work
fn readline(stdin: &io::Stdin) -> String {
    let res = stdin
        .lock()
        .lines()
        .next()
        .expect("No input detected")
        .expect("Error reading input");
    println!(); //insert line break
    res
}

#[tokio::main]
/// WARNING: this example uses blocking stdin/out without generating a separate thread this is generally a bad idea, but
/// done here to keep the example simple
async fn main() {
    let p = Params::from_args();
    let save = if p.save {
        let output_path: PathBuf = match p.save_file {
            Some(path) => PathBuf::from(path),
            None => {
                let mut home = home::home_dir().expect("Could not get home directory. Please specify path for storing the value using --save-file");
                home.push(".b2_auth.yaml");
                home
            }
        };
        Some(output_path)
    } else {
        None
    };

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let application_key_id = match p.application_key_id {
        Some(key_id) => key_id,
        None => {
            write!(stdout, "Please enter application key id: ").unwrap();
            stdout.flush().unwrap();

            readline(&stdin)
        }
    };

    write!(stdout, "Please enter the application key: ").unwrap();
    stdout.flush().unwrap();
    let application_key = readline(&stdin);

    let res = b2_authorize_account(&application_key_id, &application_key).await;
    writeln!(stdout, "Result: {:#?}", res).unwrap();
    stdout.flush().unwrap();

    // this is synchrounous as well ... todo: make async
    if let Some(save_file) = save {
        if let Ok(auth_data) = res {
            match std::fs::File::create(&save_file) {
                Ok(f) => {
                    serde_yaml::to_writer(&f, &auth_data).unwrap();
                }
                Err(e) => panic!(
                    "Could not open file {:#?} to save authentication data:\n {:#?}",
                    save_file, e
                ),
            }
            println!("Authentication Data saved to file {:#?}", save_file);
        } else {
            println!("Authentication failed, will not save result");
        }
    }
}
