struct Ns;
impl zod_core::Namespace for Ns {
    const NAME: &'static str = "MyNs";
}
mod test {
    use zod_derive_experiments::Zod;
    #[derive(Zod)]
    #[zod(namespace = "super::Ns")]
    struct Tuple(u8, String);
}
