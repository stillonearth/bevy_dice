use bevy::prelude::*;
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use bevy_rapier3d::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(OverlayPlugin {
            font_size: 32.0,
            ..default()
        })
        .add_event::<DiceRollEndEvent>()
        .add_event::<DiceRollStartEvent>()
        .add_startup_system(setup_scene)
        .add_startup_system(setup_button)
        .add_system(event_collisions)
        .add_system(event_stop_dice_rolls)
        .add_system(event_start_dice_roll)
        .add_system(button_system)
        .run();
}

const PLANE_SIZE: (f32, f32) = (64.0, 64.0);

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 3.0, 1.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
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
        .insert(ActiveEvents::COLLISION_EVENTS)
        // .insert(Sensor(true))
        .insert(Name::new("Ground"));
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn button_system(
    mut interaction_query: Query<
        (Entity, &Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_dice_started: EventWriter<DiceRollStartEvent>,
) {
    for (entity, interaction, mut color, _) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                ev_dice_started.send(DiceRollStartEvent(entity));
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn setup_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    // commands.spawn_bundle(Camera2dBundle::default());
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Roll Dice",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });
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
) {
    if events.iter().len() == 0 {
        return;
    }

    // clear previously spawned dice
    q_dice.iter().for_each(|(entity, _)| {
        commands.entity(entity).despawn_recursive();
    });

    let scene_handle = asset_server.load("models/dice/scene.gltf#Scene0");
    let mut rng = rand::thread_rng();
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        rng.gen_range(0.0..std::f32::consts::PI * 2.0),
        rng.gen_range(0.0..std::f32::consts::PI * 2.0),
        rng.gen_range(0.0..std::f32::consts::PI * 2.0),
    );
    let transform = Transform::from_xyz(0., rng.gen_range(1.0..3.0), 0.).with_rotation(rotation);
    let transparent_material_handle = materials.add(Color::rgba(0., 0., 0., 0.).into());

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            material: transparent_material_handle,
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

fn event_collisions(
    mut events: EventReader<CollisionEvent>,
    mut ev_dice_stopped: EventWriter<DiceRollEndEvent>,
    q_dice: Query<(Entity, &Dice)>,
) {
    let is_dice = |entity: &Entity| q_dice.get(*entity).is_ok();

    for event in events.iter() {
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
struct DiceRollStartEvent(Entity);

const CUBE_SIDES: [Vec3; 6] = [
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 0.0, -1.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(-1.0, 0.0, 0.0),
    Vec3::new(0.0, -1.0, 0.0),
];

fn event_stop_dice_rolls(
    mut events: EventReader<DiceRollEndEvent>,
    query: Query<(Entity, &Transform, &Dice)>,
) {
    for _ in events.iter() {
        for (dice, (_, transform, _)) in query.iter().enumerate() {
            let mut height = 0.0;
            let mut value = 0;
            for (i, side) in CUBE_SIDES.iter().enumerate() {
                let y = transform.rotation.mul_vec3(*side)[1];
                if height < y {
                    height = y;
                    value = i + 1;
                }
            }

            screen_print!(col: Color::CYAN, "dice: {dice} value: {value}");
        }
    }
}
