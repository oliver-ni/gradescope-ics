pub mod course;
pub mod home;

macro_rules! select {
    ($elt:expr, $selector:literal) => {{
        use std::sync::LazyLock;

        use scraper::Selector;

        static SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse($selector).unwrap());

        $elt.select(&SELECTOR)
    }};
}

pub(crate) use select;
