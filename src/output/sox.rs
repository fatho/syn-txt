//! Easy interface for getting sound to play using a sox subprocess.
use std::io;
use std::process::{Command, Stdio};

pub fn with_sox_player<R, F: FnOnce(&mut dyn io::Write) -> io::Result<R>>(
    sample_rate: i32,
    callback: F,
) -> io::Result<R> {
    let mut player = Command::new("play")
        .arg("--channels")
        .arg("2")
        .arg("--rate")
        .arg(format!("{}", sample_rate))
        .arg("--type")
        .arg("f64")
        .arg("/dev/stdin")
        .arg("stats")
        .arg("spectrogram")
        .arg("-y")
        .arg("513")
        .stdin(Stdio::piped())
        .spawn()?;

    let mut audio_stream = player.stdin.take().expect("Used stdin(Stdio::piped())");

    let result = callback(&mut audio_stream);

    drop(audio_stream);
    player.wait()?;

    result
}

pub fn with_sox_wav<R, F: FnOnce(&mut dyn io::Write) -> io::Result<R>>(
    sample_rate: i32,
    callback: F,
) -> io::Result<R> {
    let mut player = Command::new("sox")
        .arg("--channels")
        .arg("2")
        .arg("--rate")
        .arg(format!("{}", sample_rate))
        .arg("--type")
        .arg("f64")
        .arg("/dev/stdin")
        .arg("--type")
        .arg("wavpcm")
        .arg("out.wav")
        .stdin(Stdio::piped())
        .spawn()?;

    let mut audio_stream = player.stdin.take().expect("Used stdin(Stdio::piped())");

    let result = callback(&mut audio_stream);

    drop(audio_stream);
    player.wait()?;

    result
}
