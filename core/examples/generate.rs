use zod_core::Namespace;

fn main() {
    println!("import * as z from \"zod\"");
    println!("{}", zod_core::Rs::generate())
}
