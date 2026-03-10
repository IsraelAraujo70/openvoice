use crate::modules::audio::domain::{AudioSourceKind, CapturedAudio, CapturedTrack};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SupportedStreamConfig};
use std::sync::{Arc, Mutex};

type SharedSamples = Arc<Mutex<Vec<f32>>>;
type SharedError = Arc<Mutex<Option<String>>>;

pub struct Recorder {
    config: SupportedStreamConfig,
    stream: cpal::Stream,
    samples: SharedSamples,
    last_error: SharedError,
    device_name: String,
}

impl Recorder {
    pub fn finish(self) -> Result<CapturedTrack, String> {
        let Recorder {
            config,
            stream,
            samples,
            last_error,
            device_name,
        } = self;

        let _ = stream.pause();
        drop(stream);

        if let Some(error) = last_error
            .lock()
            .map_err(|_| String::from("Nao foi possivel ler o estado do stream de audio."))?
            .clone()
        {
            return Err(error);
        }

        let samples = samples
            .lock()
            .map_err(|_| String::from("Nao foi possivel finalizar a captura de audio."))?
            .clone();

        Ok(CapturedTrack {
            source: AudioSourceKind::Microphone,
            device_name,
            audio: CapturedAudio {
                samples,
                sample_rate: config.sample_rate(),
                channels: config.channels(),
            },
        })
    }
}

pub fn start_default_recording() -> Result<Recorder, String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| String::from("Nenhum microfone padrao foi encontrado."))?;
    let device_name = device
        .description()
        .map(|description| description.to_string())
        .unwrap_or_else(|_| String::from("microfone padrao"));
    let config = device
        .default_input_config()
        .map_err(|error| format!("Falha ao ler a configuracao do microfone: {error}"))?;

    let samples = Arc::new(Mutex::new(Vec::new()));
    let last_error = Arc::new(Mutex::new(None));
    let stream = build_stream(
        &device,
        &config,
        Arc::clone(&samples),
        Arc::clone(&last_error),
    )?;

    stream
        .play()
        .map_err(|error| format!("Falha ao iniciar a captura de audio: {error}"))?;

    Ok(Recorder {
        config,
        stream,
        samples,
        last_error,
        device_name,
    })
}

fn build_stream(
    device: &cpal::Device,
    config: &SupportedStreamConfig,
    samples: SharedSamples,
    last_error: SharedError,
) -> Result<cpal::Stream, String> {
    let err_fn = move |error| {
        if let Ok(mut slot) = last_error.lock() {
            *slot = Some(format!("O stream de audio falhou: {error}"));
        }
    };

    match config.sample_format() {
        cpal::SampleFormat::I8 => device
            .build_input_stream(
                &config.clone().into(),
                move |input: &[i8], _| push_samples(input, &samples),
                err_fn,
                None,
            )
            .map_err(stream_error),
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &config.clone().into(),
                move |input: &[i16], _| push_samples(input, &samples),
                err_fn,
                None,
            )
            .map_err(stream_error),
        cpal::SampleFormat::I32 => device
            .build_input_stream(
                &config.clone().into(),
                move |input: &[i32], _| push_samples(input, &samples),
                err_fn,
                None,
            )
            .map_err(stream_error),
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &config.clone().into(),
                move |input: &[f32], _| push_samples(input, &samples),
                err_fn,
                None,
            )
            .map_err(stream_error),
        other => Err(format!("Formato de audio nao suportado: {other:?}")),
    }
}

fn push_samples<T>(input: &[T], samples: &SharedSamples)
where
    T: Sample,
    f32: FromSample<T>,
{
    if let Ok(mut buffer) = samples.lock() {
        buffer.extend(input.iter().copied().map(f32::from_sample));
    }
}

fn stream_error(error: cpal::BuildStreamError) -> String {
    format!("Falha ao preparar o stream do microfone: {error}")
}
