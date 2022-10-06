use bevy::prelude::*;
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use bevy_dice::{DicePlugin, DiceRollResult, DiceRollStartEvent};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(DicePlugin)
        .add_plugin(OverlayPlugin {
            font_size: 32.0,
            ..default()
        })
        .add_startup_system(setup_button)
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
        screen_print!(col: Color::CYAN, "Dice roll result: {0}", event.value);
    }
}
