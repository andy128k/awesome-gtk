mod bitset;
mod widget;

macro_rules! run_test {
    ($name:path) => {{
        eprint!("test {} ... ", stringify!($name));
        $name();
        eprintln!("ok");
    }};
}

fn main() {
    eprintln!("");
    gtk::init().unwrap();

    run_test!(bitset::test_bitset_iter);

    run_test!(widget::test_iter);
    run_test!(widget::test_iter_rev);
    run_test!(widget::test_traverse);

    eprintln!("");
}
