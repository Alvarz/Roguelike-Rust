use specs::prelude::*;

use crate::{AmuletOfYendor, Name};

pub struct AmuletSystem {}

impl<'a> System<'a> for AmuletSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, crate::components::Player>,
        ReadStorage<'a, AmuletOfYendor>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, amulet_if_yendor, name) = data;

        for (_p, _amulet, n) in (&player, &amulet_if_yendor, &name).join() {
            rltk::console::log(format!("{} has the amulet", n.name));
        }

        // Clean up
        //amulet_if_yendor.clear();
    }
}
