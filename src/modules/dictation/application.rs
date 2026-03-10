use crate::modules::dictation::domain::{
    CapturedAudio, DictationConfig, DictationOutput, PreparedAudio, TARGET_SAMPLE_RATE,
};
use crate::modules::dictation::infrastructure;
use base64::Engine;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::io::Cursor;

pub fn transcribe_capture(
    config: DictationConfig,
    capture: CapturedAudio,
) -> Result<DictationOutput, String> {
    let prepared = prepare_audio(capture)?;
    let transcript = infrastructure::transcribe(&config, &prepared.wav_base64)?;
    let transcript = transcript.trim().to_owned();

    if transcript.is_empty() {
        return Err(String::from(
            "A API respondeu sem texto. Tente falar de forma mais clara.",
        ));
    }

    Ok(DictationOutput {
        transcript,
        duration_seconds: prepared.duration_seconds,
    })
}

fn prepare_audio(capture: CapturedAudio) -> Result<PreparedAudio, String> {
    if capture.samples.is_empty() {
        return Err(String::from("Nenhum audio foi capturado."));
    }

    let mono = downmix_to_mono(&capture.samples, capture.channels)?;
    let normalized = resample_linear(&mono, capture.sample_rate, TARGET_SAMPLE_RATE);
    let wav = samples_to_wav(&normalized, TARGET_SAMPLE_RATE)?;

    Ok(PreparedAudio {
        wav_base64: base64::engine::general_purpose::STANDARD.encode(wav),
        duration_seconds: capture.duration_seconds(),
    })
}

fn downmix_to_mono(samples: &[f32], channels: u16) -> Result<Vec<f32>, String> {
    match channels {
        0 => Err(String::from("O dispositivo retornou zero canais.")),
        1 => Ok(samples.to_vec()),
        channels => {
            let width = channels as usize;

            Ok(samples
                .chunks(width)
                .map(|frame| frame.iter().copied().sum::<f32>() / frame.len() as f32)
                .collect())
        }
    }
}

fn resample_linear(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if samples.is_empty() || source_rate == 0 || source_rate == target_rate {
        return samples.to_vec();
    }

    let ratio = source_rate as f64 / target_rate as f64;
    let target_len = ((samples.len() as f64) / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(target_len);

    for index in 0..target_len {
        let source_position = index as f64 * ratio;
        let left_index = source_position.floor() as usize;
        let right_index = (left_index + 1).min(samples.len().saturating_sub(1));
        let fraction = (source_position - left_index as f64) as f32;
        let left = samples[left_index];
        let right = samples[right_index];

        output.push(left + ((right - left) * fraction));
    }

    output
}

fn samples_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut cursor = Cursor::new(Vec::new());

    {
        let mut writer = WavWriter::new(&mut cursor, spec)
            .map_err(|error| format!("Falha ao criar WAV temporario: {error}"))?;

        for sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let encoded = (clamped * i16::MAX as f32) as i16;

            writer
                .write_sample(encoded)
                .map_err(|error| format!("Falha ao escrever WAV temporario: {error}"))?;
        }

        writer
            .finalize()
            .map_err(|error| format!("Falha ao finalizar WAV temporario: {error}"))?;
    }

    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::{downmix_to_mono, resample_linear, samples_to_wav};

    #[test]
    fn downmixes_stereo_frames() {
        let mono = downmix_to_mono(&[0.2, 0.4, 0.6, 0.8], 2).expect("mono");

        assert!((mono[0] - 0.3).abs() < f32::EPSILON);
        assert!((mono[1] - 0.7).abs() < 0.0001);
    }

    #[test]
    fn resamples_audio_with_linear_interpolation() {
        let resampled = resample_linear(&[0.0, 0.5, 1.0, 0.5], 8_000, 16_000);

        assert_eq!(resampled.first().copied(), Some(0.0));
        assert_eq!(resampled.len(), 8);
        assert!(resampled[3] > resampled[2]);
    }

    #[test]
    fn encodes_pcm_as_wav() {
        let wav = samples_to_wav(&[0.0, 0.5, -0.5, 0.2], 16_000).expect("wav");

        assert!(wav.len() > 44);
        assert_eq!(&wav[0..4], b"RIFF");
    }
}
