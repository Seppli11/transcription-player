mod bindings;

use core::num;
use std::{borrow::Borrow, thread};

use bitflags::bitflags;

use bindings::{
    rubberband_available, rubberband_calculate_stretch, rubberband_delete,
    rubberband_get_channel_count, rubberband_get_engine_version, rubberband_get_formant_scale,
    rubberband_get_latency, rubberband_get_pitch_scale, rubberband_get_preferred_start_pad,
    rubberband_get_samples_required, rubberband_get_start_delay, rubberband_get_time_ratio,
    rubberband_new, rubberband_process, rubberband_reset, rubberband_retrieve,
    rubberband_set_debug_level, rubberband_set_default_debug_level, rubberband_set_detector_option,
    rubberband_set_expected_input_duration, rubberband_set_formant_option,
    rubberband_set_formant_scale, rubberband_set_key_frame_map, rubberband_set_max_process_size,
    rubberband_set_phase_option, rubberband_set_pitch_scale, rubberband_set_time_ratio,
    rubberband_set_transients_option, rubberband_study, RubberBandState,
};

bitflags! {
    pub struct RubberBandOption: i32 {

        const PROCESS_OFFLINE       = 0x00000000;
        const PROCESS_REAL_TIME      = 0x00000001;

        const STRETCH_ELASTIC       = 0x00000000; // obsolet
        const STRETCH_PRECISE       = 0x00000010; // obsolet

        const TRANSIENTS_CRISP      = 0x00000000;
        const TRANSIENTS_MIXED      = 0x00000100;
        const TRANSIENTS_SMOOTH     = 0x00000200;

        const DETECTOR_COMPOUND     = 0x00000000;
        const DETECTOR_PERCUSSIVE   = 0x00000400;
        const DETECTOR_SOFT         = 0x00000800;

        const PHASE_LAMINAR         = 0x00000000;
        const PHASE_INDEPENDENT     = 0x00002000;

        const THREADING_AUTO        = 0x00000000;
        const THREADING_NEVER       = 0x00010000;
        const THREADING_ALWAYS      = 0x00020000;

        const WINDOW_STANDARD       = 0x00000000;
        const WINDOW_SHORT          = 0x00100000;
        const WINDOW_LONG           = 0x00200000;

        const SMOOTHING_OFF         = 0x00000000;
        const SMOOTHING_ON          = 0x00800000;

        const FORMANT_SHIFTED       = 0x00000000;
        const FORMANT_PRESERVED     = 0x01000000;

        const PITCH_HIGH_SPEED       = 0x00000000;
        const PITCH_HIGH_QUALITY     = 0x02000000;
        const PITCH_HIGH_CONSISTENCY = 0x04000000;

        const CHANNELS_APART        = 0x00000000;
        const CHANNELS_TOGETHER     = 0x10000000;

        const ENGINE_FASTER         = 0x00000000;
        const ENGINE_FINER          = 0x2000000;
    }
}

impl Default for RubberBandOption {
    fn default() -> Self {
        return RubberBandOption::empty();
    }
}

impl RubberBandOption {
    pub fn percussive_options() -> Self {
        RubberBandOption::WINDOW_SHORT | RubberBandOption::PHASE_INDEPENDENT
    }
}

pub struct AudioBuffer {
    channels: Vec<Vec<f32>>,
}

impl AudioBuffer {
    pub fn new_sized(num_channels: u32, size: usize) -> Self {
        assert!(num_channels > 0, "num_channels must be greater than zero");
        let channels = (0..num_channels).map(|_| vec![0.0; size]).collect();
        AudioBuffer { channels }
    }
    pub fn new(num_channels: u32) -> Self {
        Self::new_sized(num_channels, 0)
    }

    pub fn from_interleafed(num_channels: u32, sample_block: &[f32]) -> Self {
        let mut buffer = Self::new(num_channels);
        buffer.push(sample_block);
        buffer
    }

    pub fn num_channels(&self) -> usize {
        self.channels.len()
    }

    pub fn num_samples(&self) -> usize {
        self.channels[0].len()
    }

    pub fn push(&mut self, sample: &[f32]) {
        if sample.len() % self.num_channels() != 0 {
            panic!(
                "The number of samples ({}) needs to be a multiple of the number of channels ({})",
                sample.len(),
                self.num_channels()
            );
        }

        for (i, sample_value) in sample.iter().enumerate() {
            let channel_nr = i % self.num_channels();
            self.channels[channel_nr].push(*sample_value)
        }
    }

    pub fn clear(&mut self) {
        for channel in &mut self.channels {
            channel.clear();
        }
    }

    pub fn replace(&mut self, sample: &[f32]) {
        self.clear();
        self.push(sample);
    }

    pub fn channel(&self, i: usize) -> &Vec<f32> {
        &self.channels[i]
    }

    pub fn to_interleaved(&self) -> Vec<f32> {
        let output_size = self.num_channels() * self.num_samples();
        let mut output = Vec::with_capacity(output_size);
        for sample_i in 0..self.num_samples() {
            for channel_i in 0..self.num_channels() {
                output.push(self.channel(channel_i)[sample_i]);
            }
        }
        output
    }

    fn as_ptr_list(&self) -> Vec<*const f32> {
        self.channels.iter().map(|arr| arr.as_ptr()).collect()
    }

    fn as_mut_ptr_list(&mut self) -> Vec<*mut f32> {
        self.channels
            .iter_mut()
            .map(|arr| arr.as_mut_ptr())
            .collect()
    }
}

pub struct RubberBand {
    state: RubberBandState,
}

pub struct KeyFrame {
    from: u32,
    to: u32,
}

impl RubberBand {
    pub fn new(
        sample_rate: u32,
        channels: u32,
        options: RubberBandOption,
        initial_time_ratio: f64,
        initial_pitch_scale: f64,
    ) -> RubberBand {
        unsafe {
            let state = rubberband_new(
                sample_rate,
                channels,
                options.bits(),
                initial_time_ratio,
                initial_pitch_scale,
            );
            RubberBand { state: state }
        }
    }

    pub fn reset(&mut self) {
        unsafe {
            rubberband_reset(self.state);
        }
    }

    pub fn engine_version(&self) -> i32 {
        unsafe { rubberband_get_engine_version(self.state) }
    }

    pub fn set_time_ratio(&mut self, ratio: f64) {
        unsafe {
            rubberband_set_time_ratio(self.state, ratio);
        }
    }

    pub fn set_pitch_scale(&mut self, scale: f64) {
        unsafe {
            rubberband_set_pitch_scale(self.state, scale);
        }
    }

    pub fn set_formant_scale(&mut self, scale: f64) {
        unsafe {
            rubberband_set_formant_scale(self.state, scale);
        }
    }

    pub fn time_ratio(&self) -> f64 {
        unsafe { rubberband_get_time_ratio(self.state) }
    }

    pub fn pitch_scale(&self) -> f64 {
        unsafe { rubberband_get_pitch_scale(self.state) }
    }

    pub fn formant_scale(&self) -> f64 {
        unsafe { rubberband_get_formant_scale(self.state) }
    }

    pub fn preferred_start_pad(&self) -> u32 {
        unsafe { rubberband_get_preferred_start_pad(self.state) }
    }

    pub fn start_delay(&self) -> u32 {
        unsafe { rubberband_get_start_delay(self.state) }
    }

    pub fn latency(&self) -> u32 {
        unsafe { rubberband_get_latency(self.state) }
    }

    pub fn channel_count(&self) -> u32 {
        unsafe { rubberband_get_channel_count(self.state) }
    }
    pub fn set_transients_options(&mut self, options: RubberBandOption) {
        unsafe {
            rubberband_set_transients_option(self.state, options.bits());
        }
    }

    pub fn set_detector_option(&mut self, options: RubberBandOption) {
        unsafe {
            rubberband_set_detector_option(self.state, options.bits());
        }
    }
    pub fn set_phase_option(&mut self, options: RubberBandOption) {
        unsafe {
            rubberband_set_phase_option(self.state, options.bits());
        }
    }
    pub fn set_formant_options(&mut self, options: RubberBandOption) {
        unsafe {
            rubberband_set_formant_option(self.state, options.bits());
        }
    }

    pub fn set_expected_input_duration(&mut self, samples: u32) {
        unsafe {
            rubberband_set_expected_input_duration(self.state, samples);
        }
    }

    pub fn set_max_process_size(&mut self, samples: u32) {
        unsafe {
            rubberband_set_max_process_size(self.state, samples);
        }
    }

    pub fn samples_required(&self) -> u32 {
        unsafe { rubberband_get_samples_required(self.state) }
    }
    pub fn set_key_frame_map(&mut self, keyframes: &[KeyFrame]) {
        let mut from: Vec<u32> = keyframes.iter().map(|keyframe| keyframe.from).collect();
        let mut to: Vec<u32> = keyframes.iter().map(|keyframe| keyframe.to).collect();
        unsafe {
            rubberband_set_key_frame_map(
                self.state,
                keyframes.len() as u32,
                from.as_mut_ptr(),
                to.as_mut_ptr(),
            );
        }
    }

    pub fn study(&mut self, input: &AudioBuffer, final_flag: bool) {
        unsafe {
            let sample_num = input.num_samples() as u32;
            let pointer_list = input.as_ptr_list();
            rubberband_study(
                self.state,
                pointer_list.as_ptr(),
                sample_num,
                final_flag.into(),
            );
        }
    }

    pub fn process(&mut self, input: &AudioBuffer, final_flag: bool) {
        unsafe {
            let sample_num = input.num_samples() as u32;
            let pointer_list = input.as_ptr_list();
            rubberband_process(
                self.state,
                pointer_list.as_ptr(),
                sample_num,
                final_flag.into(),
            );
        }
    }

    pub fn available(&self) -> i32 {
        unsafe { rubberband_available(self.state) }
    }

    pub fn retrieve(&mut self, output: &mut AudioBuffer) -> u32 {
        let sample_num = output.num_samples() as u32;
        let pointer_list = output.as_mut_ptr_list();
        unsafe { rubberband_retrieve(self.state, pointer_list.as_ptr(), sample_num) }
    }

    pub fn calculate_stretch(&mut self) {
        unsafe {
            rubberband_calculate_stretch(self.state);
        }
    }

    pub fn set_debug_level(&mut self, level: i32) {
        unsafe {
            rubberband_set_debug_level(self.state, level);
        }
    }

    pub fn set_default_debug_level(level: i32) {
        unsafe {
            rubberband_set_default_debug_level(level);
        }
    }
}

unsafe impl Send for RubberBand {}

impl Drop for RubberBand {
    fn drop(&mut self) {
        unsafe {
            rubberband_delete(self.state);
        }
    }
}
