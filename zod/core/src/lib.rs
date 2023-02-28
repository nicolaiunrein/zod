mod build_ins;

pub use build_ins::*;

pub trait Codegen {
    fn schema() -> String;
    fn docs() -> Option<&'static str> {
        None
    }
    fn type_def() -> String;
    fn type_name() -> String {
        Self::type_def()
    }
}

pub trait Namespace {
    const NAME: &'static str;

    fn docs() -> Option<&'static str> {
        None
    }

    // fn code() -> String {
    // let members =
    // || inventory::iter::<NsMember>().filter(|member| member.ns_name() == Self::NAME);

    // let member_code = members().map(|member| member.decl()).collect::<String>();

    // format!("export namespace {} {{\n{member_code}}};", Self::NAME)
    // }
}
