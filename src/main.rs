use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::process::Command;

#[tokio::main]
async fn main() -> notify::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <path-to-monitor> <command-to-execute>", args[0]);
        eprintln!("The <command-to-execute> will be run with the new created file path as its argument.");
        std::process::exit(1);
    }

    let path = &args[1];
    let command = &args[2];
    println!("watching {}", path);

    if let Err(e) = async_watch(path.clone(), command.clone()).await {
        println!("error: {:?}", e);
    }

    Ok(())
}

async fn async_watch(path: String, command: String) -> notify::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let mut watcher = RecommendedWatcher::new(move |res| {
        tx.blocking_send(res).unwrap();
    }, Config::default())?;

    watcher.watch(Path::new(&path), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => match event.kind {
                EventKind::Access(notify::event::AccessKind::Close(
                    notify::event::AccessMode::Write,
                )) => {
                    for path in event.paths {
                        println!("path: {:?}", path);
                        Command::new(&command)
                            .arg(path.to_str().unwrap())
                            .spawn()
                            .expect("Failed to execute command")
                            .wait_with_output()
                            .await
                            .expect("Command execution failed");
                    }
                }
                _ => (),
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
