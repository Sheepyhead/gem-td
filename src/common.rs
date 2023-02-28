use bevy::{prelude::*, utils::HashMap};

pub fn approx_equal(a: f32, b: f32) -> bool {
    let margin = f32::EPSILON;
    (a - b).abs() < margin
}

#[derive(Default, Resource)]
pub struct BuildGrid(HashMap<IVec2, Entity>);

impl BuildGrid {
    pub fn insert(&mut self, pos: IVec2, content: Entity) {
        let positions = [pos, pos + IVec2::X, pos + IVec2::Y, pos + IVec2::ONE];
        if positions.iter().all(|pos| !self.contains(pos)) {
            for pos in positions {
                self.0.insert(pos, content);
            }
        }
    }

    pub fn contains(&self, pos: &IVec2) -> bool {
        self.0.contains_key(pos)
    }

    pub fn get(&self, pos: &IVec2) -> Option<Entity> {
        self.0.get(pos).copied()
    }

    pub fn remove(&mut self, pos: &IVec2) {
        if let Some(entity) = self.get(pos) {
            let possible_positions = [
                *pos + IVec2::NEG_X,
                *pos + IVec2::NEG_Y,
                *pos + IVec2::NEG_ONE,
                *pos + IVec2::X,
                *pos + IVec2::Y,
                *pos + IVec2::ONE,
                *pos + IVec2::NEG_X + IVec2::Y,
                *pos + IVec2::X + IVec2::NEG_Y,
            ];
            possible_positions
                .iter()
                .filter(|pos| {
                    self.get(*pos)
                        .is_some_and(|pos_entity| pos_entity == entity)
                })
                .copied()
                .collect::<Vec<_>>()
                .iter()
                .for_each(|pos| self.remove(pos));
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub number: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    StartMenu,
    InGame,
}
