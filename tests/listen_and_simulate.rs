extern crate rdev;
extern crate tokio;
use rdev::{simulate, EventType};

use std::process::Stdio;
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[tokio::test]
async fn test_listen_and_simulate() {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--example").arg("listen");
    cmd.kill_on_drop(true);

    // Specify that we want the command's standard output piped back to us.
    // By default, standard input/output/error will be inherited from the
    // current process (for example, this means that standard input will
    // come from the keyboard and standard output/error will go directly to
    // the terminal if this process is invoked from the command line).
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn()
        .expect("failed to spawn command");

    let stdout = child.stdout.take()
        .expect("child did not have a handle to stdout");

    let mut reader = BufReader::new(stdout).lines();

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async {
        let status = child.await
            .expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    thread::sleep(Duration::from_secs(1));

    let event_type = EventType::KeyPress{code: 39};
    let result = simulate(&event_type);
    assert!(result.is_ok());
    let result = simulate(&EventType::KeyRelease{code: 39});
    assert!(result.is_ok());

    if let Ok(Some(line)) = reader.next_line().await {
        println!("Received line {:?}", line);
        assert!(line.contains("KeyPress { code: 39 }"));
    }
}
