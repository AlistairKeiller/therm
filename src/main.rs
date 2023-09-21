use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_xpbd_2d::{math::*, prelude::*};
use rand::prelude::*;

const BOX_WIDTH: f32 = 50. * 20.;
const BOX_HEIGHT: f32 = 50. * 5.;
const BOX_POSITION: Vec2 = Vec2 { x: 0., y: 200. };
const BOX_THICKNESS: f32 = 32.;

const PLOT_WIDTH: f32 = BOX_WIDTH - BOX_THICKNESS * 6.;
const PLOT_HEIGHT: f32 = BOX_HEIGHT;
const PLOT_POSITION: Vec2 = Vec2 { x: 0., y: -200. };

const GRID_WIDTH_OUT: i32 = 8;
const GRID_HEIGHT_OUT: i32 = 4;

const BALL_RADIUS: f32 = 4.;

const SPEED_SCALE: f32 = 16.;

const HANDLE_RADIUS: f32 = 16.;

#[derive(Component)]
struct Handle;

#[derive(Component)]
struct Piston;

#[derive(Component)]
struct Particle;

#[derive(Component)]
struct Test;

#[derive(Component)]
struct MovingWall;

#[derive(Resource)]
struct Data {
    handle_x: f32,
    handle_y: f32,
    delta_handle_x: f32,
    delta_handle_y: f32,
}

impl Data {
    fn get_volume(&self) -> f32 {
        return self.handle_x - (PLOT_POSITION.x - PLOT_WIDTH / 2.);
    }
    fn get_pressure(&self) -> f32 {
        return self.handle_y - (PLOT_POSITION.y - PLOT_HEIGHT / 2.);
    }
    fn get_tempurature(&self) -> f32 {
        return self.get_volume() * self.get_pressure() * SPEED_SCALE;
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(Gravity::ZERO)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_pv_input,
                move_handle,
                move_piston,
                fix_particles,
                fix_particles_energy,
                move_walls,
            ),
        )
        .run();
}

fn handle_pv_input(
    buttons: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera>>,
    mut data: ResMut<Data>,
) {
    if let Some(mouse_position) = windows.single().cursor_position().and_then(|cursor| {
        camera_q
            .single()
            .0
            .viewport_to_world_2d(camera_q.single().1, cursor)
    }) {
        if buttons.pressed(MouseButton::Left)
            && mouse_position.x < PLOT_POSITION.x + PLOT_WIDTH / 2. - HANDLE_RADIUS
            && mouse_position.x > PLOT_POSITION.x - PLOT_WIDTH / 2. + HANDLE_RADIUS
            && mouse_position.y < PLOT_POSITION.y + PLOT_HEIGHT / 2. - HANDLE_RADIUS
            && mouse_position.y > PLOT_POSITION.y - PLOT_HEIGHT / 2. + HANDLE_RADIUS
        {
            data.delta_handle_x = mouse_position.x - data.handle_x;
            data.delta_handle_y = mouse_position.y - data.handle_y;
            data.handle_x = mouse_position.x;
            data.handle_y = mouse_position.y;
        }
    }
}

fn move_handle(mut handles: Query<&mut Transform, With<Handle>>, data: Res<Data>) {
    for mut transform in &mut handles {
        transform.translation.x = data.handle_x;
        transform.translation.y = data.handle_y;
    }
}

fn move_piston(mut pistons: Query<&mut Position, With<Piston>>, data: Res<Data>) {
    for mut position in &mut pistons {
        position.x = data.handle_x;
    }
}

fn fix_particles(mut particles: Query<&mut Position, With<Particle>>, data: Res<Data>) {
    let mut rng = rand::thread_rng();
    for mut position in &mut particles {
        if position.x < BOX_POSITION.x - BOX_WIDTH / 2.
            || position.x > data.handle_x - BOX_THICKNESS / 2.
            || position.y < BOX_POSITION.y - BOX_HEIGHT / 2.
            || position.y > BOX_POSITION.y + BOX_HEIGHT / 2.
        {
            position.x = rng.gen_range(
                BOX_POSITION.x - BOX_WIDTH / 2. + BOX_THICKNESS + BALL_RADIUS
                    ..data.handle_x - BOX_THICKNESS / 2.0 - BALL_RADIUS,
            );
            position.y = rng.gen_range(
                BOX_POSITION.y - BOX_HEIGHT / 2. + BOX_THICKNESS + BALL_RADIUS
                    ..BOX_POSITION.y + BOX_HEIGHT / 2. - BOX_THICKNESS - BALL_RADIUS,
            );
        }
    }
}

fn fix_particles_energy(
    mut particles: Query<&mut LinearVelocity, With<Particle>>,
    data: Res<Data>,
) {
    let mut current_energy = 0.;
    for velocity in &mut particles {
        current_energy += velocity.length_squared();
    }
    let scale = (data.get_tempurature() / current_energy).sqrt();
    for mut velocity in &mut particles {
        velocity.x *= scale;
        velocity.y *= scale;
    }
}

fn move_walls(
    mut walls: Query<&mut Transform, With<MovingWall>>,
    data: Res<Data>,
    buttons: Res<Input<MouseButton>>,
) {
    for mut transform in &mut walls {
        transform.scale.x =
            (data.handle_x + BOX_THICKNESS / 2. - (BOX_POSITION.x - BOX_WIDTH / 2.)) / BOX_WIDTH;
        transform.translation.x =
            (BOX_POSITION.x - BOX_WIDTH / 2. + data.handle_x + BOX_THICKNESS / 2.) / 2.;
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(Data {
        handle_x: PLOT_POSITION.x,
        handle_y: PLOT_POSITION.y,
        delta_handle_x: 0.,
        delta_handle_y: 0.,
    });

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(HANDLE_RADIUS).into()).into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.2, 0.2, 0.2))),
            transform: Transform::from_translation(Vec3 {
                x: PLOT_POSITION.x,
                y: PLOT_POSITION.y,
                z: 1.,
            }),
            ..default()
        },
        Handle,
    ));

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(shape::Quad::new(Vec2::new(PLOT_WIDTH, PLOT_HEIGHT)).into())
            .into(),
        material: materials.add(ColorMaterial::from(Color::rgb(0.1, 0.1, 0.1))),
        transform: Transform::from_translation(Vec3 {
            x: PLOT_POSITION.x,
            y: PLOT_POSITION.y,
            z: 0.,
        }),
        ..default()
    });

    // Ceiling
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(BOX_WIDTH, BOX_THICKNESS)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            BOX_POSITION
                + Vec2 {
                    x: 0.,
                    y: (BOX_HEIGHT - BOX_THICKNESS) / 2.,
                },
        ),
        Collider::cuboid(BOX_WIDTH, BOX_THICKNESS),
        Restitution::new(1.),
        Friction::new(0.),
        MovingWall,
    ));
    // Floor
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(BOX_WIDTH, BOX_THICKNESS)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            BOX_POSITION
                + Vec2 {
                    x: 0.,
                    y: -(BOX_HEIGHT - BOX_THICKNESS) / 2.,
                },
        ),
        Collider::cuboid(BOX_WIDTH, BOX_THICKNESS),
        Restitution::new(1.),
        Friction::new(0.),
        MovingWall,
    ));
    // Right Wall
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(BOX_THICKNESS, BOX_HEIGHT)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            BOX_POSITION
                + Vec2 {
                    x: (BOX_WIDTH - BOX_THICKNESS) / 2.,
                    y: 0.,
                },
        ),
        Collider::cuboid(BOX_THICKNESS, BOX_HEIGHT),
        Restitution::new(1.),
        Friction::new(0.),
        Piston,
    ));
    // Left wall
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(BOX_THICKNESS, BOX_HEIGHT)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            BOX_POSITION
                + Vec2 {
                    x: -(BOX_WIDTH - BOX_THICKNESS) / 2.,
                    y: 0.,
                },
        ),
        Collider::cuboid(BOX_THICKNESS, BOX_HEIGHT),
        Restitution::new(1.),
        Friction::new(0.),
    ));
    let mut rng = rand::thread_rng();
    for x in -GRID_WIDTH_OUT..GRID_WIDTH_OUT + 1 {
        for y in -GRID_HEIGHT_OUT..GRID_HEIGHT_OUT + 1 {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(BALL_RADIUS).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::rgb(0.29, 0.33, 0.64))),
                    ..default()
                },
                Collider::ball(BALL_RADIUS),
                RigidBody::Dynamic,
                Position(
                    BOX_POSITION
                        + Vec2::new(
                            x as Scalar * (BOX_WIDTH - 2. * BOX_THICKNESS - 3. * BALL_RADIUS)
                                / (GRID_WIDTH_OUT * 2) as Scalar,
                            y as Scalar * (BOX_HEIGHT - 2. * BOX_THICKNESS - 3. * BALL_RADIUS)
                                / (GRID_HEIGHT_OUT * 2) as Scalar,
                        ),
                ),
                Restitution::new(1.),
                Friction::new(0.),
                LinearVelocity(Vec2::new(
                    rng.gen_range(-200.0..200.0),
                    rng.gen_range(-200.0..200.0),
                )),
                Particle,
            ));
        }
    }
}
