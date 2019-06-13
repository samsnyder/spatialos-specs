use crate::component_registry::ComponentRegistry;
use crate::system_commands::SystemCommandSender;
use spatialos_sdk::worker::connection::WorkerConnection;
use specs::prelude::{System, WriteExpect};
use crate::spatial_reader::ResourcesSystemData;

pub struct SpatialWriterSystem;

impl<'a> System<'a> for SpatialWriterSystem {
    type SystemData = (
        WriteExpect<'a, WorkerConnection>,
        SystemCommandSender<'a>,
        ResourcesSystemData<'a>);

    fn run(&mut self, (mut connection, mut system_command_sender, res): Self::SystemData) {
        for interface in res.res.fetch::<ComponentRegistry>().interfaces_iter() {
            interface.replicate(&res.res, &mut connection);
        }

        system_command_sender.flush_requests(&mut connection);
    }
}
