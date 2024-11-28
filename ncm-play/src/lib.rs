use gstreamer_play::{gst, Play, PlayState, PlayVideoRenderer};

pub struct Player {
    play: Play,
    //
    play_state: PlayState,
}

impl Player {
    pub fn new() -> Self {
        gst::init().expect("Failed to initialize GST");

        let play = Play::new(None::<PlayVideoRenderer>);
        let mut config = play.config();
        config.set_user_agent(
            "User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:100.0) Gecko/20100101 Firefox/100.0",
        );
        config.set_position_update_interval(250);
        config.set_seek_accurate(true);
        play.set_config(config).unwrap();
        play.set_volume(0.2);

        Self {
            play,
            play_state: PlayState::Stopped,
        }
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.play.set_volume(volume);
    }

    pub fn mute(&mut self) {
        self.play.set_volume(0.0);
    }

    pub fn pause(&mut self) {
        if self.play_state == PlayState::Playing {
            self.play.pause();
            self.play_state = PlayState::Paused;
        }
    }

    pub fn play(&mut self) {
        if self.play_state == PlayState::Paused {
            self.play.play();
            self.play_state = PlayState::Playing;
        }
    }

    pub fn play_or_pause(&mut self) {
        if self.play_state == PlayState::Playing {
            self.play.pause();
            self.play_state = PlayState::Paused;
        } else if self.play_state == PlayState::Paused {
            self.play.play();
            self.play_state = PlayState::Playing;
        }
    }

    pub fn is_playing(&self) -> bool {
        self.play_state == PlayState::Playing
    }

    pub fn play_state(&self) -> String {
        self.play_state.clone().to_string()
    }

    pub fn duration(&self) -> Option<gst::ClockTime> {
        self.play.duration()
    }

    pub fn position(&self) -> Option<gst::ClockTime> {
        self.play.position()
    }

    pub fn play_new_song_by_uri(&mut self, uri: &str) {
        self.play.stop();
        self.play.set_uri(Some(uri));
        self.play.play();
        self.play_state = PlayState::Playing;
    }
}
