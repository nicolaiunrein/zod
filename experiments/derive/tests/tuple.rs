mod test {
    use zod_derive_experiments::Zod;
    #[derive(Zod)]
    struct Tuple(u8, String);
}
