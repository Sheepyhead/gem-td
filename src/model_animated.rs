// Credits to https://github.com/asafigan/bevy_jam_2/blob/814e7da7b3fe74a29e3984a435dbae524366a87c/src/battle.rs#L223

use bevy::{asset::HandleId, gltf::Gltf, prelude::*};

use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount, EnumIter, EnumVariantNames};

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Loading>();
        app.add_system(build_unit_animators);
        app.add_system(play_idle_animation);
        app.add_system(find_animations);
        app.add_startup_system(load_unit_models);
        //app.add_system(play_anim);
    }
}

#[derive(Component)]
struct AnimationClipsList {
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
    let models: Vec<_> = ModelKind::gltf_paths()
        .into_iter()
        .map(|path| asset_server.load_untyped(&path))
        .collect();
    loading.assets.extend(models);
}

fn find_animations(
    model_kinds: Query<(Entity, &ModelKind), Without<AnimationClipsList>>,
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
) {
    for (entity, anim) in &model_kinds {
        if let Some(gltf) = gltfs.get(&anim.gltf_handle()) {
            commands.entity(entity).insert(AnimationClipsList {
                run: gltf.animations[0].clone(),
            });
        }
    }
}
fn build_unit_animators(
    mut commands: Commands,
    animator_pending: Query<Entity, (With<AnimationClipsList>, Without<UnitAnimator>)>,
    children: Query<&Children>,
    animation_players: Query<&AnimationPlayer>,
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

    for entity in &animator_pending {
        if let Some(animation_player) = find_animation_player(entity, &children, &animation_players)
        {
            commands.entity(entity).insert(UnitAnimator {
                animation_player,
                current_animation: None,
            });
        }
    }
}
fn play_idle_animation(
    mut animators: Query<(&AnimationClipsList, &mut UnitAnimator)>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animation_asset: Res<Assets<AnimationClip>>,
) {
    for (enemy_animations, mut animator) in &mut animators {
        let mut animation_player = animation_players
            .get_mut(animator.animation_player)
            .unwrap();
        // The default animation player is playing by default and never stops even though there is no animation clip.
        // The animation's elapsed time is very unlikely to be a 0.0 unless there is no animation clip.
        // Therefore, it is assumed at if elapsed time in 0.0 there in no animation playing.
        // What is needed on bevy side is a getter to the animation player's animation clip handle
        // so we can see if it is the default handle (no animation clip).
        let no_animation = !animation_player.is_changed() && animation_player.elapsed() == 0.0;

        let current_animation = animator
            .current_animation
            .as_ref()
            .and_then(|x| animation_asset.get(x));

        // There is no way to check if animation player is looping?
        let animation_ended = current_animation
            .map(|x| animation_player.elapsed() > x.duration())
            .unwrap_or_default();

        if (no_animation || animation_ended)
            && (animator.current_animation.as_ref() != Some(&enemy_animations.run))
        {
            animator.current_animation = Some(enemy_animations.run.clone());
            animation_player.play(enemy_animations.run.clone()).repeat();
        }
    }
}
#[derive(Clone, Copy, EnumCount, Display, EnumVariantNames, EnumIter, Component)]
pub enum ModelKind {
    Mouse,
}

impl ModelKind {
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
