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
struct Planet {
  pub radius: f32,
}

#[derive(Component, Debug)]
struct AlignPlanet;

#[derive(Component, Debug)]
struct Player;

#[derive(Component, Debug)]
struct ShowDirectionLines;

#[derive(Component, Debug)]
struct ShowDirectionLinesApplied;

#[derive(Component, Debug)]
struct DirectionLine;

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
struct PanOrbitCamera {
  /// The "focus point" to orbit around. It is automatically updated when panning the camera
  pub focus: Vec3,
  pub radius: f32,
  pub upside_down: bool,
}

impl Default for PanOrbitCamera {
  fn default() -> Self {
    PanOrbitCamera {
      focus: Vec3::ZERO,
      radius: 5.0,
      upside_down: false,
    }
  }
}

const PLAYER_HEIGHT: f32 = 0.4;
const PLAYER_WALKING_SPEED: f32 = 30.0;
const PLAYER_TURN_SPEED: f32 = 2.0;
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
const SUN_INTENSITY: f32 = 10.0;
const SUN_SOFTNESS: f32 = 10.0;
const SUN_DISTANCE: f32 = PLANET_RADIUS * 4.0;
const SUN_LOCATION: Transform = Transform::from_xyz(0.0, SUN_DISTANCE, 0.0);
const SUN_COLOR: Color = Color::rgb(255.0, 248.0, 224.0);
const SUN_SHADOW_DEPTH_BIAS: f32 = 2.00;
const SUN_SHADOW_NORMAL_BIAS: f32 = 2.00;

macro_rules! extend_commands {
  ($command_name:ident($( $arg:ident: $arg_type:ty ),*), $command_fn:expr) => {
    #[allow(non_camel_case_types)]
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

  const SMALL_PLANET_SIZE: f32 = 3.0;
  commands
    .spawn_circle(
      Vec3::new(PLANET_RADIUS * 1.0 + 5.0, 0.0, 0.0),
      SMALL_PLANET_SIZE,
      Color::rgb(1.0, 0.0, 0.0),
    )
    .insert(Planet {
      radius: SMALL_PLANET_SIZE,
    });
  commands
    .spawn_circle(
      Vec3::new(0.0, PLANET_RADIUS * 1.0 + 5.0, 0.0),
      SMALL_PLANET_SIZE,
      Color::rgb(1.0, 1.0, 0.0),
    )
    .insert(Planet {
      radius: SMALL_PLANET_SIZE,
    });
  commands
    .spawn_circle(
      Vec3::new(0.0, 0.0, PLANET_RADIUS * 1.0 + 5.0),
      SMALL_PLANET_SIZE,
      Color::rgb(0.0, 1.0, 0.0),
    )
    .insert(Planet {
      radius: SMALL_PLANET_SIZE,
    });
  // sun (sphere)
  commands
    .spawn_circle(SUN_LOCATION.translation, SUN_RADIUS, SUN_COLOR)
    .insert(Planet { radius: SUN_RADIUS });
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
  let planet_mesh = {
    let mut mesh = Mesh::from(shape::UVSphere {
      radius: PLANET_RADIUS,
      // TODO: on PLANET_RADIUS value of 30.0 the circle becomes a triangle,
      // play with that.
      sectors: (PLANET_RADIUS * 8.0).max(PLANET_MIN_QUALITY) as usize,
      stacks: (PLANET_RADIUS * 8.0).max(PLANET_MIN_QUALITY) as usize,
    });
    mesh.generate_tangents();
    mesh
  };

  commands
    .spawn(PbrBundle {
      mesh: meshes.add(planet_mesh),
      material: materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("earth-4k.jpg")),
        normal_map_texture: Some(asset_server.load("earth-normal-4k.png")),
        flip_normal_map_y: true,
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
    .insert(Planet {
      radius: PLANET_RADIUS,
    })
    .insert(AlignPlanet);
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
      let translation = Vec3::new(0.0, CAMERA_INITIAL_DISTANCE, CAMERA_INITIAL_DISTANCE / 2.5);
      child_commands
        .spawn(Camera3dBundle {
          transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
          ..default()
        })
        .insert(MainCamera)
        .insert(PanOrbitCamera {
          radius: translation.length(),
          ..default()
        });
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

    let rotation = PLAYER_TURN_SPEED
      * (f32::from(keys.pressed(KeyCode::Q)) + -f32::from(keys.pressed(KeyCode::E)));
    player_transform.rotate_local_y(rotation * time.delta_seconds());
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
  planets_q: Query<&Transform, (With<AlignPlanet>, Without<Player>)>,
  mut players_q: Query<&mut Transform, (With<Player>, Without<AlignPlanet>)>,
) {
  for planet_transform in &planets_q {
    for mut player_transform in &mut players_q {
      let planet_to_player = player_transform.translation - planet_transform.translation;
      let player_to_planet = planet_transform.translation - player_transform.translation;
      let f0 = player_transform.forward();
      let a = player_to_planet;
      let c = f0.cross(a);
      let f1 = a.cross(c);
      player_transform.look_to(f1, planet_to_player);
    }
  }
}

fn collide_with_planets(
  planets_q: Query<(&Transform, &Planet), (With<Planet>, Without<Player>)>,
  mut players_q: Query<&mut Transform, (With<Player>, Without<Planet>)>,
) {
  for (planet_transform, planet) in &planets_q {
    for mut player_transform in &mut players_q {
      let player_to_planet = (planet_transform.translation - player_transform.translation);
      let delta_length = player_to_planet.length() - (planet.radius + PLAYER_HEIGHT / 2.0);

      if delta_length < 0.0 {
        player_transform.translation += (player_to_planet).normalize() * delta_length;
      }
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

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
// fn pan_orbit_camera(
//     windows: Res<Windows>,
//     mut ev_motion: EventReader<MouseMotion>,
//     mut ev_scroll: EventReader<MouseWheel>,
//     input_mouse: Res<Input<MouseButton>>,
//     mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
// ) {
//     // change input mapping for orbit and panning here
//     let orbit_button = MouseButton::Right;
//     let pan_button = MouseButton::Middle;
//
//     let mut pan = Vec2::ZERO;
//     let mut rotation_move = Vec2::ZERO;
//     let mut scroll = 0.0;
//     let mut orbit_button_changed = false;
//
//     if input_mouse.pressed(orbit_button) {
//         for ev in ev_motion.iter() {
//             rotation_move += ev.delta;
//         }
//     } else if input_mouse.pressed(pan_button) {
//         // Pan only if we're not rotating at the moment
//         for ev in ev_motion.iter() {
//             pan += ev.delta;
//         }
//     }
//     for ev in ev_scroll.iter() {
//         scroll += ev.y;
//     }
//     if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
//         orbit_button_changed = true;
//     }
//
//     for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
//         if orbit_button_changed {
//             // only check for upside down when orbiting started or ended this frame
//             // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
//             let up = transform.rotation * Vec3::Y;
//             pan_orbit.upside_down = up.y <= 0.0;
//         }
//
//         let mut any = false;
//         if rotation_move.length_squared() > 0.0 {
//             any = true;
//             let window = get_primary_window_size(&windows);
//             let delta_x = {
//                 let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
//                 if pan_orbit.upside_down { -delta } else { delta }
//             };
//             let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
//             let yaw = Quat::from_rotation_y(-delta_x);
//             let pitch = Quat::from_rotation_x(-delta_y);
//             transform.rotation = yaw * transform.rotation; // rotate around global y axis
//             transform.rotation = transform.rotation * pitch; // rotate around local x axis
//         } else if pan.length_squared() > 0.0 {
//             any = true;
//             // make panning distance independent of resolution and FOV,
//             let window = get_primary_window_size(&windows);
//             if let Projection::Perspective(projection) = projection {
//                 pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
//             }
//             // translate by local axes
//             let right = transform.rotation * Vec3::X * -pan.x;
//             let up = transform.rotation * Vec3::Y * pan.y;
//             // make panning proportional to distance away from focus point
//             let translation = (right + up) * pan_orbit.radius;
//             pan_orbit.focus += translation;
//         } else if scroll.abs() > 0.0 {
//             any = true;
//             pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
//             // dont allow zoom to reach zero or you get stuck
//             pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
//         }
//
//         if any {
//             // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
//             // parent = x and y rotation
//             // child = z-offset
//             let rot_matrix = Mat3::from_quat(transform.rotation);
//             transform.translation = pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
//         }
//     }
//
//     // consume any remaining events, so they don't pile up if we don't need them
//     // (and also to avoid Bevy warning us about not checking events every frame update)
//     ev_motion.clear();
// }

// fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
//     let window = windows.get_primary().unwrap();
//     let window = Vec2::new(window.width() as f32, window.height() as f32);
//     window
// }

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
    .add_system(collide_with_planets.after(player_controls))
    .add_system(player_mouse_controls)
    // .add_system(pan_orbit_camera)
    .add_system(attach_to_cursor.after(move_entities))
    .run();
}
