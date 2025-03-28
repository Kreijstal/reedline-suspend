use reedline::{
    Reedline, Signal, Prompt, ReedlineEvent,
    ReedlineError
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::io;
use std::io::Write;
use thiserror::Error;

#[cfg(unix)]
use nix::sys::signal::{raise, Signal as NixSignal};

const INTERNAL_SUSPEND_MARKER: &str = ":::SUSPENDABLE_REEDLINE_SUSPEND:::";

#[derive(Error, Debug)]
pub enum SuspendableError {
    #[error("Reedline error: {0}")]
    Reedline(#[from] ReedlineError),

    #[error("Failed during suspend operation: {0}")]
    Suspend(#[from] io::Error),
}

pub type SuspendableResult<T> = Result<T, SuspendableError>;

pub struct SuspendableReedline {
    editor: Reedline,
    suspend_marker: String,
}

#[derive(Debug)]
pub enum ReadResult {
    Success(String),
    Aborted,
    Suspended,
}

impl SuspendableReedline {
    pub fn create() -> Self {
        let mut keybindings = reedline::default_emacs_keybindings();
        let suspend_marker = INTERNAL_SUSPEND_MARKER.to_string();

        let ctrl_z_modifier = KeyModifiers::CONTROL;
        let ctrl_z_keycode_lower = KeyCode::Char('z');
        let ctrl_z_keycode_upper = KeyCode::Char('Z');

        keybindings.remove_binding(ctrl_z_modifier, ctrl_z_keycode_lower);
        keybindings.remove_binding(ctrl_z_modifier, ctrl_z_keycode_upper);

        let suspend_event = ReedlineEvent::ExecuteHostCommand(suspend_marker.clone());
        keybindings.add_binding(
            ctrl_z_modifier,
            ctrl_z_keycode_lower,
            suspend_event.clone(),
        );
        keybindings.add_binding(
            ctrl_z_modifier,
            ctrl_z_keycode_upper,
            suspend_event,
        );

        let editor = Reedline::create().with_edit_mode(Box::new(reedline::Emacs::new(keybindings)));

        //println!("INFO: SuspendableReedline created. Ctrl+Z should suspend on Unix.");

        SuspendableReedline { editor, suspend_marker }
    }

    pub fn read_line(&mut self, prompt: &dyn Prompt) -> SuspendableResult<ReadResult> {
        match self.editor.read_line(prompt) {
            Ok(Signal::Success(buffer)) => {
                if buffer == self.suspend_marker {
                    #[cfg(unix)]
                    {
                        //println!("\nSuspending process (SIGTSTP)...");
                        let _ = std::io::stdout().flush();

                        if let Err(e) = raise(NixSignal::SIGTSTP) {
                            let io_error = io::Error::new(
                                io::ErrorKind::Other,
                                format!("Failed to send SIGTSTP: {}", e)
                            );
                            return Err(SuspendableError::Suspend(io_error));
                        }
                        println!("Resumed.");
                    }
                    #[cfg(not(unix))]
                    {
                        println!("\nSuspend (Ctrl+Z) pressed, but not supported on this platform. Ignoring.");
                    }
                    Ok(ReadResult::Suspended)
                } else {
                    Ok(ReadResult::Success(buffer))
                }
            }
            Ok(Signal::CtrlC) | Ok(Signal::CtrlD) => {
                Ok(ReadResult::Aborted)
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }
}