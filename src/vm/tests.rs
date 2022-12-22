use super::*;

#[test]
fn test_create_vm() {
    let test_vm = VM::new();
    assert_eq!(test_vm.registers[0], 0)
}

mod opcode {
    use super::*;

    #[test]
    fn test_hlt() {
        let mut test_vm = VM::new();
        test_vm.program = vec![0];
        test_vm.program = VM::prepend_header(test_vm.program, Vec::new());
        test_vm.run();
        assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 1);
    }

    #[test]
    fn test_igl() {
        let mut test_vm = VM::new();
        test_vm.set_program(vec![200], Vec::new());
        test_vm.run();
        assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 1);
    }

    #[test]
    fn test_load() {
        let mut test_vm = VM::new();
        test_vm.set_program(vec![1, 0, 1, 244], Vec::new());
        test_vm.run();
        assert_eq!(test_vm.registers[0], 500);
    }

    mod math {
        use super::*;

        #[test]
        fn test_add() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 50;
            test_vm.registers[1] = 25;
            test_vm.set_program(vec![2, 0, 1, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.registers[0], 75);
        }

        #[test]
        fn test_sub() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 50;
            test_vm.registers[1] = 25;
            test_vm.set_program(vec![3, 0, 1, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.registers[0], 25);
        }

        #[test]
        fn test_mul() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 50;
            test_vm.registers[1] = 5;
            test_vm.set_program(vec![4, 0, 1, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.registers[0], 250);
        }

        #[test]
        fn test_div() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 50;
            test_vm.registers[1] = 6;
            test_vm.set_program(vec![5, 0, 1, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.registers[0], 8);
            assert_eq!(test_vm.remainder, 2);
        }
    }

    mod jump {
        use super::*;

        #[test]
        fn test_jmp() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 5;
            test_vm.set_program(vec![6, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 5);
        }

        #[test]
        fn test_jmpf() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 2;
            test_vm.set_program(vec![7, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 4);
        }

        #[test]
        fn test_jmpb() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 2;
            test_vm.set_program(vec![8, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH);
        }
    }

    mod logic {
        use super::*;

        #[test]
        fn test_eq() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 10;
            test_vm.registers[1] = 10;
            test_vm.set_program(vec![9, 0, 1, 0, 9, 0, 1, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, true);
            test_vm.registers[1] = 20;
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, false);
        }

        #[test]
        fn test_neq() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 10;
            test_vm.registers[1] = 20;
            test_vm.set_program(vec![10, 0, 1, 0, 10, 0, 1, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, true);
            test_vm.registers[1] = 10;
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, false);
        }

        #[test]
        fn test_gt() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 20;
            test_vm.registers[1] = 10;
            test_vm.set_program(vec![11, 0, 1, 0, 11, 0, 1, 0, 11, 0, 1, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, true);
            test_vm.registers[1] = 30;
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, false);
            test_vm.registers[1] = 20;
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, false);
        }

        #[test]
        fn test_lt() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 20;
            test_vm.registers[1] = 30;
            test_vm.set_program(vec![12, 0, 1, 0, 12, 0, 1, 0, 12, 0, 1, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, true);
            test_vm.registers[1] = 10;
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, false);
            test_vm.registers[1] = 20;
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, false);
        }

        #[test]
        fn test_gtq() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 20;
            test_vm.registers[1] = 10;
            test_vm.set_program(vec![13, 0, 1, 0, 13, 0, 1, 0, 13, 0, 1, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, true);
            test_vm.registers[1] = 30;
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, false);
            test_vm.registers[1] = 20;
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, true);
        }

        #[test]
        fn test_ltq() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 20;
            test_vm.registers[1] = 30;
            test_vm.set_program(vec![14, 0, 1, 0, 14, 0, 1, 0, 14, 0, 1, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, true);
            test_vm.registers[1] = 10;
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, false);
            test_vm.registers[1] = 20;
            test_vm.run_once();
            assert_eq!(test_vm.equal_flag, true);
        }

        #[test]
        fn test_jeq() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 4;
            test_vm.equal_flag = true;
            test_vm.set_program(vec![15, 0, 0, 0, 15, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 4);
            test_vm.equal_flag = false;
            test_vm.run_once();
            assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 6);
        }
    }

    #[test]
    fn test_aloc() {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 1024;
        test_vm.set_program(vec![16, 0, 0, 0], Vec::new());
        test_vm.run_once();
        assert_eq!(test_vm.memory_heap.len(), 1024);
    }

    #[test]
    fn test_prts_opcode() {
        let mut test_vm = VM::new();
        test_vm.set_program(vec![17, 0, 0, 0], vec![72, 101, 108, 108, 111, 0]);
        test_vm.run_once();
    }

    mod time {
        use super::*;
        use chrono::Utc;

        #[test]
        fn test_slp_opcode() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 100;
            test_vm.set_program(vec![18, 0], Vec::new());
            let start = Utc::now().timestamp_millis();
            test_vm.run_once();
            assert!(Utc::now().timestamp_millis() - start >= 100);
        }

        #[test]
        fn test_slps_opcode() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 1;
            test_vm.set_program(vec![19, 0], Vec::new());
            let start = Utc::now().timestamp_millis();
            test_vm.run_once();
            assert!(Utc::now().timestamp_millis() - start >= 1000);
        }
    }
}
