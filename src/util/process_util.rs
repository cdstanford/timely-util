/*
    Process management
*/

use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{fork, ForkResult};
use std::process::exit;

// Run a function as a separate process.
// If it crashes or is terminated, finish gracefully.
// This function may panic if the system calls fail or the process
// has an unexpected result (e.g. stopped, continued, nonzero exit code).
pub fn run_as_process<Out, F: FnOnce() -> Out>(func: F) {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            println!("[parent] running in subprocess PID: {}", child);
            match waitpid(child, None) {
                Ok(WaitStatus::Exited(pid, code)) => {
                    debug_assert!(child == pid);
                    if code != 0 {
                        println!("[parent] non-zero exit code! {}", code);
                    }
                }
                Ok(WaitStatus::Signaled(pid, signal, code)) => {
                    debug_assert!(child == pid);
                    println!(
                        "[parent] process killed! signal {}, exit code {}",
                        signal, code
                    );
                }
                Ok(status) => panic!(
                    "[parent] Error: unexpected child process status! {:?}",
                    status
                ),
                Err(err) => panic!("[parent] Error: waitpid failed! {}", err),
            }
        }
        Ok(ForkResult::Child) => {
            // println!("[child] starting");
            func();
            // println!("[child] exiting");
            exit(0)
        }
        Err(err) => panic!("[parent] Error: fork failed! {}", err),
    }
}
