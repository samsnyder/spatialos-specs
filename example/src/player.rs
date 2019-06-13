use crate::generated::game::*;
use crate::generated::improbable::*;
use spatialos_specs::*;
use specs::prelude::*;

pub struct MovePlayerSys;

const DISTANCE_PER_FRAME: f64 = 0.1;
const DISTANCE: f64 = 5.0;

impl<'a> System<'a> for MovePlayerSys {
    type SystemData = (
        SpatialWriteStorage<'a, Player>,
        SpatialWriteStorage<'a, Position>,
    );

    fn run(&mut self, (mut player, mut position): Self::SystemData) {
        for (player, position) in (&mut player, &mut position).join() {
            let mut change_direction = false;

            match player.current_direction {
                0 => {
                    position.coords.x += DISTANCE_PER_FRAME;
                    change_direction = position.coords.x > DISTANCE;
                }
                1 => {
                    position.coords.z += DISTANCE_PER_FRAME;
                    change_direction = position.coords.z > DISTANCE;
                }
                2 => {
                    position.coords.x -= DISTANCE_PER_FRAME;
                    change_direction = position.coords.x < -DISTANCE;
                }
                3 => {
                    position.coords.z -= DISTANCE_PER_FRAME;
                    change_direction = position.coords.z < -DISTANCE;
                }
                _ => {}
            }

            if change_direction {
                player.current_direction = (player.current_direction + 1) % 4;
            }
        }
    }
}
