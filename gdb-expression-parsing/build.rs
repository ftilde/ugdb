extern crate lalrpop;

fn main() {
    // Preprocess lalrpop grammar files
    lalrpop::process_root().unwrap();
}
