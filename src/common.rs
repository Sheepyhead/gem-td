use bevy::{math::Vec3Swizzles, prelude::*};

// Converts the cursor position into a world position, taking into account any transforms applied
// the camera.
pub fn cursor_pos_in_world(
    window: &Window,
    cursor_pos: Vec2,
    cam_t: &Transform,
    cam: &Camera,
) -> Vec3 {
    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    // (ndc = normalized device coordinates)
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix().inverse();
    let ndc = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
    ndc_to_world.project_point3(ndc.extend(0.0))
}

#[derive(Resource)]
pub struct CursorPos(pub Vec3);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec3::new(-1000.0, -1000.0, 0.0))
    }
}

#[derive(Component)]
pub struct TrackWorldObjectToScreenPosition {
    pub target: Entity,
    pub offset: Vec2,
}

impl TrackWorldObjectToScreenPosition {
    pub fn track(
        windows: Query<&Window>,
        cameras: Query<(&GlobalTransform, &Camera)>,
        world_objects: Query<&GlobalTransform>,
        mut tracking_objects: Query<(&mut Style, &TrackWorldObjectToScreenPosition)>,
    ) {
        for (mut style, TrackWorldObjectToScreenPosition { target, offset }) in
            &mut tracking_objects
        {
            if let Ok(world_pos) = world_objects.get(*target) {
                if let Size {
                    width: Val::Px(width),
                    height: Val::Px(height),
                } = style.size
                {
                    let (cam_pos, camera) = cameras.single();
                    if let Some(screen_position) =
                        camera.world_to_viewport(cam_pos, world_pos.translation())
                    {
                        let window = windows.single();
                        let new_pos = UiRect::new(
                            Val::Px(screen_position.x - width / 2.0 + offset.x),
                            Val::Auto,
                            Val::Px(window.height() - screen_position.y - height / 2.0 + offset.y),
                            Val::Auto,
                        );

                        style.position = new_pos;
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct MovingTo {
    pub destination: Vec2,
}

impl MovingTo {
    pub fn move_to(time: Res<Time>, mut movers: Query<(&mut Transform, &MovingTo)>) {
        for (mut transform, MovingTo { destination }) in &mut movers {
            let direction = (*destination - transform.translation.xy()).normalize();
            let movement = Vec2::new(direction.x * 10.0, direction.y * 5.0) * time.delta_seconds();
            transform.translation += movement.extend(0.0);
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Fadeout(pub Timer);

impl Fadeout {
    pub fn fadeout(
        mut commands: Commands,
        time: Res<Time>,
        mut fadeouts: Query<(Entity, &mut Fadeout)>,
    ) {
        for (entity, mut timer) in &mut fadeouts {
            if timer.tick(time.delta()).finished() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

#[derive(Component)]
pub struct AvoidZOrderClamping;

pub fn clamp_z_order_to_y(
    mut transforms: Query<&mut Transform, (Without<AvoidZOrderClamping>, Changed<Transform>)>,
) {
    for mut transform in &mut transforms {
        transform.translation.z = -transform.translation.y;
    }
}

#[derive(Deref, DerefMut, Resource)]
pub struct Builds(pub u32);

impl Default for Builds {
    fn default() -> Self {
        Self(5)
    }
}

impl Builds {
    pub fn reset_system(mut builds: ResMut<Builds>) {
        *builds = Self::default();
    }
}
