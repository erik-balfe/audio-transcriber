use crate::state::StateManager;
use anyhow::{Context, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use log::{debug, info, trace};
use std::sync::Arc;
use tokio::select;
use tokio::time::{Duration, Instant};

pub async fn record_audio(state_manager: Arc<StateManager>) -> Result<()> {
    info!("Starting audio recording");
    let config = Arc::clone(&state_manager).get_config();
    let max_duration = Duration::from_secs_f64(config.max_recording_duration());
    debug!("Max recording duration: {:?}", max_duration);

    let pipeline = gst::parse_launch(&format!(
        "autoaudiosrc ! audioconvert ! audioresample ! audio/x-raw,rate={},channels={},format=F32LE ! appsink name=sink",
        config.sample_rate,
        config.channels
    )).context("Failed to create GStreamer pipeline")?;
    debug!("GStreamer pipeline created");

    let sink = pipeline
        .downcast_ref::<gst::Bin>()
        .unwrap()
        .by_name("sink")
        .context("Sink element not found")?
        .downcast::<gst_app::AppSink>()
        .map_err(|_| anyhow::anyhow!("Sink element is not an AppSink"))?;
    debug!("AppSink element retrieved from pipeline");

    let state_manager_clone = Arc::clone(&state_manager);
    sink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                trace!("New audio sample received");
                let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                let buffer = sample.buffer().ok_or_else(|| gst::FlowError::Error)?;
                let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;

                let new_data = bytemuck::cast_slice::<u8, f32>(&map).to_vec();

                state_manager_clone.append_audio_data(&new_data);

                if state_manager_clone.is_recording_sync() {
                    Ok(gst::FlowSuccess::Ok)
                } else {
                    trace!("Recording stopped, ending sample processing");
                    Err(gst::FlowError::Eos)
                }
            })
            .build(),
    );

    pipeline
        .set_state(gst::State::Playing)
        .context("Failed to set pipeline to Playing state")?;
    info!("GStreamer pipeline started");

    let mut stop_receiver = Arc::clone(&state_manager).start_recording();

    let start_time = Instant::now();
    let state_manager_clone = Arc::clone(&state_manager);
    tokio::spawn(async move {
        loop {
            select! {
                _ = stop_receiver.recv() => {
                    info!("Received stop signal");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    if !state_manager_clone.is_recording() {
                        info!("Recording stopped");
                        break;
                    }
                    if start_time.elapsed() >= max_duration {
                        info!("Maximum recording duration reached");
                        state_manager_clone.stop_recording();
                        break;
                    }
                    debug!("Recording in progress: {:?} elapsed", start_time.elapsed());
                }
            }
        }
    });

    // Wait for the recording to stop
    while Arc::clone(&state_manager).is_recording() {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    pipeline
        .set_state(gst::State::Null)
        .context("Failed to set pipeline to Null state")?;
    info!("Audio recording completed");

    Ok(())
}
