#![feature(test)]
extern crate test;

// BIG TODO: bench all of this and make the code testable (
pub fn add_two(a: i32) -> i32 {
    a + 2
}

#[cfg(test)]
mod tests {
    //use super::*;
    use test::Bencher;

    #[bench]
    fn one_eventloop_100(b: &mut Bencher) {
        b.iter(|| {

        });
    }
    #[bench]
    fn two_eventloops_100(b: &mut Bencher) {
        b.iter(|| {

        });
    }
    #[bench]
    fn two_eventloops_cpupool_100(b: &mut Bencher) {
        b.iter(|| {

        });
    }
}

