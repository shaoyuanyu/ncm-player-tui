use crate::{NCM_CLIENT, PLAYER};
use anyhow::Result;

pub async fn init_songlists() -> Result<()> {
    let ncm_client_guard = NCM_CLIENT.lock().await;
    let mut player_guard = PLAYER.lock().await;
    if let Ok(songlists) = ncm_client_guard.get_user_all_songlists().await {
        let len = songlists.len();

        player_guard.set_playlist_candidates(songlists);

        if len > 0 {
            player_guard.switch_playlist(0, ncm_client_guard).await?;
        }
    }

    Ok(())
}
