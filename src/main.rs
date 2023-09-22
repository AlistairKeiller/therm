use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_prototype_lyon::prelude::*;
use bevy_xpbd_2d::{math::*, prelude::*};
use rand::prelude::*;

const BOX_WIDTH: Scalar = 1000.;
const BOX_HEIGHT: Scalar = 250.;
const BOX_POSITION: Vec2 = Vec2 { x: 0., y: 200. };
const BOX_THICKNESS: Scalar = 32.;

const PLOT_WIDTH: Scalar = BOX_WIDTH - BOX_THICKNESS * 6.;
const PLOT_HEIGHT: Scalar = BOX_HEIGHT;
const PLOT_POSITION: Vec2 = Vec2 { x: 0., y: -200. };

const GRID_WIDTH_OUT: i64 = 8;
const GRID_HEIGHT_OUT: i64 = 4;
const NUMBER_OF_PARTICLES: i64 = (GRID_WIDTH_OUT * 2 + 1) * (GRID_HEIGHT_OUT * 2 + 1);

const PARTICLE_MASS: Scalar = 1e-2; // kg
const PARTICLE_RADIUS: Scalar = 4.;
const HANDLE_RADIUS: Scalar = 16.;

const BOLTZMANN_CONSTANT: Scalar = 1.380649e-23; // J/K

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

#[derive(Component)]
struct IsothermicLine;

#[derive(Resource)]
struct Data {
    handle_x: Scalar,
    handle_y: Scalar,
    delta_handle_x: Scalar,
    delta_handle_y: Scalar,
}

fn get_volume(handle_x: Scalar) -> Scalar {
    return handle_x - (PLOT_POSITION.x - PLOT_WIDTH / 2.);
}

fn get_pressure(handle_y: Scalar) -> Scalar {
    return handle_y - (PLOT_POSITION.y - PLOT_HEIGHT / 2.);
}

fn get_pressure_from_volume_and_tempurature(volume: Scalar, tempurature: Scalar) -> Scalar {
    return tempurature * NUMBER_OF_PARTICLES as Scalar * BOLTZMANN_CONSTANT / volume;
}

fn get_tempurature(handle_x: Scalar, handle_y: Scalar) -> Scalar {
    return get_volume(handle_x) * get_pressure(handle_y)
        / (NUMBER_OF_PARTICLES as Scalar * BOLTZMANN_CONSTANT);
}

fn get_handle_x(volume: Scalar) -> Scalar {
    return volume + (PLOT_POSITION.x - PLOT_WIDTH / 2.);
}

fn get_handle_y(pressure: Scalar) -> Scalar {
    return pressure + (PLOT_POSITION.y - PLOT_HEIGHT / 2.);
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
                move_box_floor_and_ceiling,
                move_isobaric,
                move_isochoric,
                move_isothermic,
                fix_particles_location,
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

fn fix_particles_location(mut particles: Query<&mut Position, With<Particle>>, data: Res<Data>) {
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
        current_energy += PARTICLE_MASS * velocity.length_squared() / 2.;
    }
    let desired_energy =
        3. / 2. * BOLTZMANN_CONSTANT * get_tempurature(data.handle_x, data.handle_y);
    let scale = (desired_energy / current_energy).sqrt();
    for mut velocity in &mut particles {
        velocity.x *= scale;
        velocity.y *= scale;
    }
}

fn move_box_floor_and_ceiling(
    mut walls: Query<&mut Transform, With<BoxFloorOrCeiling>>,
    data: Res<Data>,
) {
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

fn move_isothermic(mut isothermics: Query<&mut Path, With<IsothermicLine>>, data: Res<Data>) {
    for mut path in &mut isothermics {
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(Vec2 {
            x: data.handle_x,
            y: data.handle_y,
        });
        for handle_x in ((PLOT_POSITION.x - PLOT_WIDTH / 2.) as i64..=data.handle_x as i64).rev() {
            let pressure = get_pressure_from_volume_and_tempurature(
                get_volume(handle_x as Scalar),
                get_tempurature(data.handle_x, data.handle_y),
            );
            if pressure > get_pressure(PLOT_POSITION.y + PLOT_HEIGHT / 2.) {
                break;
            }
            path_builder.line_to(Vec2 {
                x: handle_x as Scalar,
                y: get_handle_y(pressure),
            });
        }
        path_builder.move_to(Vec2 {
            x: data.handle_x,
            y: data.handle_y,
        });
        for handle_x in data.handle_x as i64..=(PLOT_POSITION.x + PLOT_WIDTH / 2.) as i64 {
            let pressure = get_pressure_from_volume_and_tempurature(
                get_volume(handle_x as Scalar),
                get_tempurature(data.handle_x, data.handle_y),
            );
            if pressure < get_pressure(PLOT_POSITION.y - PLOT_HEIGHT / 2.) {
                break;
            }
            path_builder.line_to(Vec2 {
                x: handle_x as Scalar,
                y: get_handle_y(pressure),
            });
        }
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
        Stroke::new(Color::BLUE, 5.0),
        IsobaricLine,
    ));
    commands.spawn((
        ShapeBundle { ..default() },
        Stroke::new(Color::GREEN, 5.0),
        IsochoricLine,
    ));
    commands.spawn((
        ShapeBundle { ..default() },
        Stroke::new(Color::YELLOW, 5.0),
        IsothermicLine,
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
    for x in -GRID_WIDTH_OUT..=GRID_WIDTH_OUT {
        for y in -GRID_HEIGHT_OUT..=GRID_HEIGHT_OUT {
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
