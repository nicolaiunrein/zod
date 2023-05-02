use zod::Namespace;
use zod::RequestType;

#[derive(RequestType)]
#[zod(namespace = "Ns")]
struct Test;

#[derive(Namespace)]
struct Ns {}

fn main() {}
