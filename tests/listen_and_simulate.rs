extern crate rdev;
extern crate tokio;
use rdev::{simulate, EventType, Key};

use std::process::Stdio;
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

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

    let mut child = cmd.spawn().expect("failed to spawn command");

    let stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");

    let mut reader = BufReader::new(stdout).lines();

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async {
        let status = child.await.expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    // Wait for listen to start
    thread::sleep(Duration::from_secs(1));

    let event_type = EventType::KeyPress(Key::KeyS);
    let result = simulate(&event_type);
    assert!(result.is_ok());
    let result = simulate(&EventType::KeyRelease(Key::KeyS));
    assert!(result.is_ok());

    let string = format!("{:?}", event_type);
    let fut = timeout(Duration::from_secs(1), reader.next_line());
    match fut.await{
        Ok(Ok(Some(line))) => {
            println!("Received line {:?}", line);
            assert!(line.contains(&string));
        },
        Ok(Ok(None)) => {assert!(false, "Empty stdout");}
        Ok(Err(_)) => {assert!(false, "Error reading stdout");}
        Err(_) => {assert!(false, "Timeout expired !");}
    }
}
