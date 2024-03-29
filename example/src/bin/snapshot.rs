use example::generated::game::*;
use example::generated::improbable::*;
use spatialos_sdk::worker::entity::Entity as WorkerEntity;
use spatialos_sdk::worker::entity_builder::EntityBuilder;
use spatialos_sdk::worker::snapshot::*;
use spatialos_sdk::worker::*;
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    let current_dir = std::env::current_dir().expect("Could not find current working directory.");

    let mut path_buf = PathBuf::new();
    path_buf.push(current_dir);
    path_buf.push(opt.snapshot_path);

    let snapshot_path = path_buf.to_str().unwrap();
    println!("Creating snapshot at: {}", snapshot_path);

    let mut stream =
        SnapshotOutputStream::new(snapshot_path).expect("Failed to create snapshot stream.");

    println!(
        "{:?}",
        stream.write_entity(EntityId::new(1), &create_player_creator_entity())
    );
}

fn create_player_creator_entity() -> WorkerEntity {
    let mut builder = EntityBuilder::new(0.0, 0.0, 0.0, "managed");

    builder.add_component(PlayerCreator {}, "managed");
    builder.add_component(Persistence {}, "managed");
    builder.set_metadata("PlayerCreator", "managed");
    builder.set_entity_acl_write_access("managed");

    builder.build().unwrap()
}

#[derive(StructOpt, Debug)]
#[structopt(name = "generate_snapshot")]
struct Opt {
    /// Relative path for the snapshot to be written to.
    #[structopt(short = "p", long = "snapshot-path")]
    snapshot_path: PathBuf,
}
