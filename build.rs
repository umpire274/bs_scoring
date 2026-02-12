fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/bs_scoring_1.ico");
        res.compile().expect("Failed to embed Windows icon");
    }
}
