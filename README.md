# bevy_dice

Physics-based dice rolls for bevy. This is a work in progress. You can you this plugin to build tabletop games.

https://user-images.githubusercontent.com/97428129/194198974-239a14d2-0056-45c0-982f-06f9604cc5d1.mp4

Dice rolls are performed in a physical space, rendered to a image handle which you can display in your UI when dice rolls are needed.

Dice roll results are emmited as events, so you can listen for them and perform actions based on the result.

## Dependencies

| Name          | Version |
| ------------- | ------- |
| bevy          | 0.8.1   |
| bevy_rapier3d | 0.17.0  |

````

## Usage

```rust
use bevy::prelude::*;
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use bevy_dice::{DicePlugin, DicePluginSettings, DiceRollResult, DiceRollStartEvent};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(OverlayPlugin {
            font_size: 32.0,
            ..default()
        })
        .add_plugin(DicePlugin)
        .insert_resource(DicePluginSettings {
            num_dice: 1,
            render_size: (512, 512),
            render_handle: None,
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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dice_plugin_settings: Res<DicePluginSettings>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands.spawn_bundle(SpriteBundle {
        texture: dice_plugin_settings.render_handle.clone().unwrap(),
        ..default()
    });

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
        screen_print!(col: Color::CYAN, "Dice roll result: {0}", event.value[0]);
    }
}

````

## Dice 3D Model

Model Information:

- title: Dice
- source: https://sketchfab.com/3d-models/dice-3b955af797e140eca0947ede57f412ba
- author: tnRaro (https://sketchfab.com/tnRaro)

Model License:

- license type: CC-BY-4.0 (http://creativecommons.org/licenses/by/4.0/)
- requirements: Author must be credited. Commercial use is allowed.

If you use this 3D model in your project be sure to copy paste this credit wherever you share it:
This work is based on "Dice" (https://sketchfab.com/3d-models/dice-3b955af797e140eca0947ede57f412ba) by tnRaro (https://sketchfab.com/tnRaro) licensed under CC-BY-4.0 (http://creativecommons.org/licenses/by/4.0/)
