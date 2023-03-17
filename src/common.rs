use bevy::{math::Vec3Swizzles, prelude::*};
use seldom_interop::prelude::Position2;

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

pub fn ray_from_screenspace(
    cursor_pos_screen: Vec2,
    window: &Window,
    camera: &Camera,
    projection: &PerspectiveProjection,
    camera_transform: &GlobalTransform,
    length: f32,
) -> (Vec3, Vec3) {
    let view = camera_transform.compute_matrix();
    let screen_size = Vec2::from([window.width(), window.height()]);
    let projection_matrix = camera.projection_matrix();

    // 2D Normalized device coordinate cursor position from (-1, -1) to (1, 1)
    let cursor_ndc = (cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
    let ndc_to_world: Mat4 = view * projection_matrix.inverse();
    let world_to_ndc = projection_matrix * view;
    let is_orthographic = approx_equal(projection_matrix.w_axis[3], 1.0);

    // Compute the cursor position at the near plane. The bevy camera looks at -Z.
    let ndc_near = world_to_ndc.transform_point3(-Vec3::Z * projection.near).z;
    let cursor_pos_near = ndc_to_world.transform_point3(cursor_ndc.extend(ndc_near));

    // Compute the ray's direction depending on the projection used.
    let ray_direction = if is_orthographic {
        view.transform_vector3(-Vec3::Z)
    } else {
        cursor_pos_near - camera_transform.translation()
    };

    (cursor_pos_near, ray_direction * length)
}

pub fn approx_equal(a: f32, b: f32) -> bool {
    let margin = f32::EPSILON;
    (a - b).abs() < margin
}

#[derive(Component)]
pub struct CreepPos {
    pub pos: Vec2,
}

impl Position2 for CreepPos {
    type Position = Vec2;

    fn get(&self) -> Self::Position {
        self.pos
    }

    fn set(&mut self, pos: Self::Position) {
        self.pos = pos;
    }
}

pub fn update_creep_position(mut creeps: Query<(&mut Transform, &CreepPos), Changed<CreepPos>>) {
    for (mut transform, pos) in &mut creeps {
        transform.translation = Vec3::new(pos.pos.x, transform.translation.y, pos.pos.y);
    }
}
