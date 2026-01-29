use bevy::prelude::*;
use bevy::window::*;
use rand::prelude::*;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    InProcessGame,
    GameOver,
    Pause,
}

#[derive(Component, Default)]
struct VelocityInY {
    y: f32,
}

#[derive(Component)]
struct AnimationInFrames {
    images: Vec<Handle<Image>>,
    timer: Timer,
    current_frame: u8,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Cactus {
    velocity: f32,
}

#[derive(Component)]
struct GameOverTextStruct;

#[derive(Component)]
struct PauseStruct;

#[derive(Component)]
struct GameRestart;

#[derive(Component)]
struct GameWorld;

#[derive(Component)]
struct ScoreStruct;

#[derive(Resource)]
struct ScoreStructForGame {
    value: u128,
}

fn gravity_in_y(mut player_query: Query<(&mut Transform, &mut VelocityInY), With<Player>>,
time: Res<Time>) {
    let Ok((mut t, mut v)) = player_query.get_single_mut() else { return; };

    let g = -1150.0;

    v.y += g * time.delta_seconds();

    t.translation.y += v.y * time.delta_seconds();

    if t.translation.y <= 20.0 {
        t.translation.y = 20.0;
        v.y = 0.0;
    }
}

fn draw_background(time: Res<Time>, mut giz: Gizmos) {
    let frequency = 2.0;
    let amplitude = 1.0;
    let points = 100;
    let width = 700.0;

    let elapsed = time.elapsed_seconds();

    let mut path = Vec::new();
    for i in 0..=points {
        let x = (i as f32 / points as f32) * width - (width / 2.0);
        let y = (x * frequency * 0.02 + elapsed).sin() * amplitude;
        path.push(Vec2::new(x, y));
    }

    giz.linestrip_2d(path, Color::BLACK);
}

fn start(mut commands: Commands,
         asset_server: Res<AssetServer>,
) {
    let mut images_sprites = Vec::new();

    for i in 1..4 {
        images_sprites.push(asset_server.load(format!("images/player{}.png", i)));
    }

    commands.spawn((
        SpriteBundle {
            texture: images_sprites[0].clone(),
            transform: Transform::from_xyz(-310.0, 22.0, 0.0),
            ..default()
        },
        Player,
        VelocityInY::default(),
        AnimationInFrames {
            images: images_sprites.clone(),
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            current_frame: 0,
        },
    ));
}

fn animation_player(
    time: Res<Time>,
    mut animation_query: Query<(&mut AnimationInFrames, &mut Handle<Image>)>,
) {
    for (mut config, mut texture) in &mut animation_query {
        config.timer.tick(time.delta());

        if config.timer.just_finished() {
            config.current_frame =
                (config.current_frame + 1) % config.images.len() as u8;
            *texture = config.images[config.current_frame as usize].clone();
        }
    }
}

fn hide_game(
    mut query: Query<&mut Visibility, With<GameWorld>>,
) {
    for mut vis in &mut query {
        *vis = Visibility::Hidden;
    }
}

fn show_game(
    mut query: Query<&mut Visibility, With<GameWorld>>,
) {
    for mut vis in &mut query {
        *vis = Visibility::Inherited;
    }
}

fn keys(key_code: Res<Input<KeyCode>>, mut player_query: Query<(&mut Transform, &mut VelocityInY), With<Player>>, state: Res<State<GameState>>,
mut next_state: ResMut<NextState<GameState>>, mut commands: Commands, asset_server: Res<AssetServer>,
mut score_struct_for_game: ResMut<ScoreStructForGame>, query_pause: Query<Entity, With<PauseStruct>>) {
    let Ok((transform, mut v)) = player_query.get_single_mut() else {return;};

    if key_code.just_pressed(KeyCode::W) && *state.get() == GameState::InProcessGame
    && transform.translation.y <= 20.0 {
        v.y = 500.0;
        score_struct_for_game.value += 1;
    }

    if key_code.just_pressed(KeyCode::P) {
        if *state.get() == GameState::Pause {
            next_state.set(GameState::InProcessGame);
            for e in &query_pause {
                commands.entity(e).despawn_recursive();
            }
        } else if *state.get() == GameState::InProcessGame {
            next_state.set(GameState::Pause);
            commands.spawn((
                TextBundle::from_section(
                    "Пауза",
                    TextStyle {
                        font: asset_server.load("fonts/ArialMT.ttf"),
                        font_size: 60.0,
                        color: Color::BLACK,
                    },
                )
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(300.0),
                        left: Val::Px(200.0),
                        ..default()
                    }),
                PauseStruct,
            ));
        }
    }

    if key_code.just_pressed(KeyCode::Space) && *state.get() == GameState::GameOver {
        next_state.set(GameState::InProcessGame);
    }
}

fn delete_game_over(
    mut commands: Commands,
    q: Query<Entity, With<GameOverTextStruct>>,
) {
    for entity in &q {
        commands.entity(entity).despawn_recursive();
    }
}

fn collision_player_with_cactus(
    player_q: Query<&Transform, With<Player>>,
    mut player_image: Query<&mut Handle<Image>, With<Player>>,
    cactus_q: Query<&Transform, With<Cactus>>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut score_struct_for_game: ResMut<ScoreStructForGame>
) {
    if *state.get() != GameState::InProcessGame {
        return;
    }

    let Ok(player) = player_q.get_single() else { return };
    let Ok(mut player_texture) = player_image.get_single_mut() else { return };

    for cactus_tf in cactus_q.iter() {
        if collide(player.translation, cactus_tf.translation) {
            next_state.set(GameState::GameOver);
            *player_texture = asset_server.load("images/player4.png");

            commands.spawn((
                TextBundle::from_section(
                    "Ви програли гру",
                    TextStyle {
                        font: asset_server.load("fonts/ArialMT.ttf"),
                        font_size: 60.0,
                        color: Color::BLACK,
                    },
                )
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(300.0),
                        left: Val::Px(200.0),
                        ..default()
                    }),
                GameOverTextStruct,
            ));
            score_struct_for_game.value = 0;
        }
    }
}

fn update_score(
    score_struct_for_game: Res<ScoreStructForGame>,
    mut query: Query<&mut Text, With<ScoreStruct>>,
) {
    if score_struct_for_game.is_changed() {
        for mut text_score in &mut query {
            text_score.sections[0].value = format!("Очки: {}", score_struct_for_game.value);
        }
    }
}

fn collide(a: Vec3, b: Vec3) -> bool {
    let size_player = Vec2::new(40.0, 40.0);
    let size_cactus = Vec2::new(30.0, 40.0);

    let collision_x = (a.x - b.x).abs() < (size_player.x + size_cactus.x + 24.0) / 2.0;
    let collision_y = (a.y - b.y).abs() < (size_player.y + size_cactus.y + 24.0) / 2.0;

    collision_x && collision_y
}

fn load_cactus(asset_server: Res<AssetServer>, mut commands: Commands) {
    let image_cactus = asset_server.load("images/cactus.png");

    commands.spawn((
        SpriteBundle {
            texture: image_cactus.clone(),
            transform: Transform::from_xyz(600.0, 14.0, 0.0),
            ..default()
        },
        Cactus {
            velocity: thread_rng().gen_range(100.0..400.0),
        }
    ));
}

fn move_cactus(mut cactus_query: Query<(&mut Transform, &Cactus)>, time: Res<Time>) {
    for (mut t, cactus) in cactus_query.iter_mut() {
        t.translation.x -= cactus.velocity * time.delta_seconds();

        if t.translation.x <= -450.0 {
            t.translation.x = 450.0;
        }
    }
}

fn restart(
    q: Query<Entity, With<GameRestart>>,
    mut commands: Commands,
    key_code: Res<Input<KeyCode>>
) {
    if key_code.just_pressed(KeyCode::Space) {
        for entity in &q {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn score_function(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle::from_section(
            "Очки: 0",
            TextStyle {
                font: asset_server.load("fonts/times.ttf"),
                font_size: 30.0,
                color: Color::BLACK,
            }
        )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(30.0),
                left: Val::Px(20.0),
                ..default()
            }),
        ScoreStruct,
    ));
}

fn main() {
    let mut app = App::new();
    app
        .add_state::<GameState>()
        .insert_resource(ClearColor(Color::rgb(1., 1., 1.)))
        .insert_resource(ScoreStructForGame {value: 0})
        .add_plugins((
            DefaultPlugins.set(
                WindowPlugin {
                    primary_window: Some(Window {
                        title: "Dino game".into(),
                        resolution: WindowResolution::new(700.0, 700.0),
                        ..default()
                    }),
                    ..default()
                },
            ),
        ))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2dBundle::default());
        })
        .add_systems(Startup, (start, load_cactus, score_function))
        .add_systems(Update, restart.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, draw_background.run_if(in_state(GameState::InProcessGame)))
        .add_systems(Update, animation_player.run_if(in_state(GameState::InProcessGame)))
        .add_systems(Update, collision_player_with_cactus.run_if(in_state(GameState::InProcessGame)))
        .add_systems(Update, gravity_in_y.run_if(in_state(GameState::InProcessGame)))
        .add_systems(Update, move_cactus.run_if(in_state(GameState::InProcessGame)))
        .add_systems(OnEnter(GameState::Pause), hide_game)
        .add_systems(OnExit(GameState::Pause), show_game)
        .add_systems(OnExit(GameState::GameOver), delete_game_over)
        .add_systems(Update, keys)
        .add_systems(Update, update_score)
        .run();
}
