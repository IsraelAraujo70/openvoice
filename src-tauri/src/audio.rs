//! Audio recording module using cpal
//! Records audio from the microphone and converts to WAV format

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, StreamConfig};
use hound::{WavSpec, WavWriter};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Audio device information for frontend display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

/// Audio recorder state
pub struct AudioRecorder {
    host: Host,
    recording: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    channels: Arc<Mutex<u16>>,
    selected_device: Arc<Mutex<Option<String>>>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        let host = cpal::default_host();
        Self {
            host,
            recording: Arc::new(AtomicBool::new(false)),
            samples: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(44100)),
            channels: Arc::new(Mutex::new(1)),
            selected_device: Arc::new(Mutex::new(None)),
        }
    }

    /// Get list of available input devices
    pub fn get_input_devices(&self) -> Vec<AudioDevice> {
        let mut devices = Vec::new();
        let default_device_name = self.host.default_input_device().and_then(|d| d.name().ok());

        if let Ok(input_devices) = self.host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    let is_default = default_device_name
                        .as_ref()
                        .map(|d| d == &name)
                        .unwrap_or(false);
                    devices.push(AudioDevice { name, is_default });
                }
            }
        }

        devices
    }

    /// Set the device to use for recording
    pub fn set_device(&self, device_name: Option<String>) {
        let mut selected = self.selected_device.lock().unwrap();
        *selected = device_name;
    }

    /// Get the selected or default input device
    fn get_device(&self) -> Option<Device> {
        let selected = self.selected_device.lock().unwrap();

        if let Some(ref name) = *selected {
            // Find device by name
            if let Ok(devices) = self.host.input_devices() {
                for device in devices {
                    if let Ok(device_name) = device.name() {
                        if &device_name == name {
                            return Some(device);
                        }
                    }
                }
            }
        }

        // Fall back to default
        self.host.default_input_device()
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.recording.load(Ordering::SeqCst)
    }

    /// Start recording audio - this blocks and runs in the same thread
    pub fn start_recording(&self) -> Result<(), String> {
        if self.is_recording() {
            return Err("Already recording".to_string());
        }

        let device = self.get_device().ok_or("No input device available")?;

        log::info!("Recording from device: {:?}", device.name());

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        log::info!("Using config: {:?}", config);

        // Store sample rate and channels
        {
            let mut sr = self.sample_rate.lock().unwrap();
            *sr = config.sample_rate().0;
        }
        {
            let mut ch = self.channels.lock().unwrap();
            *ch = config.channels();
        }

        // Clear previous samples
        {
            let mut samples = self.samples.lock().unwrap();
            samples.clear();
        }

        let samples = Arc::clone(&self.samples);
        let recording = Arc::clone(&self.recording);

        let err_fn = |err| log::error!("Audio stream error: {}", err);

        let stream_config: StreamConfig = config.clone().into();

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                let samples_clone = Arc::clone(&samples);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut samples = samples_clone.lock().unwrap();
                        samples.extend_from_slice(data);
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::I16 => {
                let samples_clone = Arc::clone(&samples);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut samples = samples_clone.lock().unwrap();
                        for &sample in data {
                            samples.push(sample as f32 / i16::MAX as f32);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                let samples_clone = Arc::clone(&samples);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let mut samples = samples_clone.lock().unwrap();
                        for &sample in data {
                            samples.push((sample as f32 / u16::MAX as f32) * 2.0 - 1.0);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            _ => return Err("Unsupported sample format".to_string()),
        }
        .map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start stream: {}", e))?;

        log::info!(">>> Audio stream started - microphone is now recording <<<");
        self.recording.store(true, Ordering::SeqCst);

        // Keep the stream alive in the current thread - poll until recording stops
        // This is necessary because cpal::Stream is !Send on some platforms
        while recording.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        drop(stream);
        log::info!("Recording stream stopped");

        Ok(())
    }

    /// Signal to stop recording (called from another thread)
    pub fn signal_stop(&self) {
        self.recording.store(false, Ordering::SeqCst);
    }

    /// Get the recorded audio as base64 WAV
    pub fn get_audio_base64(&self) -> Result<String, String> {
        let samples = {
            let samples = self.samples.lock().unwrap();
            samples.clone()
        };

        if samples.is_empty() {
            return Err("No audio recorded".to_string());
        }

        let sample_rate = *self.sample_rate.lock().unwrap();
        let channels = *self.channels.lock().unwrap();

        log::info!(
            "Recorded {} samples at {}Hz, {} channels",
            samples.len(),
            sample_rate,
            channels
        );

        // Convert to mono if stereo (for smaller file size)
        let mono_samples: Vec<f32> = if channels > 1 {
            samples
                .chunks(channels as usize)
                .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                .collect()
        } else {
            samples
        };

        // Downsample to 16kHz for smaller file size (OpenRouter accepts this)
        let target_sample_rate = 16000u32;
        let downsampled = if sample_rate > target_sample_rate {
            downsample(&mono_samples, sample_rate, target_sample_rate)
        } else {
            mono_samples
        };

        // Convert to WAV
        let wav_data = samples_to_wav(&downsampled, target_sample_rate.min(sample_rate))?;

        // Encode to base64
        let base64_data =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &wav_data);

        log::info!(
            "Generated {} bytes of WAV data, {} base64 chars",
            wav_data.len(),
            base64_data.len()
        );

        Ok(base64_data)
    }
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple downsampling by averaging
fn downsample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio) as usize;
    let mut result = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let pos = i as f64 * ratio;
        let idx = pos as usize;
        if idx < samples.len() {
            result.push(samples[idx]);
        }
    }

    result
}

/// Convert f32 samples to WAV bytes
fn samples_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut cursor, spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

        for &sample in samples {
            // Clamp and convert to i16
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * i16::MAX as f32) as i16;
            writer
                .write_sample(int_sample)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    }

    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downsample() {
        let samples: Vec<f32> = (0..44100).map(|i| (i as f32 / 44100.0).sin()).collect();
        let downsampled = downsample(&samples, 44100, 16000);
        assert!(downsampled.len() < samples.len());
        assert!((downsampled.len() as f32 - 16000.0).abs() < 100.0);
    }

    #[test]
    fn test_samples_to_wav() {
        let samples: Vec<f32> = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];
        let wav = samples_to_wav(&samples, 16000).unwrap();
        assert!(!wav.is_empty());
        // WAV header is 44 bytes
        assert!(wav.len() > 44);
    }
}
