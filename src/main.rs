use std::{num::NonZero, time::Duration};

use bevy::{audio::PlaybackMode, prelude::*};
use phf::phf_set;
use walkdir::WalkDir;

#[derive(Resource, Default)]
struct AudioTimer(Timer);

#[derive(Component)]
struct AudioProvider(Handle<AudioSource>);

fn main() {
    App::new()
        .init_resource::<AudioTimer>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, read_audio)
        .add_systems(Update, spawn_audios.after(read_audio))
        .set_runner(run)
        .run();
}

fn run(mut app: App) -> AppExit {
    app.finish();
    app.cleanup();

    loop {
        app.update();

        if let Some(exit) = app.should_exit() {
            return exit;
        }

        std::thread::yield_now();
    }
}

fn read_audio(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for dir in WalkDir::new("./audio") {
        if let Ok(dir) = dir {
            if !dir.file_type().is_file() {
                // idk if it would count the dirs as well but this is just in case.
                continue;
            }

            let path = dir.path();
            let Some(ext) = path.extension() else { continue };
            let Some(ext) = ext.to_str() else { continue };

            if !(phf_set!{"ogg", "wav", "mp3"}.contains(&ext)) {
                debug!("Skipping: {:?}", dir.file_name());
                continue;
            }

            let audio = asset_server.load(path.canonicalize().unwrap());
            commands.spawn(AudioProvider(audio));
            info!("Loaded file: {:?}", dir.file_name());
        }
    }
}

fn spawn_audios(
    providers: Query<&AudioProvider>,
    time: Res<Time>,
    mut timer: ResMut<AudioTimer>,
    mut commands: Commands,
    mut app_exit: EventWriter<AppExit>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let Some(provider) = fastrand::choice(providers.iter()) else {
            error!("No valid audio files specified.");
            app_exit.send(AppExit::Error(NonZero::new(1).unwrap()));
            return;
        };

        commands.spawn((
            AudioPlayer(provider.0.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Loop,
                ..default()
            }
        ));

        timer.0.reset();
        timer.0.set_duration(Duration::from_secs_f32(fastrand::f32() * 1.25 + 0.5));
    }
}