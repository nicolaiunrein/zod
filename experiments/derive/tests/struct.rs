mod test {
    use zod_derive_experiments::Zod;
    #[derive(Zod)]
    struct Hello {
        value: u8,
    }
}
