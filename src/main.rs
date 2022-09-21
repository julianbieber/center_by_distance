use bevy::prelude::*;
use nanorand::{Rng, WyRand};
use std::f32::consts::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Flip { b: false })
        .insert_resource(NextIterationTimer {
            t: Timer::from_seconds(0.05, true),
        })
        .add_startup_system(setup)
        .add_system(tick.before(turn_cyan))
        .add_system(turn_cyan.before(process_spheres))
        .add_system(process_spheres)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = WyRand::new();

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -5.0),
        ..default()
    });
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    for _ in 0..1000 {
        let coordinates = random_position(&mut rng);
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.01,
                    subdivisions: 10,
                })),
                material: materials.add(Color::CYAN.into()),
                transform: Transform::from_translation(coordinates),
                ..default()
            })
            .insert(Marker { visited: false });
    }
}

#[derive(Component)]
struct Marker {
    visited: bool,
}

struct Flip {
    b: bool,
}

struct NextIterationTimer {
    t: Timer,
}

fn turn_cyan(
    mut spheres_query: Query<(Entity, &Marker, &Transform, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    t: Res<NextIterationTimer>,
    flip: Res<Flip>,
) {
    if t.t.finished() && flip.b {
        for (_, _, _, mut mat) in spheres_query.iter_mut() {
            *mat = materials.add(Color::CYAN.into());
        }
    }
}

fn tick(time: Res<Time>, mut t: ResMut<NextIterationTimer>) {
    t.t.tick(time.delta());
}

fn process_spheres(
    mut spheres_query: Query<(
        Entity,
        &mut Marker,
        &Transform,
        &mut Handle<StandardMaterial>,
    )>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut flip: ResMut<Flip>,
    t: Res<NextIterationTimer>,
) {
    if t.t.finished() {
        if flip.b {
            for (_, _, _, mut mat) in spheres_query.iter_mut().filter(|v| !v.1.visited).take(3) {
                *mat = materials.add(Color::CRIMSON.into());
            }
        } else {
            let mut spheres = spheres_query.iter_mut().collect::<Vec<_>>();
            let mut unvisited = spheres.iter_mut().filter(|v| !v.1.visited);
            if let (Some(first), Some(second), Some(third)) =
                (unvisited.next(), unvisited.next(), unvisited.next())
            {
                first.1.visited = true;
                second.1.visited = true;
                third.1.visited = true;
                let (a, b, c) = dbg!(angles(first.2, second.2, third.2));
                if a > b && a > c {
                    dbg!(first.2.translation);
                    commands.entity(second.0).despawn_recursive();
                    commands.entity(third.0).despawn_recursive();
                } else if b > a && b > c {
                    dbg!(second.2.translation);
                    commands.entity(first.0).despawn_recursive();
                    commands.entity(third.0).despawn_recursive();
                } else {
                    dbg!(third.2.translation);
                    commands.entity(first.0).despawn_recursive();
                    commands.entity(second.0).despawn_recursive();
                }
            } else {
                for t in spheres.iter_mut() {
                    t.1.visited = false;
                }
            }
        }

        flip.b = !flip.b;
    }
}

fn angles(a: &Transform, b: &Transform, c: &Transform) -> (f32, f32, f32) {
    let ab = distance(a, b);
    let ac = distance(a, c);
    let bc = distance(b, c);

    let gamma = ((ab * ab - ac * ac - bc * bc) / (-2.0 * ac * bc)).acos();
    let beta = ((ac * ac - ab * ab - bc * bc) / (-2.0 * ab * bc)).acos();
    let alpha = ((bc * bc - ac * ac - ab * ab) / (-2.0 * ab * ac)).acos();
    (alpha, beta, gamma)
}

fn distance(a: &Transform, b: &Transform) -> f32 {
    a.translation.distance(b.translation)
}

fn random_position(rng: &mut WyRand) -> Vec3 {
    Vec3::new(random_coord(rng), random_coord(rng), random_coord(rng))
}

fn random_coord(rng: &mut WyRand) -> f32 {
    let i: i32 = rng.generate_range(0..=200);
    ((i - 100) as f32) / 100.0
}
