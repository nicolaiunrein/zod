// Pretend that this is somebody else's crate, not a module.
mod other_crate {
    // Neither Serde nor the other crate provides Serialize and Deserialize
    // impls for this struct.
    pub struct Duration {
        pub secs: i64,
        pub nanos: i32,
    }
}

////////////////////////////////////////////////////////////////////////////////

use other_crate::Duration;
use serde::{Deserialize, Serialize};

// Serde calls this the definition of the remote type. It is just a copy of the
// remote data structure. The `remote` attribute gives the path to the actual
// type we intend to derive code for.
#[derive(Serialize, Deserialize)]
#[serde(remote = "Duration")]
struct DurationDef {
    secs: i64,
    nanos: i32,
}

// Now the remote type can be used almost like it had its own Serialize and
// Deserialize impls all along. The `with` attribute gives the path to the
// definition for the remote type. Note that the real type of the field is the
// remote type, not the definition type.
#[derive(Serialize, Deserialize)]
struct Process {
    command_line: String,

    #[serde(with = "DurationDef")]
    wall_time: Duration,
}


fn main(){

}
