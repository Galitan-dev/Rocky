#[macro_use]
extern crate criterion;
extern crate rocky;

use criterion::Criterion;
use rocky::vm::VM;

mod examples {
    use rocky::run_file;

    use super::*;

    fn execute_hello_rk(c: &mut Criterion) {
        let clos = || run_file(num_cpus::get(), "examples/hello.rk");

        c.bench_function("execute_hello_rk", move |b| b.iter(clos));
    }

    fn execute_math_rk(c: &mut Criterion) {
        let clos = || run_file(num_cpus::get(), "examples/math.rk");

        c.bench_function("execute_math_rk", move |b| b.iter(clos));
    }

    criterion_group! {
        name = examples;
        config = Criterion::default();
        targets = execute_math_rk, execute_hello_rk
    }
}

mod arithmetic {
    use super::*;

    fn execute_add(c: &mut Criterion) {
        let clos = {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 50;
            test_vm.registers[1] = 25;
            test_vm.set_program(vec![2, 0, 1, 0], Vec::new());
            test_vm.run_once();
        };

        c.bench_function("execute_add", move |b| b.iter(|| clos));
    }

    fn execute_sub(c: &mut Criterion) {
        let clos = {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 50;
            test_vm.registers[1] = 25;
            test_vm.set_program(vec![3, 0, 1, 0], Vec::new());
            test_vm.run_once();
        };

        c.bench_function("execute_sub", move |b| b.iter(|| clos));
    }

    fn execute_mul(c: &mut Criterion) {
        let clos = {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 50;
            test_vm.registers[1] = 5;
            test_vm.set_program(vec![4, 0, 1, 0], Vec::new());
            test_vm.run_once();
        };

        c.bench_function("execute_mul", move |b| b.iter(|| clos));
    }

    fn execute_div(c: &mut Criterion) {
        let clos = {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 50;
            test_vm.registers[1] = 6;
            test_vm.set_program(vec![5, 0, 1, 0], Vec::new());
            test_vm.run_once();
        };

        c.bench_function("execute_div", move |b| b.iter(|| clos));
    }

    criterion_group! {
        name = arithmetic;
        config = Criterion::default();
        targets = execute_add, execute_sub, execute_mul, execute_div,
    }
}

criterion_main!(arithmetic::arithmetic, examples::examples);
