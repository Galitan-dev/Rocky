use std::{thread, io::{self, Write}};
use crate::vm::{VM, VMEvent};

#[allow(unused)]
#[derive(Default)]
pub struct Scheduler {
    next_pid: u32,
    max_pid: u32,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            next_pid: 0,
            max_pid: 50_000
        }
    }

    pub fn get_thread(&self, mut vm: VM) -> thread::JoinHandle<Vec<VMEvent>> {
        thread::spawn(move || {
            let events = vm.run();
            println!("--------------------------");
            println!("VM Events");
            println!("--------------------------");
            for event in &events {
                println!("{:#?}", event);
            };
            print!(">>> ");
            io::stdout().flush().expect("Unable to flush stdout");
            events
        })
    }
}