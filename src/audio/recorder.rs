use crate::state::{AppStateEnum, StateManager};
use anyhow::{anyhow, Context as _};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use log::{debug, info, trace};
use std::sync::Arc;
use tokio::select;
use tokio::time::{Duration, Instant};

pub async fn record_audio(state_manager: Arc<StateManager>) -> anyhow::Result<()> {
    info!("Starting audio recording");
    let config = Arc::clone(&state_manager).get_config();
    let max_duration = Duration::from_secs_f64(config.max_recording_duration());
    debug!("Max recording duration: {:?}", max_duration);

    let pipeline_desc = format!(
        "autoaudiosrc ! audioconvert ! audioresample ! audio/x-raw,rate={},channels={},format=F32LE ! appsink name=sink",
        config.sample_rate,
        config.channels
    );
    let pipeline = gst::parse_launch(&pipeline_desc)
        .map_err(|e| anyhow!("Failed to create GStreamer pipeline: {}", e))?;
    let pipeline = pipeline.downcast::<gst::Pipeline>().unwrap();

    debug!("GStreamer pipeline created");

    let sink = pipeline
        .by_name("sink")
        .ok_or_else(|| anyhow!("Sink element not found"))?
        .downcast::<gst_app::AppSink>()
        .map_err(|_| anyhow!("Sink element is not an AppSink"))?;
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

                if state_manager_clone.get_app_state() == AppStateEnum::Recording {
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
        .map_err(|e| anyhow::anyhow!("Failed to set pipeline to Playing state: {:?}", e))?;
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
                    if state_manager_clone.get_app_state() != AppStateEnum::Recording {
                        info!("Recording stopped");
                        break;
                    }
                    if start_time.elapsed() >= max_duration {
                        info!("Maximum recording duration reached");
                        state_manager_clone.set_app_state(AppStateEnum::Recorded);
                        break;
                    }
                    debug!("Recording in progress: {:?} elapsed", start_time.elapsed());
                }
            }
        }
    });

    // Wait for the recording to stop
    while state_manager.get_app_state() == AppStateEnum::Recording {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    pipeline
        .set_state(gst::State::Null)
        .map_err(|e| anyhow::anyhow!("Failed to set pipeline to Null state: {:?}", e))?;
    info!("Audio recording completed");

    Ok(())
}
