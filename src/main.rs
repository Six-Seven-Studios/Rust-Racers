use bevy::{prelude::*, window::PresentMode};

#[derive(Component, Deref, DerefMut)]
struct PopupTimer(Timer);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
    Credits,
}

#[derive(Component)]
struct CreditsEntity;

#[derive(Resource)]
struct CreditsTimer(Timer);

const WIN_W: f32 = 1280.;
const WIN_H: f32 = 720.;

const PLAYER_SPEED: f32 = 350.;
const ACCEL_RATE: f32 = 700.;
const FRICTION: f32 = 0.95;

const TURNING_RATE: f32 = 3.5;

const CAR_SIZE: u32 = 64;

const LEVEL_W: f32 = 1920.;
const LEVEL_H: f32 = 1080.;

#[derive(Component)]
struct Car;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Orientation {
    angle: f32,
}

impl Orientation {
    fn new(angle: f32) -> Self {
        Self { angle }
    }
    
    fn forward_vector(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }
}

#[derive(Component, Deref, DerefMut)]
struct Velocity {
    velocity: Vec2,
}

impl Velocity {
    fn new() -> Self {
        Self {
            velocity: Vec2::ZERO,
        }
    }
    
}

impl From<Vec2> for Velocity {
    fn from(velocity: Vec2) -> Self {
        Self { velocity }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Racers".into(),
                resolution: (WIN_W, WIN_H).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            check_for_credits_input,
            move_car.run_if(in_state(GameState::Playing)),
            move_camera.after(move_car).run_if(in_state(GameState::Playing)),
        ))
        .add_systems(OnEnter(GameState::Credits), setup_credits)
        .add_systems(OnEnter(GameState::Credits), reset_camera_for_credits.after(setup_credits))
        .add_systems(Update, show_credits.run_if(in_state(GameState::Credits)))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    let track_texture_handle = asset_server.load("track.png");

    commands.spawn((
        Sprite::from_image(track_texture_handle.clone()),
        Transform::from_translation(Vec3::ZERO),
        Background,
    ));

    let car_sheet_handle = asset_server.load("car.png");
    let car_layout = TextureAtlasLayout::from_grid(UVec2::splat(CAR_SIZE), 2, 2, None, None);
    let car_layout_handle = texture_atlases.add(car_layout);

    commands.spawn((
        Sprite::from_atlas_image(
            car_sheet_handle,
            TextureAtlas {
                layout: car_layout_handle.clone(),
                index: 0,
            },
        ),
        Transform {
            translation: Vec3::new(0., 0., 50.),
            ..default()
        },
        Velocity::new(),
        Orientation::new(0.0),
        Car,
    ));
}

fn move_car(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    car: Single<(&mut Transform, &mut Velocity, &mut Orientation), (With<Car>, Without<Background>)>,
) {
    let (mut transform, mut velocity, mut orientation) = car.into_inner();

    let deltat = time.delta_secs();
    let accel = ACCEL_RATE * deltat;

    // Turning
    if input.pressed(KeyCode::KeyA) {
        orientation.angle += TURNING_RATE * deltat;
    }
    if input.pressed(KeyCode::KeyD) {
        orientation.angle -= TURNING_RATE * deltat;
    }

    // Accelerate forward in the direction of car orientation
    if input.pressed(KeyCode::KeyW) {
        let forward = orientation.forward_vector() * accel;
        **velocity += forward;
        **velocity = velocity.clamp_length_max(PLAYER_SPEED);
    }

    // Accelerate in the direction opposite of orientation
    if input.pressed(KeyCode::KeyS) {
        let backward = -orientation.forward_vector() * accel;
        **velocity += backward;
        **velocity = velocity.clamp_length_max(PLAYER_SPEED);
    }

    // Friction when not accelerating
    if !input.any_pressed([KeyCode::KeyW, KeyCode::KeyS]) {
        **velocity *= FRICTION;
    }

    // Updated position
    let change = **velocity * deltat;

    let min = Vec3::new(
        -LEVEL_W / 2. + (CAR_SIZE as f32) / 2.,
        -LEVEL_H / 2. + (CAR_SIZE as f32) / 2.,
        900.,
    );
    let max = Vec3::new(
        LEVEL_W / 2. - (CAR_SIZE as f32) / 2.,
        LEVEL_H / 2. - (CAR_SIZE as f32) / 2.,
        900.,
    );

    // Rotate car to match orientation
    transform.rotation = Quat::from_rotation_z(orientation.angle);

    // Update car position
    transform.translation = (transform.translation + change.extend(0.)).clamp(min, max);
}

fn move_camera(
    car: Single<&Transform, With<Car>>,
    mut camera: Single<&mut Transform, (With<Camera>, Without<Car>)>,
) {
    let max = Vec3::new(LEVEL_W / 2. - WIN_W / 2., LEVEL_H / 2. - WIN_H / 2., 0.);
    let min = -max.clone();
    camera.translation = car.translation.clamp(min, max);
}

fn check_for_credits_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if input.just_pressed(KeyCode::Space) && *current_state == GameState::Playing {
        next_state.set(GameState::Credits);
    }
}

fn reset_camera_for_credits(mut camera: Single<&mut Transform, With<Camera>>) {
    camera.translation = Vec3::ZERO;
}

fn setup_credits(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut car: Single<&mut Visibility, (With<Car>, Without<Background>)>,
    mut background: Single<&mut Visibility, (With<Background>, Without<Car>)>,
) {
    commands.insert_resource(CreditsTimer(Timer::from_seconds(20.0, TimerMode::Once)));
    
    **car = Visibility::Hidden;
    **background = Visibility::Hidden;
    
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/rust-racers.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.),
            ..default()
        },
        PopupTimer(Timer::from_seconds(0., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/developed-by.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.1),
            ..default()
        },
        PopupTimer(Timer::from_seconds(2., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/kameren-jouhal.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.2),
            ..default()
        },
        PopupTimer(Timer::from_seconds(4., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/greyson-barsotti.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.3),
            ..default()
        },
        PopupTimer(Timer::from_seconds(6., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/ethan-defilippi.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.4),
            ..default()
        },
        PopupTimer(Timer::from_seconds(8., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/carson-gollinger.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.5),
            ..default()
        },
        PopupTimer(Timer::from_seconds(10., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/jonathan-coulter.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.6),
            ..default()
        },
        PopupTimer(Timer::from_seconds(12., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/jeremy-luu.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.7),
            ..default()
        },
        PopupTimer(Timer::from_seconds(14., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/david-shi.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.8),
            ..default()
        },
        PopupTimer(Timer::from_seconds(16., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/Daniel.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.9),
            ..default()
        },
        PopupTimer(Timer::from_seconds(18., TimerMode::Once)),
        CreditsEntity,
    ));
}

fn show_credits(
    time: Res<Time>, 
    mut popup: Query<(&mut PopupTimer, &mut Transform), With<CreditsEntity>>,
    mut credits_timer: ResMut<CreditsTimer>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    let mut counter = 2.;
    
    for (mut timer, mut transform) in popup.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.translation.z += counter;
            counter += 1.;
        }
    }
    
    credits_timer.0.tick(time.delta());
    
    if credits_timer.0.just_finished() {
        exit.write(bevy::app::AppExit::Success);
    }
}