use crate::{AmuletOfYendor, Map, MyTurn, Name};
use specs::prelude::*;

pub struct AmuletSystem {}

impl<'a> System<'a> for AmuletSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadStorage<'a, AmuletOfYendor>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, MyTurn>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, amulet_if_yendor, name, turns) = data;

        for (_amulet, n, _my_turn) in (&amulet_if_yendor, &name, &turns).join() {
            rltk::console::log(format!("{} has the amulet in depth: {}", n.name, map.depth));

            if map.depth <= 1 {
                rltk::console::log(format!("{} won the game!", n.name,));
            }
        }

        // Clean up
        //amulet_if_yendor.clear();
    }
}
