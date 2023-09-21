use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_xpbd_2d::{math::*, prelude::*};
use rand::prelude::*;

const BOX_WIDTH: f32 = 50. * 20.;
const BOX_HEIGHT: f32 = 50. * 5.;
const BOX_POSITION: Vec2 = Vec2 { x: 0., y: 200. };
const BOX_THICKNESS: f32 = 32.;

const PLOT_WIDTH: f32 = 50. * 20.;
const PLOT_HEIGHT: f32 = 50. * 5.;
const PLOT_POSITION: Vec2 = Vec2 { x: 0., y: -200. };

const GRID_WIDTH_OUT: i32 = 8;
const GRID_HEIGHT_OUT: i32 = 4;

const BALL_RADIUS: f32 = 4.;

#[derive(Component)]
struct Handle;

#[derive(Component)]
struct Piston;

#[derive(Component)]
struct Particle;

#[derive(Component)]
struct Test;

#[derive(Resource)]
struct Data {
    handle_x: f32,
    handle_y: f32,
    delta_handle_x: f32,
    delta_handle_y: f32,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(Gravity::ZERO)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_pv_input)
        .add_systems(Update, move_handle)
        .add_systems(Update, move_piston)
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
            && mouse_position.x < PLOT_POSITION.x + PLOT_WIDTH / 2.
            && mouse_position.x > PLOT_POSITION.x - PLOT_WIDTH / 2.
            && mouse_position.y < PLOT_POSITION.y + PLOT_HEIGHT / 2.
            && mouse_position.y > PLOT_POSITION.y - PLOT_HEIGHT / 2.
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

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(Data {
        handle_x: 0.,
        handle_y: 0.,
        delta_handle_x: 0.,
        delta_handle_y: 0.,
    });

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(20.).into()).into(),
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
        Piston,
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
