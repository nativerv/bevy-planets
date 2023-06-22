use bevy::render::render_resource::Face;
use bevy::{
  ecs::system::EntityCommands, input::mouse::MouseWheel, prelude::*, sprite::MaterialMesh2dBundle,
};

use std::f32::consts::PI;

// const SIZE: f32 = 10.0;

#[derive(Component, Debug)]
struct AttachedToCursor;

#[derive(Component, Debug)]
struct MainCamera;

#[derive(Component, Debug)]
struct Planet;

#[derive(Component, Debug)]
struct Player;

#[derive(Component, Debug)]
struct ShowDirectionLines;

#[derive(Component, Debug)]
struct ShowDirectionLinesApplied;

#[derive(Component, Debug)]
struct DirectionLine;

const PLAYER_HEIGHT: f32 = 0.4;
const PLAYER_WALKING_SPEED: f32 = 30.0;
const PLAYER_COLOR: Color = Color::rgb(0.1, 0.8, 0.1);

const DIRECTION_LINE_COLOR: Color = Color::rgb(0.9, 0.1, 0.1);

const CAMERA_ZOOM_SPEED: f32 = 40.0;
const CAMERA_INITIAL_DISTANCE: f32 = 30.0;
const CAMERA_MAX_DISTANCE: f32 = 150.0;
const CAMERA_MIN_DISTANCE: f32 = 1.0;

const PLANET_COLOR: Color = Color::rgb(0.1, 0.1, 0.9);

const PLANET_RADIUS: f32 = 30.0;
const PLANET_MIN_QUALITY: f32 = 160.0;
const PLANET_GRAVITY_SPEED: f32 = 4.0;

const SUN_RADIUS: f32 = 5.0;
const SUN_RANGE: f32 = PLANET_RADIUS * 500.0;
const SUN_INTENSITY: f32 = 10_000.0;
const SUN_SOFTNESS: f32 = 0.0;
const SUN_DISTANCE: f32 = PLANET_RADIUS * 2.0;
const SUN_LOCATION: Transform = Transform::from_xyz(0.0, SUN_DISTANCE, 0.0);
const SUN_COLOR: Color = Color::rgb(255.0, 248.0, 224.0);
const SUN_SHADOW_DEPTH_BIAS: f32 = 2.00;
const SUN_SHADOW_NORMAL_BIAS: f32 = 2.00;

macro_rules! extend_commands {
    ($command_name:ident($( $arg:ident: $arg_type:ty ),*), $command_fn:expr) => {
        pub(crate) trait $command_name<'w, 's> {
            fn $command_name<'a>(
                &'a mut self,
                $($arg: $arg_type),*
            ) -> EntityCommands<'w, 's, 'a>;
        }

        impl<'w, 's> $command_name<'w, 's> for Commands<'w, 's> {
            fn $command_name<'a>(
                &'a mut self,
                $($arg: $arg_type),*
            ) -> EntityCommands<'w, 's, 'a> {
                let entity = self.spawn_empty();
                let entity_id = entity.id();

                self.add(move |world: &mut World| {
                    $command_fn(world, entity_id, $($arg),*);
                });

                self.entity(entity_id)
            }
        }
    };
}

extend_commands!(
  spawn_circle(position: Vec3, radius: f32, color: Color),
  |world: &mut World, entity_id: Entity, position: Vec3, radius: f32, color: Color| {
    let mut mesh = world.resource_mut::<Assets<Mesh>>().add(Mesh::from(shape::UVSphere {
        radius,
        sectors: (radius * 8.0).max(PLANET_MIN_QUALITY) as usize,
        stacks: (radius * 8.0).max(PLANET_MIN_QUALITY) as usize,
      }));
    let mut material = world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
        base_color: color.into(),
        ..default()
      });

    world
      .entity_mut(entity_id)
      .insert(PbrBundle {
      mesh,
      material,
      transform: Transform::from_translation(position),
      ..default()
    });
  }
);

fn setup(
  mut commands: Commands,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut meshes: ResMut<Assets<Mesh>>,
  asset_server: Res<AssetServer>,
) {
  commands.insert_resource(ClearColor(Color::BLACK));
  commands.spawn_circle(
    Vec3::new(PLANET_RADIUS * 1.0 + 5.0, 0.0, 0.0),
    3.0,
    Color::rgb(1.0, 0.0, 0.0),
  );
  commands.spawn_circle(
    Vec3::new(0.0, PLANET_RADIUS * 1.0 + 5.0, 0.0),
    3.0,
    Color::rgb(1.0, 1.0, 0.0),
  );
  commands.spawn_circle(
    Vec3::new(0.0, 0.0, PLANET_RADIUS * 1.0 + 5.0),
    3.0,
    Color::rgb(0.0, 1.0, 0.0),
  );
  // sun (light)
  commands.spawn(PointLightBundle {
    point_light: PointLight {
      intensity: SUN_INTENSITY,
      radius: SUN_SOFTNESS,
      range: SUN_RANGE,
      color: SUN_COLOR,
      shadows_enabled: true,
      // shadow_depth_bias: SUN_SHADOW_DEPTH_BIAS,
      // shadow_normal_bias: SUN_SHADOW_NORMAL_BIAS,
      ..default()
    },
    transform: SUN_LOCATION,
    ..default()
  });
  // commands.insert_resource(AmbientLight {
  //   color: SUN_COLOR,
  //   brightness: 0.2,
  // });
  // sphere/planet
  // this material renders the texture normally
  commands
    .spawn(PbrBundle {
      mesh: meshes.add(Mesh::from(shape::UVSphere {
        radius: PLANET_RADIUS,
        // TODO: on PLANET_RADIUS value of 30.0 the circle becomes a triangle,
        // play with that.
        sectors: (PLANET_RADIUS * 8.0).max(PLANET_MIN_QUALITY) as usize,
        stacks: (PLANET_RADIUS * 8.0).max(PLANET_MIN_QUALITY) as usize,
      })),
      material: materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("earth-4k.jpg")),
        //base_color: PLANET_COLOR,
        //perceptual_roughness: 1.0,
        //alpha_mode: AlphaMode::Blend,
        //unlit: false,
        //cull_mode: None,
        ..default()
      }),
      transform: Transform::from_xyz(0.0, 0.0, 0.0),
      ..default()
    })
    .insert(Planet);
  // player (capsule)
  commands
    .spawn(PbrBundle {
      mesh: meshes.add(Mesh::from(shape::Capsule {
        radius: 0.1,
        rings: 30,
        depth: PLAYER_HEIGHT,
        latitudes: 30,
        longitudes: 30,
        ..default()
      })),
      material: materials.add(PLAYER_COLOR.into()),
      transform: Transform::from_xyz(0.0, 0.0, PLANET_RADIUS + PLAYER_HEIGHT)
        .with_rotation(Quat::from_rotation_x(PI / 2.0)),
      ..default()
    })
    .insert(Player)
    .insert(ShowDirectionLines)
    .with_children(|child_commands| {
      child_commands
        .spawn(Camera3dBundle {
          transform: Transform::from_xyz(
            0.0,
            CAMERA_INITIAL_DISTANCE,
            CAMERA_INITIAL_DISTANCE / 2.5,
          )
          .looking_at(Vec3::ZERO, Vec3::Y),
          ..default()
        })
        .insert(MainCamera);
    });
}

fn move_entities(mut query: Query<(&mut Transform, &Name)>, time: Res<Time>) {
  const RADIUS: f32 = 30.0;

  for (mut transform, ..) in &mut query {
    transform.translation.x = RADIUS * time.elapsed_seconds().cos();
    transform.translation.y = RADIUS * time.elapsed_seconds().sin();
  }
}

fn enable_direction_lines(
  query: Query<
    (&Transform, Entity),
    (With<ShowDirectionLines>, Without<ShowDirectionLinesApplied>),
  >,
  mut commands: Commands,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut meshes: ResMut<Assets<Mesh>>,
) {
  for (transform, entity) in &query {
    commands
      .entity(entity)
      .insert(ShowDirectionLinesApplied)
      .with_children(|child_commands| {
        const LENGTH: f32 = 0.4;
        child_commands
          .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
              radius: 0.01,
              rings: 30,
              depth: LENGTH,
              latitudes: 30,
              longitudes: 30,
              ..default()
            })),
            material: materials.add(DIRECTION_LINE_COLOR.into()),
            transform: Transform::from_xyz(0.0, PLAYER_HEIGHT / 1.8, -LENGTH / 2.0)
              .with_rotation(Quat::from_rotation_x(PI / 2.0)),
            ..default()
          })
          .insert(DirectionLine);
      });
  }
}

fn disable_direction_lines(
  parents_q: Query<Entity, (Without<ShowDirectionLines>, With<ShowDirectionLinesApplied>)>,
  children_q: Query<(Entity, &Parent), With<DirectionLine>>,
  mut commands: Commands,
) {
  for (direction_line_entity, parent_entity) in &children_q {
    if let Ok(_) = parents_q.get(parent_entity.get()) {
      commands.entity(direction_line_entity).despawn();
      commands
        .entity(parent_entity.get())
        .remove::<ShowDirectionLinesApplied>();
    }
  }
}

fn toggle_direction_lines(
  keys: Res<Input<KeyCode>>,
  mut query_with: Query<Entity, With<ShowDirectionLines>>,
  mut query_without: Query<Entity, Without<ShowDirectionLines>>,
  mut commands: Commands,
) {
  // PERF: each keypress causes exponentially (or not) longer freeze.
  // TODO: investigate that the cause of the freeze
  if keys.just_pressed(KeyCode::RBracket) {
    for entity in &mut query_with {
      commands.entity(entity).remove::<ShowDirectionLines>();
    }
    for entity in &mut query_without {
      // BUG/FIXME: crash at this line if the key is pressed too often.
      // message: Could not insert a bundle (of type `bevy_planet::ShowDirectionLines`) for entity
      // 288v1 because it doesn't exist in this World.
      commands.entity(entity).insert(ShowDirectionLines);
    }
  }
}

fn player_controls(
  keys: Res<Input<KeyCode>>,
  mut query: Query<&mut Transform, With<Player>>,
  mut commands: Commands,
  time: Res<Time>,
) {
  for mut player_transform in &mut query {
    #[rustfmt::skip]
    let speed = PLAYER_WALKING_SPEED * (
      player_transform.forward() * (f32::from(keys.pressed(KeyCode::W)) + -f32::from(keys.pressed(KeyCode::S))) +
      player_transform.left() * (f32::from(keys.pressed(KeyCode::A)) + -f32::from(keys.pressed(KeyCode::D))) +
      player_transform.up() * (f32::from(keys.pressed(KeyCode::Space)) + -f32::from(keys.pressed(KeyCode::LControl)))
    );
    player_transform.translation += speed * time.delta_seconds();
  }
}

fn player_mouse_controls(
  mut wheel: EventReader<MouseWheel>,
  mut camera_q: Query<&mut Transform, With<MainCamera>>,
  time: Res<Time>,
) {
  use bevy::input::mouse::MouseScrollUnit;

  for event in wheel.iter() {
    for mut transform in &mut camera_q {
      let speed = transform.forward() * event.y * CAMERA_ZOOM_SPEED * time.delta_seconds();
      let mut new_translation = transform.translation;
      new_translation += speed;
      // FIXME: camera still scrolls around the parent at min allowed length
      new_translation = new_translation.clamp_length(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
      transform.translation = new_translation;
    }
  }
}

fn attract_to_planets(
  planets_q: Query<&Transform, (With<Planet>, Without<Player>)>,
  mut players_q: Query<&mut Transform, (With<Player>, Without<Planet>)>,
  time: Res<Time>,
) {
  return;
  for planet_transform in &planets_q {
    for mut player_transform in &mut players_q {
      let a = (planet_transform.translation - player_transform.translation).normalize()
        * PLANET_GRAVITY_SPEED
        * time.delta_seconds();
      player_transform.translation += a;
    }
  }
}

fn align_to_planets(
  planets_q: Query<&Transform, (With<Planet>, Without<Player>)>,
  mut players_q: Query<&mut Transform, (With<Player>, Without<Planet>)>,
) {
  for planet_transform in &planets_q {
    for mut player_transform in &mut players_q {
      let planet_to_player = player_transform.translation - planet_transform.translation;
      let player_to_planet = planet_transform.translation - player_transform.translation;
      let m = planet_to_player.normalize();
      let cos_alpha = m.x;
      let cos_beta = m.y;
      let cos_gamma = m.z;
      let rotation_x = cos_alpha.acos();
      let rotation_y = cos_beta.acos();
      let rotation_z = cos_gamma.acos();
      // dbg!(rotation_x);
      // dbg!(rotation_y);
      // dbg!(rotation_z);
      // dbg!(planet_to_player.length());

      let pomoika = player_transform.forward().cross(planet_to_player);
      let x = pomoika.x;
      let y = pomoika.y;
      let nz = pomoika.z;
      let nx = x * (PI / 2.0).cos() - y * (PI / 2.0).sin();
      let ny = x * (PI / 2.0).sin() + y * (PI / 2.0).cos();
      let forward = Vec3::new(nx, ny, nz).normalize();

      let f0 = player_transform.forward();
      let a = player_to_planet;
      let c = f0.cross(a);
      let f1 = a.cross(c);
      player_transform.look_to(f1, planet_to_player);
    }
  }
}

fn collide_with_planets(
  planets_q: Query<&Transform, (With<Planet>, Without<Player>)>,
  mut players_q: Query<&mut Transform, (With<Player>, Without<Planet>)>,
) {
  for planet_transform in &planets_q {
    for mut player_transform in &mut players_q {
      let a = (planet_transform.translation - player_transform.translation).normalize() * 0.05;
      player_transform.translation += a;
    }
  }
}

fn attach_to_cursor(
  windows_query: Query<&Window>,
  camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
  mut query: Query<(&mut Transform, &Name), With<AttachedToCursor>>,
) {
  // get the camera info and transform
  // assuming there is exactly one main camera entity, so query::single() is OK
  let (camera, camera_transform) = camera_q.single();
  let window = windows_query.single();

  // check if the cursor is inside the window and get its position
  // then, ask bevy to convert into world coordinates, and truncate to discard Z
  if let Some(world_position) = window
    .cursor_position()
    .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
    .map(|ray| ray.origin.truncate())
  {
    for (mut transform, ..) in &mut query {
      transform.translation.x += world_position.x;
      transform.translation.y += world_position.y;
    }
  }
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup)
    .add_system(move_entities)
    .add_system(enable_direction_lines)
    .add_system(disable_direction_lines)
    .add_system(attract_to_planets)
    .add_system(align_to_planets)
    .add_system(toggle_direction_lines)
    .add_system(player_controls)
    .add_system(player_mouse_controls)
    .add_system(attach_to_cursor.after(move_entities))
    .run();
}

// use std::f32::consts::PI;
//
// use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
//
// fn main() {
//     App::new()
//         .add_plugins(DefaultPlugins)
//         .add_startup_system(setup)
//         .add_system(movement)
//         .add_system(animate_light_direction)
//         .run();
// }
//
// #[derive(Component)]
// struct Movable;
//
// /// set up a simple 3D scene
// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     asset_server: Res<AssetServer>,
// ) {
//     // ground plane
//     commands.spawn(PbrBundle {
//         mesh: meshes.add(shape::Plane::from_size(10.0).into()),
//         material: materials.add(StandardMaterial {
//             base_color: Color::WHITE,
//             perceptual_roughness: 1.0,
//             ..default()
//         }),
//         ..default()
//     });
//
//     // left wall
//     let mut transform = Transform::from_xyz(2.5, 2.5, 0.0);
//     transform.rotate_z(PI / 2.);
//     commands.spawn(PbrBundle {
//         mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 0.15, 5.0))),
//         transform,
//         material: materials.add(StandardMaterial {
//             base_color: Color::INDIGO,
//             perceptual_roughness: 1.0,
//             ..default()
//         }),
//         ..default()
//     });
//     // back (right) wall
//     let mut transform = Transform::from_xyz(0.0, 2.5, -2.5);
//     transform.rotate_x(PI / 2.);
//     commands.spawn(PbrBundle {
//         mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 0.15, 5.0))),
//         transform,
//         material: materials.add(StandardMaterial {
//             base_color: Color::INDIGO,
//             perceptual_roughness: 1.0,
//             ..default()
//         }),
//         ..default()
//     });
//
//     // Bevy logo to demonstrate alpha mask shadows
//     let mut transform = Transform::from_xyz(-2.2, 0.5, 1.0);
//     transform.rotate_y(PI / 8.);
//     commands.spawn((
//         PbrBundle {
//             mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(2.0, 0.5)))),
//             transform,
//             material: materials.add(StandardMaterial {
//                 base_color_texture: Some(asset_server.load("branding/bevy_logo_light.png")),
//                 perceptual_roughness: 1.0,
//                 alpha_mode: AlphaMode::Mask(0.5),
//                 cull_mode: None,
//                 ..default()
//             }),
//             ..default()
//         },
//         Movable,
//     ));
//
//     // cube
//     commands.spawn((
//         PbrBundle {
//             mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//             material: materials.add(StandardMaterial {
//                 base_color: Color::PINK,
//                 ..default()
//             }),
//             transform: Transform::from_xyz(0.0, 0.5, 0.0),
//             ..default()
//         },
//         Movable,
//     ));
//     // sphere
//     commands.spawn((
//         PbrBundle {
//             mesh: meshes.add(Mesh::from(shape::UVSphere {
//                 radius: 0.5,
//                 ..default()
//             })),
//             material: materials.add(StandardMaterial {
//                 base_color: Color::LIME_GREEN,
//                 ..default()
//             }),
//             transform: Transform::from_xyz(1.5, 1.0, 1.5),
//             ..default()
//         },
//         Movable,
//     ));
//
//     // ambient light
//     commands.insert_resource(AmbientLight {
//         color: Color::ORANGE_RED,
//         brightness: 0.02,
//     });
//
//     // red point light
//     commands
//         .spawn(PointLightBundle {
//             // transform: Transform::from_xyz(5.0, 8.0, 2.0),
//             transform: Transform::from_xyz(1.0, 2.0, 0.0),
//             point_light: PointLight {
//                 intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
//                 color: Color::RED,
//                 shadows_enabled: true,
//                 ..default()
//             },
//             ..default()
//         })
//         .with_children(|builder| {
//             builder.spawn(PbrBundle {
//                 mesh: meshes.add(Mesh::from(shape::UVSphere {
//                     radius: 0.1,
//                     ..default()
//                 })),
//                 material: materials.add(StandardMaterial {
//                     base_color: Color::RED,
//                     emissive: Color::rgba_linear(7.13, 0.0, 0.0, 0.0),
//                     ..default()
//                 }),
//                 ..default()
//             });
//         });
//
//     // green spot light
//     commands
//         .spawn(SpotLightBundle {
//             transform: Transform::from_xyz(-1.0, 2.0, 0.0)
//                 .looking_at(Vec3::new(-1.0, 0.0, 0.0), Vec3::Z),
//             spot_light: SpotLight {
//                 intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
//                 color: Color::GREEN,
//                 shadows_enabled: true,
//                 inner_angle: 0.6,
//                 outer_angle: 0.8,
//                 ..default()
//             },
//             ..default()
//         })
//         .with_children(|builder| {
//             builder.spawn(PbrBundle {
//                 transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.0)),
//                 mesh: meshes.add(Mesh::from(shape::Capsule {
//                     depth: 0.125,
//                     radius: 0.1,
//                     ..default()
//                 })),
//                 material: materials.add(StandardMaterial {
//                     base_color: Color::GREEN,
//                     emissive: Color::rgba_linear(0.0, 7.13, 0.0, 0.0),
//                     ..default()
//                 }),
//                 ..default()
//             });
//         });
//
//     // blue point light
//     commands
//         .spawn(PointLightBundle {
//             // transform: Transform::from_xyz(5.0, 8.0, 2.0),
//             transform: Transform::from_xyz(0.0, 4.0, 0.0),
//             point_light: PointLight {
//                 intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
//                 color: Color::BLUE,
//                 shadows_enabled: true,
//                 ..default()
//             },
//             ..default()
//         })
//         .with_children(|builder| {
//             builder.spawn(PbrBundle {
//                 mesh: meshes.add(Mesh::from(shape::UVSphere {
//                     radius: 0.1,
//                     ..default()
//                 })),
//                 material: materials.add(StandardMaterial {
//                     base_color: Color::BLUE,
//                     emissive: Color::rgba_linear(0.0, 0.0, 7.13, 0.0),
//                     ..default()
//                 }),
//                 ..default()
//             });
//         });
//
//     // directional 'sun' light
//     commands.spawn(DirectionalLightBundle {
//         directional_light: DirectionalLight {
//             shadows_enabled: true,
//             ..default()
//         },
//         transform: Transform {
//             translation: Vec3::new(0.0, 2.0, 0.0),
//             rotation: Quat::from_rotation_x(-PI / 4.),
//             ..default()
//         },
//         // The default cascade config is designed to handle large scenes.
//         // As this example has a much smaller world, we can tighten the shadow
//         // bounds for better visual quality.
//         cascade_shadow_config: CascadeShadowConfigBuilder {
//             first_cascade_far_bound: 4.0,
//             maximum_distance: 10.0,
//             ..default()
//         }
//         .into(),
//         ..default()
//     });
//
//     // camera
//     commands.spawn(Camera3dBundle {
//         transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
//         ..default()
//     });
// }
//
// fn animate_light_direction(
//     time: Res<Time>,
//     mut query: Query<&mut Transform, With<DirectionalLight>>,
// ) {
//     for mut transform in &mut query {
//         transform.rotate_y(time.delta_seconds() * 0.5);
//     }
// }
//
// fn movement(
//     input: Res<Input<KeyCode>>,
//     time: Res<Time>,
//     mut query: Query<&mut Transform, With<Movable>>,
// ) {
//     for mut transform in &mut query {
//         let mut direction = Vec3::ZERO;
//         if input.pressed(KeyCode::Up) {
//             direction.y += 1.0;
//         }
//         if input.pressed(KeyCode::Down) {
//             direction.y -= 1.0;
//         }
//         if input.pressed(KeyCode::Left) {
//             direction.x -= 1.0;
//         }
//         if input.pressed(KeyCode::Right) {
//             direction.x += 1.0;
//         }
//
//         transform.translation += time.delta_seconds() * 2.0 * direction;
//     }
// }
//
