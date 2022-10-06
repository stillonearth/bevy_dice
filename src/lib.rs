use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};
use bevy_rapier3d::prelude::*;
use rand::Rng;

const PLANE_SIZE: (f32, f32) = (64.0, 64.0);

pub struct DicePlugin;

pub struct DicePluginSettings {
    pub num_dice: usize,
    pub render_size: (u32, u32),
    pub render_handle: Option<Handle<Image>>,
}

impl Plugin for DicePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_scene.label("dice_plugin_init"))
            .add_event::<DiceRollEndEvent>()
            .add_event::<DiceRollStartEvent>()
            .add_event::<DiceRollResult>()
            .add_system(event_collisions)
            .add_system(event_stop_dice_rolls)
            .add_system(event_start_dice_roll)
            .insert_resource(DiceRollResult::default());
    }
}

#[derive(Component)]
pub struct DiceCamera;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut plugin_settings: ResMut<DicePluginSettings>,
) {
    // Render target
    let size = Extent3d {
        width: plugin_settings.render_size.0,
        height: plugin_settings.render_size.1,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };
    image.resize(size);
    let image_handle = images.add(image);
    plugin_settings.render_handle = Some(image_handle.clone());

    // Spawn camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 3.0, 1.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            camera: Camera {
                priority: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        })
        .insert(DiceCamera)
        .insert(UiCameraConfig { show_ui: false });

    // Spawn light
    const HALF_SIZE: f32 = 1.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // Spawn ground
    let mesh = meshes.add(Mesh::from(shape::Plane {
        size: PLANE_SIZE.0 * PLANE_SIZE.1,
    }));

    let material_handle = materials.add(Color::GREEN.into());

    commands
        .spawn_bundle(PbrBundle {
            mesh: mesh.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            material: material_handle.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(PLANE_SIZE.0, 1.0, PLANE_SIZE.1))
        .insert(ActiveEvents::CONTACT_FORCE_EVENTS)
        // .insert(Sensor(true))
        .insert(Name::new("Ground"));
}

#[derive(Component)]
struct Dice;

fn event_start_dice_roll(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut events: EventReader<DiceRollStartEvent>,
    q_dice: Query<(Entity, &Dice)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dice_settings: Res<DicePluginSettings>,
    mut previous_roll: ResMut<DiceRollResult>,
) {
    if events.iter().len() == 0 {
        return;
    }

    // Remove previous dice
    previous_roll.value = Vec::new();

    // clear previously spawned dice
    q_dice.iter().for_each(|(entity, _)| {
        commands.entity(entity).despawn_recursive();
    });

    let scene_handle = asset_server.load("models/dice/scene.gltf#Scene0");
    let transparent_material_handle = materials.add(Color::rgba(0., 0., 0., 0.).into());

    let mut rng = rand::thread_rng();

    for _ in 0..dice_settings.num_dice {
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            rng.gen_range(0.0..std::f32::consts::PI * 2.0),
            rng.gen_range(0.0..std::f32::consts::PI * 2.0),
            rng.gen_range(0.0..std::f32::consts::PI * 2.0),
        );
        let transform =
            Transform::from_xyz(0., rng.gen_range(2.0..5.0), 0.).with_rotation(rotation);

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                material: transparent_material_handle.clone(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(SceneBundle {
                    scene: scene_handle.clone(),
                    transform: Transform::from_xyz(0., 0.0, 0.).with_scale(Vec3::splat(0.1)),
                    ..default()
                });
            })
            .insert(transform)
            .insert(Name::new("Dice"))
            .insert(RigidBody::Dynamic)
            .insert(Collider::cuboid(0.04, 0.04, 0.04))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Dice);
    }
}

fn event_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
    mut ev_dice_stopped: EventWriter<DiceRollEndEvent>,
    q_dice: Query<(Entity, &Dice)>,
) {
    let is_dice = |entity: &Entity| q_dice.get(*entity).is_ok();

    for event in contact_force_events.iter() {
        if is_dice(&event.collider1) {
            ev_dice_stopped.send(DiceRollEndEvent(event.collider1));
        }

        if is_dice(&event.collider2) {
            ev_dice_stopped.send(DiceRollEndEvent(event.collider2));
        }
    }

    for event in collision_events.iter() {
        match event {
            CollisionEvent::Started(entity_1, entity_2, _) => {
                if is_dice(entity_1) {
                    ev_dice_stopped.send(DiceRollEndEvent(*entity_1));
                }

                if is_dice(entity_2) {
                    ev_dice_stopped.send(DiceRollEndEvent(*entity_2));
                }
            }
            CollisionEvent::Stopped(entity_1, entity_2, _) => {
                if is_dice(entity_1) {
                    ev_dice_stopped.send(DiceRollEndEvent(*entity_1));
                }

                if is_dice(entity_2) {
                    ev_dice_stopped.send(DiceRollEndEvent(*entity_2));
                }
            }
        }
    }
}

struct DiceRollEndEvent(Entity);
pub struct DiceRollStartEvent(pub Entity);

const CUBE_SIDES: [Vec3; 6] = [
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 0.0, -1.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(-1.0, 0.0, 0.0),
    Vec3::new(0.0, -1.0, 0.0),
];

#[derive(Default, Clone)]
pub struct DiceRollResult {
    pub value: Vec<usize>,
}

fn event_stop_dice_rolls(
    mut event_reader: EventReader<DiceRollEndEvent>,
    mut event_writer: EventWriter<DiceRollResult>,
    query: Query<(Entity, &Transform, &Dice)>,
    mut previous_roll: ResMut<DiceRollResult>,
) {
    for _ in event_reader.iter() {
        let mut result = DiceRollResult { ..default() };

        for (_dice, (_, transform, _)) in query.iter().enumerate() {
            let mut height = 0.0;
            let mut value = 0;
            for (i, side) in CUBE_SIDES.iter().enumerate() {
                let y = transform.rotation.mul_vec3(*side)[1];
                if height < y {
                    height = y;
                    value = i + 1;
                }
            }

            result.value.push(value);
        }

        if previous_roll.value != result.value {
            previous_roll.value = result.value.clone();
            event_writer.send(result.clone());
        }
    }
}
