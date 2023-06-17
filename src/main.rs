use bevy::{
  ecs::system::{EntityCommands},
  prelude::*,
  sprite::MaterialMesh2dBundle,
};

const SIZE: f32 = 10.0;

#[derive(Component, Debug)]
struct AttachedToCursor;

#[derive(Component, Debug)]
struct MainCamera;

pub(crate) trait CircleCommandsExt<'w, 's> {
  fn spawn_circle<'a>(
    &'a mut self,
    position: Vec3,
    radius: f32,
    color: Color,
  ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> CircleCommandsExt<'w, 's> for Commands<'w, 's> {
  fn spawn_circle<'a>(
    &'a mut self,
    position: Vec3,
    radius: f32,
    color: Color,
  ) -> EntityCommands<'w, 's, 'a> {
    let entity = self.spawn_empty();
    let entity_id = entity.id();

    self.add(move |world: &mut World| {
      let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(shape::Circle::new(radius).into());
      let material = world
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial::from(color));

      world.entity_mut(entity_id).insert(MaterialMesh2dBundle {
        mesh: mesh.into(),
        material,
        transform: Transform::from_translation(position),
        ..default()
      });
    });

    self.entity(entity_id)
  }
}

fn setup(mut commands: Commands) {
  commands.insert_resource(ClearColor(Color::BLACK));
  commands.spawn(Camera2dBundle::default()).insert(MainCamera);
  commands
    .spawn_circle(Vec3::new(0., 0., 0.01), SIZE, Color::GRAY)
    .insert(Name::new("eblan"))
    .insert(AttachedToCursor);
  commands
    .spawn_circle(Vec3::new(0., 0., 0.02), SIZE, Color::CRIMSON)
    .insert(Name::new("eblan"));
  commands
    .spawn_circle(Vec3::new(0., 0., 0.03), SIZE, Color::SEA_GREEN)
    .insert(Name::new("eblan"));
}

fn move_entities(mut query: Query<(&mut Transform, &Name)>, time: Res<Time>) {
  const RADIUS: f32 = 30.0;

  for (mut transform, ..) in &mut query {
    transform.translation.x = RADIUS * time.elapsed_seconds().cos();
    transform.translation.y = RADIUS * time.elapsed_seconds().sin();
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
    .add_system(attach_to_cursor.after(move_entities))
    .run();
}
