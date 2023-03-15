use std::collections::HashSet;

use zod_core::Namespace;

fn main() {
    let mut seen = HashSet::new();
    println!("import * as z from \"zod\"");
    println!("export namespace Rs {{ \n");
    for member in zod_core::Rs::members() {
        if seen.get(member.type_def).is_none() {
            println!("{}", member.type_def);
            seen.insert(member.type_def);
        }

        if seen.get(member.schema).is_none() {
            println!("{}", member.schema);
            seen.insert(member.schema);
        }
        println!();
    }
    println!("}}")
}
