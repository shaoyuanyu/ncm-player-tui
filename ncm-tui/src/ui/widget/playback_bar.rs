use ncm_play::MediaState;
use ratatui::widgets::Gauge;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn build_playback_bar<'a>(
    playback_bar: Gauge<'a>,
    media_state: &Arc<Mutex<MediaState>>,
) -> Gauge<'a> {
    let guard = media_state.lock().await;
    playback_bar
        .label(format!(
            "{}:{}/{}:{}",
            // Minutes into playback
            guard.current_track_progress.map_or_else(
                || "--".to_owned(),
                |duration| format!("{:02}", duration.as_secs() / 60)
            ),
            // Seconds into playback
            guard.current_track_progress.map_or_else(
                || "--".to_owned(),
                |duration| format!("{:02}", duration.as_secs() % 60)
            ),
            // Track length minutes
            guard.current_track.as_ref().map_or_else(
                || "--".to_owned(),
                |track| format!("{:02}", track.length.as_secs() / 60)
            ),
            // Track length seconds
            guard.current_track.as_ref().map_or_else(
                || "--".to_owned(),
                |track| format!("{:02}", track.length.as_secs() % 60)
            ),
        ))
        .ratio(
            if let (Some(progress), Some(track)) =
                (guard.current_track_progress, &guard.current_track)
            {
                let ratio = progress.as_secs_f64() / track.length.as_secs_f64();
                if ratio < 0.0 || ratio.is_nan() {
                    0.0
                } else if ratio > 1.0 {
                    1.0
                } else {
                    ratio
                }
            } else {
                0.0
            },
        )
}
