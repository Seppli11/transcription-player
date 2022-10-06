use std::{
    backtrace::Backtrace,
    cmp,
    collections::VecDeque,
    fmt::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    thread,
    time::Duration,
};

use rodio::{
    cpal::SampleFormat,
    decoder::DecoderError,
    source::{Delay, SineWave},
    Decoder, OutputStream, OutputStreamHandle, PlayError, Sink, Source, StreamError,
};
use rubberband_rs::{AudioBuffer, RubberBand, RubberBandOption};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Couldn't create output stream for audio player")]
    CreateOutputStreamError {
        #[from]
        source: StreamError,
    },
    #[error("Couldn't create sink for audio player")]
    CreateSinkError {
        #[from]
        source: PlayError,
    },
    #[error("Couldn't load audio file \"{path}\"")]
    LoadError { path: String, source: io::Error },
    #[error("Couldn't decode audio file \"{path}\"")]
    DecodeError { path: String, source: DecoderError },
}

pub struct AudioPlayer {
    stream_handle: OutputStreamHandle,
    sink: Sink,
}

impl AudioPlayer {
    pub fn new() -> Result<Self, AudioError> {
        let (_stream, stream_handle) =
            OutputStream::try_default().map_err(|err| AudioError::from(err))?;

        let sink = Sink::try_new(&stream_handle).map_err(|err| AudioError::from(err))?;
        Ok(AudioPlayer {
            stream_handle,
            sink,
        })
    }

    pub fn load(&mut self, path: &Path) -> Result<(), AudioError> {
        let file = File::open(path).map_err(|err| AudioError::LoadError {
            path: path.display().to_string(),
            source: err,
        })?;
        let buffered_reader = BufReader::new(file);
        let source = Decoder::new(buffered_reader).map_err(|err| AudioError::DecodeError {
            path: path.display().to_string(),
            source: err,
        })?;
        let RUBBER_BAND_OPTIONS: RubberBandOption =
            RubberBandOption::PROCESS_REAL_TIME | RubberBandOption::ENGINE_FINER;
        let source = RubberBandSource::new(source.convert_samples(), RUBBER_BAND_OPTIONS);

        self.sink.append(source);
        self.sink.sleep_until_end();
        Ok(())
    }

    pub fn play(&mut self) {
        self.sink.play();
    }
    pub fn pause(&mut self) {
        self.sink.pause();
    }
    pub fn seek(&mut self, time: Duration) {
        todo!()
    }
}

const INPUT_BUFFER_SIZE: usize = 1024 * 2;

pub struct RubberBandSource<S: Source + Iterator<Item = f32>> {
    rubberband_options: RubberBandOption,
    rubberband: RubberBand,
    source: S,
    buffer: VecDeque<f32>,
    frame_len_left: Option<usize>,
}

impl<S: Source + Iterator<Item = f32>> RubberBandSource<S> {
    pub fn new(source: S, rubberband_options: RubberBandOption) -> Self {
        let rubberband = RubberBand::new(
            source.sample_rate() as u32,
            source.channels() as u32,
            rubberband_options.clone(),
            1.5,
            1.0,
        );
        let frame_len_left = source.current_frame_len();
        RubberBandSource {
            rubberband_options,
            rubberband,
            source,
            buffer: VecDeque::new(),
            frame_len_left,
        }
    }

    fn recreate_rubberband_if_necessary(&mut self) {
        if self.source.channels() as u32 != self.rubberband.channel_count() {
            println!("recreate rubberband");
            self.rubberband = RubberBand::new(
                self.source.sample_rate(),
                self.source.channels() as u32,
                self.rubberband_options,
                1.0,
                1.0,
            );
        }
    }

    fn try_process_rubberband(&mut self) -> bool {
        if self.frame_len_left == Some(0) {
            self.recreate_rubberband_if_necessary();
            self.frame_len_left = self.source.current_frame_len();
        }
        let mut input_buffer = vec![];
        let channels = self.rubberband.channel_count() as usize;
        let input_size = cmp::min(
            INPUT_BUFFER_SIZE,
            self.frame_len_left.unwrap_or(usize::MAX) * channels,
        );
        loop {
            let value = self.source.next();
            if let Some(value) = value {
                input_buffer.push(value);
            } else {
                break;
            }
            if input_buffer.len() >= input_size {
                break;
            }
        }
        if input_buffer.is_empty() {
            return false; // no elements left in source
        }
        let audio_buffer =
            AudioBuffer::from_interleafed(self.rubberband.channel_count(), &input_buffer);
        self.rubberband.process(&audio_buffer, false);
        true
    }

    fn try_retrieve_rubberband(&mut self) -> bool {
        println!("test");
        // tries to process more items. If no items are left in source then return false
        if self.rubberband.available() == 0 {
            while self.rubberband.available() < 1024 * 40 {
                if !self.try_process_rubberband() {
                    return false;
                }
            }
        }
        let mut buffer =
            AudioBuffer::new_sized(self.channels() as u32, self.rubberband.available() as usize);
        let sample_count = self.rubberband.retrieve(&mut buffer) as usize;
        let interleaved_buffer = &buffer.to_interleaved()[..sample_count * buffer.num_channels()];
        for sample in interleaved_buffer {
            self.buffer.push_back(*sample);
        }
        true
    }
}

impl<S: Source + Iterator<Item = f32>> Source for RubberBandSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl<S: Source + Iterator<Item = f32>> Iterator for RubberBandSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // retrieve more samples if buffer is empty.
        // if no samples could be retrieved because the source is empty then return None
        if self.buffer.is_empty() && !self.try_retrieve_rubberband() {
            None
        } else {
            let sample = self.buffer.pop_front();
            sample
        }
    }
}
