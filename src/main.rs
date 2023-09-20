use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_xpbd_2d::{math::*, prelude::*};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(Gravity::ZERO)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2dBundle::default());
    let boxWidth = 50. * 20.;
    let boxHeight = 50. * 11.;
    let boxPos = Vec2 { x: 0., y: 0. };
    let boxThickness = 5.;

    // Ceiling
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(boxWidth, boxThickness)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            boxPos
                + Vec2 {
                    x: 0.,
                    y: (boxHeight - boxThickness) / 2.,
                },
        ),
        Collider::cuboid(boxWidth, boxThickness),
        Restitution::new(1.),
        Friction::new(0.),
    ));
    // Floor
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(boxWidth, boxThickness)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            boxPos
                + Vec2 {
                    x: 0.,
                    y: -(boxHeight - boxThickness) / 2.,
                },
        ),
        Collider::cuboid(boxWidth, boxThickness),
        Restitution::new(1.),
        Friction::new(0.),
    ));
    // Right Wall
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(boxThickness, boxHeight)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            boxPos
                + Vec2 {
                    x: (boxWidth - boxThickness) / 2.,
                    y: 0.,
                },
        ),
        Collider::cuboid(boxThickness, boxHeight),
        Restitution::new(1.),
        Friction::new(0.),
    ));
    // Left wall
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(boxThickness, boxHeight)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.8))),
            ..default()
        },
        RigidBody::Static,
        Position(
            boxPos
                + Vec2 {
                    x: -(boxWidth - boxThickness) / 2.,
                    y: 0.,
                },
        ),
        Collider::cuboid(boxThickness, boxHeight),
        Restitution::new(1.),
        Friction::new(0.),
    ));
    for x in -12..12 {
        for y in -12..12 {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(7.5).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::rgb(0.29, 0.33, 0.64))),
                    ..default()
                },
                Collider::ball(7.5),
                RigidBody::Dynamic,
                Position(Vec2::new(x as Scalar * 20., y as Scalar * 20.)),
                Restitution::new(1.),
                Friction::new(0.),
                LinearVelocity(Vec2::new(x as Scalar * 20., y as Scalar * 20.)),
            ));
        }
    }
}
