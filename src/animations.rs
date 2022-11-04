// Credits to https://github.com/asafigan/bevy_jam_2/blob/814e7da7b3fe74a29e3984a435dbae524366a87c/src/battle.rs#L223

use bevy::{asset::HandleId, gltf::Gltf, prelude::*};

use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount, EnumIter, EnumVariantNames};

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Loading>();
        app.add_startup_system(load_model);
        app.add_system(build_unit_animators);
        app.add_system(play_idle_animation);
        app.add_system(find_enemy_animations);
        app.add_startup_system(load_unit_models);
        //app.add_system(play_anim);
    }
}

#[derive(Component)]
struct UnitAnimations {
    pub run: Handle<AnimationClip>,
}

#[derive(Component)]
struct UnitAnimator {
    animation_player: Entity,
    current_animation: Option<Handle<AnimationClip>>,
}

#[derive(Default)]
pub struct Loading {
    pub assets: Vec<HandleUntyped>,
}
fn load_unit_models(asset_server: Res<AssetServer>, mut loading: ResMut<Loading>) {
    let models: Vec<_> = UnitKind::gltf_paths()
        .into_iter()
        .map(|path| asset_server.load_untyped(&path))
        .collect();
    loading.assets.extend(models);
}

fn load_model(mut commands: Commands, asset_server: Res<AssetServer>, gltfs: Res<Assets<Gltf>>) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ..default()
    });
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
    commands
        .spawn_bundle(SceneBundle {
            scene: UnitKind::Mouse.scene_handle(),
            ..default()
        })
        .insert(UnitKind::Mouse);
}
fn find_enemy_animations(
    enemies: Query<(Entity, &UnitKind), Without<UnitAnimations>>,
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
) {
    for (entity, enemy) in &enemies {
        if let Some(gltf) = gltfs.get(&enemy.gltf_handle()) {
            commands.entity(entity).insert(UnitAnimations {
                run: gltf.animations[0].clone(),
            });
        }
    }
}
fn build_unit_animators(
    mut commands: Commands,
    units: Query<Entity, (With<UnitAnimations>, Without<UnitAnimator>)>,
    children: Query<&Children>,
    animations: Query<&AnimationPlayer>,
) {
    fn find_animation_player(
        entity: Entity,
        children: &Query<&Children>,
        animations: &Query<&AnimationPlayer>,
    ) -> Option<Entity> {
        if animations.contains(entity) {
            return Some(entity);
        }

        children
            .get(entity)
            .into_iter()
            .flatten()
            .cloned()
            .find_map(|e| find_animation_player(e, children, animations))
    }

    for entity in &units {
        dbg!("New animator?");

        if let Some(animation_player) = find_animation_player(entity, &children, &animations) {
            dbg!("New animator!");
            commands.entity(entity).insert(UnitAnimator {
                animation_player,
                current_animation: None,
            });
        }
    }
}
fn play_idle_animation(
    mut enemies: Query<(&UnitAnimations, &mut UnitAnimator)>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<Assets<AnimationClip>>,
) {
    for (enemy_animations, mut animator) in &mut enemies {
        let mut animation_player = animation_players
            .get_mut(animator.animation_player)
            .unwrap();
        dbg!("will play idle?");

        // The default animation player is playing by default and never stops even though there is no animation clip.
        // The animation's elapsed time is very unlikely to be a 0.0 unless there is no animation clip.
        // Therefore, it is assumed at if elapsed time in 0.0 there in no animation playing.
        // What is needed on bevy side is a getter to the animation player's animation clip handle
        // so we can see if it is the default handle (no animation clip).
        let no_animation = !animation_player.is_changed() && animation_player.elapsed() == 0.0;

        let current_animation = animator
            .current_animation
            .as_ref()
            .and_then(|x| animations.get(x));

        // There is no way to check if animation player is looping?
        let animation_ended = current_animation
            .map(|x| animation_player.elapsed() > x.duration())
            .unwrap_or_default();

        if (no_animation || animation_ended)
            && (animator.current_animation.as_ref() != Some(&enemy_animations.run))
        {
            dbg!("playing");
            animator.current_animation = Some(enemy_animations.run.clone());
            animation_player.play(enemy_animations.run.clone()).repeat();
        }
    }
}
#[derive(Clone, Copy, EnumCount, Display, EnumVariantNames, EnumIter, Component)]
pub enum UnitKind {
    Mouse,
}

impl UnitKind {
    pub fn gltf_paths() -> Vec<String> {
        Self::iter().map(|x| x.gltf_path()).collect()
    }

    pub fn scene_handle(&self) -> Handle<Scene> {
        let path = format!("units/{self}.gltf#Scene0");

        Handle::weak(HandleId::AssetPathId(path.as_str().into()))
    }

    pub fn gltf_path(&self) -> String {
        format!("units/{self}.gltf")
    }

    pub fn gltf_handle(&self) -> Handle<Gltf> {
        let path = self.gltf_path();

        Handle::weak(HandleId::AssetPathId(path.as_str().into()))
    }
}
