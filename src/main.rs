use bevy::prelude::*;
use nanorand::{Rng, WyRand};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Flip { b: false })
        .insert_resource(NextIterationTimer {
            t: Timer::from_seconds(0.05, true),
        })
        .insert_resource(FindCenter { select: 1000 })
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

    for _ in 0..10000 {
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
    mut spheres_query: Query<(Entity, &Marker, &Transform, &mut Handle<StandardMaterial>)>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut flip: ResMut<Flip>,
    t: Res<NextIterationTimer>,
    find_center: Res<FindCenter>,
) {
    if t.t.finished() {
        if flip.b {
            for (_, _, _, mut mat) in spheres_query
                .iter_mut()
                .filter(|v| !v.1.visited)
                .take(find_center.select)
            {
                *mat = materials.add(Color::CRIMSON.into());
            }
        } else {
            let spheres = spheres_query
                .iter()
                .map(|v| (v.0, v.2.clone()))
                .take(find_center.select)
                .collect::<Vec<_>>();
            let to_remove = find_center.remove(&spheres);
            for e in to_remove {
                commands.entity(e).despawn_recursive();
            }
        }

        flip.b = !flip.b;
    }

    if spheres_query.iter().len() == 1 {
        for (_, _, t, _) in spheres_query.iter() {
            println!("{t:?}");
        }
    }
}

struct FindCenter {
    select: usize,
}

impl FindCenter {
    fn remove(&self, values: &[(Entity, Transform)]) -> Vec<Entity> {
        if values.len() <= 1 {
            return Vec::new();
        }
        if let Some((e, _, _)) = values
            .iter()
            .map(|(e, t)| {
                let others: Vec<_> = values.iter().filter(|o| o.0 != *e).map(|o| o.1).collect();
                let d = FindCenter::mean_distance(t, &others);
                (e, t, d)
            })
            .min_by(|(_, _, d1), (_, _, d2)| d1.total_cmp(&d2))
        {
            values
                .iter()
                .filter(|o| o.0 != *e)
                .map(|(e, _)| *e)
                .collect()
        } else {
            vec![]
        }
    }

    fn mean_distance(t: &Transform, others: &[Transform]) -> f32 {
        others
            .iter()
            .map(|o| distance(t, o)) // * distance(t, o))
            .sum::<f32>()
            / others.len() as f32
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
