mod audio;

use std::{
    cmp,
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    thread,
    time::Duration,
};

use audio::AudioPlayer;
use rodio::{
    cpal::SampleFormat,
    source::{Delay, SineWave},
    Decoder, OutputStream, Source,
};
use rubberband_rs::{AudioBuffer, RubberBand, RubberBandOption};

fn main() {
    let mut player = AudioPlayer::new().unwrap();
    player.load(Path::new("./test.mp3")).unwrap();
    player.play();

    std::thread::sleep(Duration::from_secs(5));
}
