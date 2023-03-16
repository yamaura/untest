#![feature(test)]
#![feature(internal_output_capture)]

use std::io::{self, Write};

pub fn ddd_test_a() {
    assert!(false, "This is fail");
}

extern crate test;
pub fn test_runner(tests: &[&test::TestDescAndFn]) -> () {
    extern crate test;
    use std::env;

    env::set_var("RUST_BACKTRACE", "0");
    // Force thread number to 1
    env::set_var("RUST_TEST_THREADS", "1");
    for &t in tests {
        println!("... {}", t.desc.name.as_slice());
    }

    {
        use std::panic::{self, PanicInfo};
        // Update panic
        let _default_panic = panic::take_hook();
        let hook = Box::new({
            move |info: &'_ PanicInfo<'_>| {
                // The current implementation always returns `Some`.
                let location = info.location().unwrap();

                let msg = match info.payload().downcast_ref::<&'static str>() {
                    Some(s) => *s,
                    None => match info.payload().downcast_ref::<String>() {
                        Some(s) => &s[..],
                        None => "Box<dyn Any>",
                    },
                };
                let write = |err: &mut dyn crate::io::Write| {
                    let _ = writeln!(err, "{location}:\n {msg}");
                };
                if let Some(local) = io::set_output_capture(None) {
                    write(&mut *local.lock().unwrap_or_else(|e| e.into_inner()));
                    io::set_output_capture(Some(local));
                } else {
                    write(&mut io::stdout().lock());
                }
            }
        });
        panic::set_hook(hook);
    }
    //let owned_tests: Vec<_> = tests.iter().map(make_owned_test).collect();
    test::test_main_static(tests); // original
}

// from librs
pub fn make_owned_test(test: &&test::TestDescAndFn) -> test::TestDescAndFn {
    extern crate test;

    match test.testfn {
        test::StaticTestFn(f) => test::TestDescAndFn {
            testfn: test::StaticTestFn(f),
            desc: test.desc.clone(),
        },
        test::StaticBenchFn(f) => test::TestDescAndFn {
            testfn: test::StaticBenchFn(f),
            desc: test.desc.clone(),
        },
        _ => panic!("non-static tests passed to test::test_main_static"),
    }
}
