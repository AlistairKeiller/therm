use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_prototype_lyon::prelude::*;
use bevy_xpbd_2d::{math::*, prelude::*};
use rand::prelude::*;

const BOX_WIDTH: f32 = 1000.;
const BOX_HEIGHT: f32 = 250.;
const BOX_POSITION: Vec2 = Vec2 { x: 0., y: 200. };
const BOX_THICKNESS: f32 = 32.;

const PLOT_WIDTH: f32 = BOX_WIDTH - BOX_THICKNESS * 6.;
const PLOT_HEIGHT: f32 = BOX_HEIGHT;
const PLOT_POSITION: Vec2 = Vec2 { x: 0., y: -200. };

const GRID_WIDTH_OUT: i32 = 8;
const GRID_HEIGHT_OUT: i32 = 4;

const PARTICLE_RADIUS: f32 = 4.;
const HANDLE_RADIUS: f32 = 16.;

const SPEED_SCALE: f32 = 16.;

#[derive(Component)]
struct Handle;

#[derive(Component)]
struct Piston;

#[derive(Component)]
struct Particle;

#[derive(Component)]
struct BoxFloorOrCeiling;

#[derive(Component)]
struct IsobaricLine;

#[derive(Component)]
struct IsochoricLine;

#[derive(Resource)]
struct Data {
    handle_x: f32,
    handle_y: f32,
    delta_handle_x: f32,
    delta_handle_y: f32,
}

fn get_tempurature(handle_x: f32, handle_y: f32) -> f32 {
    return (handle_x - (PLOT_POSITION.x - PLOT_WIDTH / 2.))
        * (handle_y - (PLOT_POSITION.y - PLOT_HEIGHT / 2.))
        * SPEED_SCALE;
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ShapePlugin, PhysicsPlugins::default()))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(Gravity::ZERO)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_pv_input,
                move_handle,
                move_piston,
                move_walls,
                move_isobaric,
                move_isochoric,
                fix_particles,
                fix_particles_energy,
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
            && mouse_position.x > PLOT_POSITION.x - PLOT_WIDTH / 2.
            && mouse_position.x < PLOT_POSITION.x + PLOT_WIDTH / 2.
            && mouse_position.y > PLOT_POSITION.y - PLOT_HEIGHT / 2.
            && mouse_position.y < PLOT_POSITION.y + PLOT_HEIGHT / 2.
        {
            let new_handle_x = mouse_position.x.clamp(
                PLOT_POSITION.x - PLOT_WIDTH / 2. + HANDLE_RADIUS,
                PLOT_POSITION.x + PLOT_WIDTH / 2. - HANDLE_RADIUS,
            );
            let new_handle_y = mouse_position.y.clamp(
                PLOT_POSITION.y - PLOT_HEIGHT / 2. + HANDLE_RADIUS,
                PLOT_POSITION.y + PLOT_HEIGHT / 2. - HANDLE_RADIUS,
            );
            data.delta_handle_x = new_handle_x - data.handle_x;
            data.delta_handle_y = new_handle_y - data.handle_y;
            data.handle_x = new_handle_x;
            data.handle_y = new_handle_y;
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
                BOX_POSITION.x - BOX_WIDTH / 2. + BOX_THICKNESS + PARTICLE_RADIUS
                    ..data.handle_x - BOX_THICKNESS / 2. - PARTICLE_RADIUS,
            );
            position.y = rng.gen_range(
                BOX_POSITION.y - BOX_HEIGHT / 2. + BOX_THICKNESS + PARTICLE_RADIUS
                    ..BOX_POSITION.y + BOX_HEIGHT / 2. - BOX_THICKNESS - PARTICLE_RADIUS,
            );
        }
    }
}

fn fix_particles_energy(
    mut particles: Query<&mut LinearVelocity, With<Particle>>,
    data: Res<Data>,
) {
    let mut current_energy = 0.;
    for velocity in &particles {
        current_energy += velocity.length_squared();
    }
    let scale = (get_tempurature(data.handle_x, data.handle_y) / current_energy).sqrt();
    for mut velocity in &mut particles {
        velocity.x *= scale;
        velocity.y *= scale;
    }
}

fn move_walls(mut walls: Query<&mut Transform, With<BoxFloorOrCeiling>>, data: Res<Data>) {
    for mut transform in &mut walls {
        transform.scale.x =
            (data.handle_x + BOX_THICKNESS / 2. - (BOX_POSITION.x - BOX_WIDTH / 2.)) / BOX_WIDTH;
        transform.translation.x =
            (data.handle_x + BOX_THICKNESS / 2. + BOX_POSITION.x - BOX_WIDTH / 2.) / 2.;
    }
}

fn move_isobaric(mut isobarics: Query<&mut Path, With<IsobaricLine>>, data: Res<Data>) {
    for mut path in &mut isobarics {
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(Vec2 {
            x: PLOT_POSITION.x - PLOT_WIDTH / 2.,
            y: data.handle_y,
        });
        path_builder.line_to(Vec2 {
            x: PLOT_POSITION.x + PLOT_WIDTH / 2.,
            y: data.handle_y,
        });
        *path = path_builder.build();
    }
}
fn move_isochoric(mut isobarics: Query<&mut Path, With<IsochoricLine>>, data: Res<Data>) {
    for mut path in &mut isobarics {
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(Vec2 {
            x: data.handle_x,
            y: PLOT_POSITION.y - PLOT_HEIGHT / 2.,
        });
        path_builder.line_to(Vec2 {
            x: data.handle_x,
            y: PLOT_POSITION.y + PLOT_HEIGHT / 2.,
        });
        *path = path_builder.build();
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.insert_resource(Data {
        handle_x: PLOT_POSITION.x,
        handle_y: PLOT_POSITION.y,
        delta_handle_x: 0.,
        delta_handle_y: 0.,
    });

    commands.spawn((
        ShapeBundle { ..default() },
        Stroke::new(Color::BLACK, 5.0),
        IsobaricLine,
    ));
    commands.spawn((
        ShapeBundle { ..default() },
        Stroke::new(Color::BLACK, 5.0),
        IsochoricLine,
    ));

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
        BoxFloorOrCeiling,
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
        BoxFloorOrCeiling,
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
                    mesh: meshes
                        .add(shape::Circle::new(PARTICLE_RADIUS).into())
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::rgb(0.29, 0.33, 0.64))),
                    ..default()
                },
                Collider::ball(PARTICLE_RADIUS),
                RigidBody::Dynamic,
                Position(
                    BOX_POSITION
                        + Vec2::new(
                            x as Scalar * (BOX_WIDTH - 2. * BOX_THICKNESS - 3. * PARTICLE_RADIUS)
                                / (GRID_WIDTH_OUT * 2) as Scalar,
                            y as Scalar * (BOX_HEIGHT - 2. * BOX_THICKNESS - 3. * PARTICLE_RADIUS)
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

    commands.spawn(Camera2dBundle::default());
}
