use crate::state::{AppStateEnum, StateManager};
use anyhow::{Context, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_audio as gst_audio;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub async fn play_audio(state_manager: Arc<StateManager>) -> Result<()> {
    let audio_data = state_manager.get_audio_data();

    let pipeline_desc = "appsrc name=src ! audioconvert ! audioresample ! autoaudiosink";
    let pipeline =
        gst::parse_launch(pipeline_desc).context("Failed to create GStreamer pipeline")?;
    let pipeline = pipeline.downcast::<gst::Pipeline>().unwrap();

    let src = pipeline
        .by_name("src")
        .context("Source element not found")?
        .downcast::<gst_app::AppSrc>()
        .map_err(|_| anyhow::anyhow!("Source element is not an AppSrc"))?;

    src.set_caps(Some(
        &gst_audio::AudioInfo::builder(gst_audio::AudioFormat::F32le, 44100, 1)
            .build()
            .context("Failed to build AudioInfo")?
            .to_caps()
            .context("Failed to convert AudioInfo to caps")?,
    ));
    src.set_format(gst::Format::Time);

    pipeline
        .set_state(gst::State::Playing)
        .context("Failed to set pipeline to Playing state")?;

    let duration = gst::ClockTime::from_nseconds((audio_data.len() as u64 * 1_000_000_000) / 44100);

    let byte_data: Vec<u8> = audio_data
        .into_iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    let mut buffer = gst::Buffer::from_mut_slice(byte_data);

    {
        let buffer_ref = buffer.get_mut().unwrap();
        buffer_ref.set_duration(duration);
    }

    src.push_buffer(buffer)
        .context("Failed to push buffer to source")?;
    src.end_of_stream()
        .context("Failed to signal end of stream")?;

    let bus = pipeline.bus().context("Failed to get pipeline bus")?;

    loop {
        tokio::select! {
            _ = sleep(Duration::from_millis(100)) => {
                if let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(0)) {
                    use gst::MessageView;
                    match msg.view() {
                        MessageView::Eos(..) => break,
                        MessageView::Error(err) => {
                            pipeline.set_state(gst::State::Null)?;
                            return Err(anyhow::anyhow!("Error: {:?}", err));
                        }
                        _ => (),
                    }
                }

                if state_manager.get_app_state() != AppStateEnum::Playing {
                    break;
                }
            }
        }
    }

    pipeline
        .set_state(gst::State::Null)
        .context("Failed to set pipeline to Null state")?;
    Ok(())
}
