fn main() {
    // Sematkan ikon ke .exe di Windows. Di OS lain tidak melakukan apa-apa.
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().expect("gagal menyematkan ikon (butuh Windows SDK / rc.exe)");
    }
}
