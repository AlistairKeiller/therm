use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_xpbd_2d::{math::*, prelude::*};
use rand::prelude::*;

#[derive(Component)]
struct Controller;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(Gravity::ZERO)
        .add_systems(Startup, setup)
        .add_systems(Update, mouse_motion)
        .run();
}

fn mouse_motion(
    buttons: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera>>,
    mut controllers: Query<&mut Transform, With<Controller>>,
) {
    if buttons.pressed(MouseButton::Left) {
        if let Some(position) = windows.single().cursor_position().and_then(|cursor| {
            camera_q
                .single()
                .0
                .viewport_to_world_2d(camera_q.single().1, cursor)
        }) {
            for mut transform in &mut controllers {
                transform.translation.x = position.x;
                transform.translation.y = position.y;
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(20.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.2, 0.2, 0.2))),
            ..default()
        },
        Controller,
    ));

    let box_width = 50. * 20.;
    let box_height = 50. * 10.;
    let box_position = Vec2 { x: 0., y: 0. };
    let box_thickness = 50.;
    let grid_width_out = 10;
    let grid_height_out = 5;
    let ball_radius = 7.5;

    // Ceiling
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(box_width, box_thickness)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            box_position
                + Vec2 {
                    x: 0.,
                    y: (box_height - box_thickness) / 2.,
                },
        ),
        Collider::cuboid(box_width, box_thickness),
        Restitution::new(1.),
        Friction::new(0.),
    ));
    // Floor
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(box_width, box_thickness)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            box_position
                + Vec2 {
                    x: 0.,
                    y: -(box_height - box_thickness) / 2.,
                },
        ),
        Collider::cuboid(box_width, box_thickness),
        Restitution::new(1.),
        Friction::new(0.),
    ));
    // Right Wall
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(box_thickness, box_height)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            box_position
                + Vec2 {
                    x: (box_width - box_thickness) / 2.,
                    y: 0.,
                },
        ),
        Collider::cuboid(box_thickness, box_height),
        Restitution::new(1.),
        Friction::new(0.),
    ));
    // Left wall
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(box_thickness, box_height)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            box_position
                + Vec2 {
                    x: -(box_width - box_thickness) / 2.,
                    y: 0.,
                },
        ),
        Collider::cuboid(box_thickness, box_height),
        Restitution::new(1.),
        Friction::new(0.),
    ));
    let mut rng = rand::thread_rng();
    for x in -grid_width_out..grid_width_out + 1 {
        for y in -grid_height_out..grid_height_out + 1 {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(7.5).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::rgb(0.29, 0.33, 0.64))),
                    ..default()
                },
                Collider::ball(7.5),
                RigidBody::Dynamic,
                Position(
                    box_position
                        + Vec2::new(
                            x as Scalar * (box_width - 2. * box_thickness - 2. * ball_radius)
                                / (grid_width_out * 2) as Scalar,
                            y as Scalar * (box_height - 2. * box_thickness - 2. * ball_radius)
                                / (grid_height_out * 2) as Scalar,
                        ),
                ),
                Restitution::new(1.),
                Friction::new(0.),
                LinearVelocity(Vec2::new(
                    rng.gen_range(-200.0..200.0),
                    rng.gen_range(-200.0..200.0),
                )),
            ));
        }
    }
}
