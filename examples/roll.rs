use bevy::prelude::*;
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use bevy_dice::{DicePlugin, DicePluginSettings, DiceRollResult, DiceRollStartEvent};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(OverlayPlugin {
            font_size: 32.0,
            ..default()
        })
        .add_plugin(DicePlugin)
        .insert_resource(DicePluginSettings {
            render_size: (640, 720),
            number_of_fields: 2,
            ..default()
        })
        .add_startup_system(setup.after("dice_plugin_init"))
        .add_system(button_system)
        .add_system(display_roll_result)
        .run();
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
    for (_entity, interaction, mut color, _) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();

                let mut num_dice: Vec<usize> = Vec::new();
                num_dice.push(2);
                num_dice.push(2);

                ev_dice_started.send(DiceRollStartEvent { num_dice });
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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dice_plugin_settings: Res<DicePluginSettings>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    for render_handle in dice_plugin_settings.render_handles.iter() {
        commands.spawn_bundle(ImageBundle {
            image: UiImage(render_handle.clone()),
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..default()
            },
            ..default()
        });
    }

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
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

fn display_roll_result(mut dice_rolls: EventReader<DiceRollResult>) {
    for event in dice_rolls.iter() {
        screen_print!(
            col: Color::CYAN,
            "Dice 1 roll result: {0}, {1}\nDice 2 roll result: {2}, {3}",
            event.values[0][0],
            event.values[0][1],
            event.values[1][0],
            event.values[1][1]
        );
    }
}
