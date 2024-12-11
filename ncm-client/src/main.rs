use anyhow::Result;
use fast_qr::QRBuilder;
use ncm_client::NcmClient;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let api_program_path = PathBuf::from("/home/ysy/.local/share/ncm-tui/neteasecloudmusicapi/");
    let cookie_path = PathBuf::from("/home/ysy/.local/share/ncm-tui/cookie");
    let lyrics_path = PathBuf::from("/home/ysy/.cache/ncm-tui/");

    let mut ncm_client = NcmClient::new(api_program_path, cookie_path, lyrics_path);

    if ncm_client.check_api().await {
        // 尝试 cookie 登录
        if !ncm_client.try_cookie_login().await? {
            // cookie 登录失败，转而二维码登录
            let (qr_unikey, qr_url) = ncm_client.get_login_qr().await?;
            println!("{}, {}", &qr_unikey, &qr_url);

            let qr_code = QRBuilder::new(qr_url).build()?.to_str();
            println!("{}", &qr_code);

            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                let qr_status_code = ncm_client.check_login_qr(&qr_unikey).await?;
                println!("{}", qr_status_code);

                if qr_status_code == 803 {
                    break;
                }
            }
        }

        //
        ncm_client.check_login_status().await?;
    }

    // 获取用户所有歌单
    if let Ok(mut songlists) = ncm_client.get_user_all_songlists().await {
        if let Some(songlist) = songlists.get_mut(0) {
            ncm_client.load_songlist_songs(songlist).await?;
            println!(
                "name: {}, id: {}, len: {}",
                songlist.name,
                songlist.id,
                songlist.songs.len()
            );

            if let Some(song) = songlist.songs.get(10) {
                println!(
                    "name: {}, singer: {}, album: {}",
                    song.name, song.singer, song.album
                );

                if let Ok(lyrics) = ncm_client.get_song_lyrics(song.id).await {
                    println!("{:?}", lyrics);
                }
            }
        }
    }

    ncm_client.exit_client().await?;

    Ok(())
}
