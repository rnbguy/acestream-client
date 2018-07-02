extern crate acestream_client as acestream;
extern crate clap;
extern crate crossbeam;

use std::io::Write;
use std::panic;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use clap::{App, Arg};

fn main() {
    let matches = App::new("aceplay")
        .version("0.1.0")
        .about("Launch acestream with player")
        .author("Ranadeep Biswas")
        .arg(
            Arg::with_name("url")
                .value_name("URL")
                .help("Acestream link")
                .required(true),
        )
        .get_matches();

    let id: String = matches
        .values_of("url")
        .unwrap()
        .collect::<String>()
        .split("//")
        .last()
        .unwrap()
        .to_owned();

    let mut engine_process = Command::new("acestreamengine")
        .args(&["--client-console"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    panic::catch_unwind(|| {
        let mut engine: acestream::Engine = Default::default();
        while !engine.is_up() {
            sleep(Duration::from_secs(1));
        }

        engine.add_stream(&id);

        let stat_poll = AtomicBool::new(true);
        let player_poll = AtomicBool::new(false);

        crossbeam::scope(|scope| {
            let stat_t = scope.spawn(|| {
                let indicator = &['⠤', '⠒', '⠉', '⠒'];
                let mut counter = 0;
                while stat_poll.load(Ordering::Relaxed) {
                    let stat = engine.get_stream_stat(&id);
                    if let Some(ref status) = stat.status {
                        if status == "dl" {
                            player_poll.store(true, Ordering::Relaxed);
                        }
                    }
                    let live = if engine.is_stream_live(&id) {
                        "LIVE"
                    } else {
                        "VOD"
                    };
                    let status = stat
                        .status
                        .as_ref()
                        .map(|x| match x.as_str() {
                            "prebuf" => "BUFFER".to_owned(),
                            "dl" => "STREAM".to_owned(),
                            _ => x.to_owned(),
                        })
                        .unwrap_or("N/A".to_owned());
                    print!(
                        "\r{} [{}] : {} : down: {}kB/s up: {}kB/s peers: {}             ",
                        live,
                        indicator[counter],
                        status,
                        stat.speed_down.as_ref().unwrap_or(&0),
                        stat.speed_up.as_ref().unwrap_or(&0),
                        stat.peers.as_ref().unwrap_or(&0),
                    );
                    std::io::stdout().flush().unwrap();
                    sleep(Duration::from_secs(1));
                    counter = (counter + 1) % indicator.len();
                }
                println!("\nQuitting..");
            });

            let play_t = scope.spawn(|| {
                if !player_poll.load(Ordering::Relaxed) {
                    sleep(Duration::from_secs(1));
                }
                let mut player = Command::new("mpv")
                    .args(&[&engine.get_stream_link(&id)])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
                player.wait().expect("error when waiting for the player");
                engine.stop_stream(&id);
                stat_poll.store(false, Ordering::Relaxed);
            });

            play_t.join();
            stat_t.join();
        });
    }).is_ok();
    let mut child_kill = Command::new("pkill")
        .args(&["-2", "-P", &engine_process.id().to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child_kill
        .wait()
        .expect("error when killing child processes");
    engine_process
        .wait()
        .expect("error when waiting engine to finish");
}
