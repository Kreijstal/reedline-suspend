mod suspendable_editor;
use suspendable_editor::{SuspendableReedline, ReadResult, SuspendableError};
use reedline::DefaultPrompt;

fn main() {
    let mut editor = SuspendableReedline::create();
    let prompt = DefaultPrompt::default();

    loop {
        let sig = editor.read_line(&prompt);

        match sig {
            Ok(ReadResult::Success(buffer)) => {
                println!("We processed: {}", buffer);
                if buffer.trim() == "exit" {
                    println!("Exiting...");
                    break;
                }
            }
            Ok(ReadResult::Aborted) => {
                println!("\nAborted!");
                break;
            }
            Ok(ReadResult::Suspended) => {
                continue;
            }
            Err(SuspendableError::Reedline(e)) => {
                eprintln!("Reedline error: {}", e);
                break;
            }
            Err(SuspendableError::Suspend(e)) => {
                eprintln!("Suspend error: {}", e);
                break;
            }
        }
    }
}