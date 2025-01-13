use crate::{path::get_path, ApplyMove, Map, MyTurn, Position, WantsToApproach};
use specs::prelude::*;

pub struct ApproachAI {}

impl<'a> System<'a> for ApproachAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToApproach>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut want_approach, positions, mut map, entities, mut apply_move) = data;

        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, pos, approach, _myturn) in
            (&entities, &positions, &want_approach, &turns).join()
        {
            turn_done.push(entity);

            let (target_x, target_y) = map.idx_xy(approach.idx as usize);
            let path = get_path(pos.x, pos.y, target_x, target_y, &mut *map);
            if path.success && path.steps.len() > 1 {
                apply_move
                    .insert(
                        entity,
                        ApplyMove {
                            dest_idx: path.steps[1],
                        },
                    )
                    .expect("Unable to insert");
            }
        }

        want_approach.clear();

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
