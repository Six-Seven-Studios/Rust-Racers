use bevy::prelude::*;
use crate::car::{Car, Velocity, Orientation, PlayerControlled, CAR_SIZE};
use crate::networking_plugin::{NetworkClient, PlayerPositions};
use crate::lap_system::LapCounter;

#[derive(Component)]
pub struct NetworkPlayer {
    pub player_id: u32,
}

pub fn send_car_position(
    mut network_client: ResMut<NetworkClient>,
    player_query: Query<(&Transform, &Velocity, &Orientation), With<PlayerControlled>>,
) {
    let Some(client) = network_client.client.as_mut() else { return };
    let Ok((transform, velocity, orientation)) = player_query.single() else { return };
    
    let pos = transform.translation.truncate();
    let _ = client.send_car_position(pos.x, pos.y, velocity.velocity.x, velocity.velocity.y, orientation.angle);
}

pub fn get_car_positions(
    network_client: Res<NetworkClient>,
    mut network_cars: Query<(&NetworkPlayer, &mut Transform, &mut Velocity, &mut Orientation), Without<PlayerControlled>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    player_positions: Res<PlayerPositions>,
) {
    if network_client.client.is_none() { return }
    let my_id = network_client.player_id;

    // Process all positions from the resource
    for (id, player_pos) in &player_positions.positions {
        // Skip our own position
        if Some(*id) == my_id { continue; }

        compensate_lag(
            &mut network_cars,
            *id,
            player_pos.x,
            player_pos.y,
            player_pos.vx,
            player_pos.vy,
            player_pos.angle,
            &mut commands,
            &asset_server,
            &mut texture_atlases
        );
    }
}

fn compensate_lag(
    network_cars: &mut Query<(&NetworkPlayer, &mut Transform, &mut Velocity, &mut Orientation), Without<PlayerControlled>>,
    id: u32,
    x: f32, y: f32, vx: f32, vy: f32, angle: f32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    // Update existing car or spawn new one
    for (net_player, mut transform, mut velocity, mut orientation) in network_cars.iter_mut() {
        if net_player.player_id == id {
            transform.translation = Vec3::new(x, y, transform.translation.z);
            transform.rotation = Quat::from_rotation_z(angle);
            velocity.velocity = Vec2::new(vx, vy);
            orientation.angle = angle;
            return;
        }
    }
    
    // Spawn new car for this player
    let car_layout = TextureAtlasLayout::from_grid(UVec2::splat(CAR_SIZE), 2, 2, None, None);
    commands.spawn((
        Sprite::from_atlas_image(
            asset_server.load("red-car.png"),
            TextureAtlas { layout: texture_atlases.add(car_layout), index: 0 },
        ),
        Transform::from_xyz(x, y, 10.).with_rotation(Quat::from_rotation_z(angle)),
        Velocity::from(Vec2::new(vx, vy)),
        Orientation::new(angle),
        Car,
        NetworkPlayer { player_id: id },
        LapCounter::default(),
    ));
}

