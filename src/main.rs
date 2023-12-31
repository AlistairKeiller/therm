use bevy::{prelude::*, sprite::Anchor, sprite::MaterialMesh2dBundle};
use bevy_prototype_lyon::prelude::*;
use bevy_xpbd_2d::{math::*, prelude::*};
use rand::prelude::*;

const BOX_WIDTH: Scalar = 1000.;
const BOX_HEIGHT: Scalar = 250.;
const BOX_POSITION: Vec2 = Vec2 { x: 0., y: 110. };
const BOX_THICKNESS: Scalar = 32.;

const PLOT_WIDTH: Scalar = BOX_WIDTH - BOX_THICKNESS * 6.;
const PLOT_HEIGHT: Scalar = BOX_HEIGHT;
const PLOT_POSITION: Vec2 = Vec2 { x: 0., y: -190. };

const GRID_WIDTH_OUT: i64 = 8;
const GRID_HEIGHT_OUT: i64 = 4;

const PARTICLE_MASS: Scalar = 1e-3; // kg
const PARTICLE_RADIUS: Scalar = 4.;
const HANDLE_RADIUS: Scalar = 16.;

const TEXT_OFFSET: Scalar = 10.;
const FONT_SIZE: Scalar = 40.;

const R: Scalar = 8.314; // J mol^-1 K^-1
const CP: Scalar = 5. / 2. * R; // J mol^-1 K^-1
const CV: Scalar = 3. / 2. * R; // J mol^-1 K^-1
const GAMMA: Scalar = CP / CV;
const N: Scalar = 1.; // actually n in mols

const PRESSURE_SCALE: Scalar = 10.;
const VOLUME_SCALE: Scalar = 10.;

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

#[derive(Component)]
struct AdiabaticLine;

#[derive(Component)]
struct TempuratureReading;

#[derive(Resource)]
struct Data {
    handle_x: Scalar,
    handle_y: Scalar,
    work: Scalar,
}

// m^3
fn get_volume(handle_x: Scalar) -> Scalar {
    (handle_x - (PLOT_POSITION.x - PLOT_WIDTH / 2.)) / VOLUME_SCALE
}

// Pa=N m^-2
fn get_pressure(handle_y: Scalar) -> Scalar {
    (handle_y - (PLOT_POSITION.y - PLOT_HEIGHT / 2.)) / PRESSURE_SCALE
}

// K
fn get_tempurature(handle_x: Scalar, handle_y: Scalar) -> Scalar {
    get_volume(handle_x) * get_pressure(handle_y) / (N * R)
}

// J
fn get_energy(handle_x: Scalar, handle_y: Scalar) -> Scalar {
    3. / 2. * N * R * get_tempurature(handle_x, handle_y)
}

fn get_handle_x(volume: Scalar) -> Scalar {
    volume * VOLUME_SCALE + PLOT_POSITION.x - PLOT_WIDTH / 2.
}

fn get_handle_y(pressure: Scalar) -> Scalar {
    pressure * PRESSURE_SCALE + PLOT_POSITION.y - PLOT_HEIGHT / 2.
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            ShapePlugin,
            PhysicsPlugins::default(),
        ))
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
                move_adiabatic,
                fix_particles_location,
                fix_particles_energy,
                update_tempurature_reading,
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
            data.work -= (get_pressure(data.handle_y) + get_pressure(new_handle_y))
                * (get_volume(data.handle_x) - get_volume(new_handle_x))
                / 2.;
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
    let scale = (get_energy(data.handle_x, data.handle_y) / current_energy).sqrt();
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
        for handle_y in data.handle_y as i64..=(PLOT_POSITION.y + PLOT_HEIGHT / 2.) as i64 {
            path_builder.line_to(Vec2 {
                x: get_handle_x(
                    get_volume(data.handle_x) * get_pressure(data.handle_y)
                        / get_pressure(handle_y as Scalar),
                ),
                y: handle_y as Scalar,
            });
        }
        path_builder.move_to(Vec2 {
            x: data.handle_x,
            y: data.handle_y,
        });
        for handle_x in data.handle_x as i64..=(PLOT_POSITION.x + PLOT_WIDTH / 2.) as i64 {
            path_builder.line_to(Vec2 {
                x: handle_x as Scalar,
                y: get_handle_y(
                    get_pressure(data.handle_y) * get_volume(data.handle_x)
                        / get_volume(handle_x as Scalar),
                ),
            });
        }
        *path = path_builder.build();
    }
}

fn move_adiabatic(mut isothermics: Query<&mut Path, With<AdiabaticLine>>, data: Res<Data>) {
    for mut path in &mut isothermics {
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(Vec2 {
            x: data.handle_x,
            y: data.handle_y,
        });
        for handle_y in data.handle_y as i64..=(PLOT_POSITION.y + PLOT_HEIGHT / 2.) as i64 {
            path_builder.line_to(Vec2 {
                x: get_handle_x(
                    (get_volume(data.handle_x).powf(GAMMA) * get_pressure(data.handle_y)
                        / get_pressure(handle_y as Scalar))
                    .powf(1. / GAMMA),
                ),
                y: handle_y as Scalar,
            });
        }
        path_builder.move_to(Vec2 {
            x: data.handle_x,
            y: data.handle_y,
        });
        for handle_x in data.handle_x as i64..=(PLOT_POSITION.x + PLOT_WIDTH / 2.) as i64 {
            path_builder.line_to(Vec2 {
                x: handle_x as Scalar,
                y: get_handle_y(
                    get_pressure(data.handle_y) * get_volume(data.handle_x).powf(GAMMA)
                        / get_volume(handle_x as Scalar).powf(GAMMA),
                ),
            });
        }
        *path = path_builder.build();
    }
}

fn update_tempurature_reading(
    mut tempurature_readings: Query<&mut Text, With<TempuratureReading>>,
    data: Res<Data>,
) {
    for mut text in &mut tempurature_readings {
        text.sections[0].value = format!(
            "T = {} K\nW = {} J\nQ = {} J",
            get_tempurature(data.handle_x, data.handle_y).round(),
            data.work.round(),
            (get_energy(data.handle_x, data.handle_y) + data.work).round()
        );
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
        work: 0.,
    });

    // lines on plot
    commands.spawn((
        ShapeBundle { ..default() },
        Stroke::new(Color::rgb_u8(5, 46, 22), 5.0),
        IsobaricLine,
    ));
    commands.spawn((
        ShapeBundle { ..default() },
        Stroke::new(Color::rgb_u8(23, 37, 84), 5.0),
        IsochoricLine,
    ));
    commands.spawn((
        ShapeBundle { ..default() },
        Stroke::new(Color::rgb_u8(69, 10, 10), 5.0),
        IsothermicLine,
    ));
    commands.spawn((
        ShapeBundle { ..default() },
        Stroke::new(Color::rgb_u8(59, 7, 100), 5.0),
        AdiabaticLine,
    ));

    // handle on plot
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

    // text labels
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "P",
            TextStyle {
                font_size: FONT_SIZE,
                color: Color::ANTIQUE_WHITE,
                ..default()
            },
        ),
        transform: Transform::from_translation(Vec3 {
            x: PLOT_POSITION.x - PLOT_WIDTH / 2. - TEXT_OFFSET,
            y: PLOT_POSITION.y,
            z: 0.,
        }),
        text_anchor: Anchor::CenterRight,
        ..default()
    });
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "V",
            TextStyle {
                font_size: FONT_SIZE,
                color: Color::ANTIQUE_WHITE,
                ..default()
            },
        ),
        transform: Transform::from_translation(Vec3 {
            x: PLOT_POSITION.x,
            y: PLOT_POSITION.y - PLOT_HEIGHT / 2. - TEXT_OFFSET,
            z: 0.,
        }),
        text_anchor: Anchor::TopCenter,
        ..default()
    });
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "isobaric",
            TextStyle {
                font_size: FONT_SIZE,
                color: Color::rgb_u8(5, 46, 22),
                ..default()
            },
        ),
        transform: Transform::from_translation(Vec3 {
            x: PLOT_POSITION.x + PLOT_WIDTH / 2. + TEXT_OFFSET,
            y: PLOT_POSITION.y - FONT_SIZE / 2.,
            z: 0.,
        }),
        text_anchor: Anchor::CenterLeft,
        ..default()
    });
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "isochoric",
            TextStyle {
                font_size: FONT_SIZE,
                color: Color::rgb_u8(23, 37, 84),
                ..default()
            },
        ),
        transform: Transform::from_translation(Vec3 {
            x: PLOT_POSITION.x + PLOT_WIDTH / 2. + TEXT_OFFSET,
            y: PLOT_POSITION.y + FONT_SIZE / 2.,
            z: 0.,
        }),
        text_anchor: Anchor::CenterLeft,
        ..default()
    });
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "isothermic",
            TextStyle {
                font_size: FONT_SIZE,
                color: Color::rgb_u8(69, 10, 10),
                ..default()
            },
        ),
        transform: Transform::from_translation(Vec3 {
            x: PLOT_POSITION.x + PLOT_WIDTH / 2. + TEXT_OFFSET,
            y: PLOT_POSITION.y + 3. * FONT_SIZE / 2.,
            z: 0.,
        }),
        text_anchor: Anchor::CenterLeft,
        ..default()
    });
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "adiabatic",
            TextStyle {
                font_size: FONT_SIZE,
                color: Color::rgb_u8(59, 7, 100),
                ..default()
            },
        ),
        transform: Transform::from_translation(Vec3 {
            x: PLOT_POSITION.x + PLOT_WIDTH / 2. + TEXT_OFFSET,
            y: PLOT_POSITION.y - 3. * FONT_SIZE / 2.,
            z: 0.,
        }),
        text_anchor: Anchor::CenterLeft,
        ..default()
    });
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font_size: FONT_SIZE,
                    color: Color::ANTIQUE_WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(Vec3 {
                x: BOX_POSITION.x,
                y: BOX_POSITION.y + BOX_HEIGHT / 2. + TEXT_OFFSET,
                z: 0.,
            }),
            text_anchor: Anchor::BottomCenter,
            ..default()
        },
        TempuratureReading,
    ));

    // plot background
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

    // ceiling
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
    // floor
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
    // right Wall
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
    // left wall
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

    // particles
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
