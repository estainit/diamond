use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct CMachine {
    count: Mutex<u8>,

    m_clone_id: Mutex<i8>,
    m_should_loop_threads: Mutex<bool>,
    // = true;
    m_is_develop_mod: Mutex<bool>,// = true;
}

impl CMachine {
    pub fn get_instance() -> Arc<CMachine> {
        SINGLETON_POOL.with(|singleton_pool| singleton_pool.clone())
    }

    pub fn init() -> bool
    {
        let singleton = CMachine::get_instance();

        let mut m_should_loop_threads = singleton.m_should_loop_threads.try_lock().unwrap();
        *m_should_loop_threads = true;
        println!("singleton init m_should_loop_threads: {}", m_should_loop_threads);
        // CMachine::get_instance().m_is_develop_mod = Mutex::from(is_develop_mod);
        println!("::::::::::init>>>>>>>>>>>>>> {:?}",CMachine::get_instance().m_should_loop_threads);


        true
    }

    // func name was parseArgs
    pub fn parse_args(args: Vec<String>, manual_clone_id: i8)
    {
        // println!("Env args: {:?}", args);

        let mut clone_id: i8 = 0;    // FIXME: this value must be defined by command line
        let mut is_develop_mod: bool = false;


        if args.len() > 1 {
            clone_id = args[1].parse().unwrap();
        }

        if manual_clone_id > 0 {
            clone_id = manual_clone_id;
        }

        if args.len() > 2 {
            is_develop_mod = true;
        }

        CMachine::set_clone_dev(clone_id, is_develop_mod);
    }

    // func name was setCloneDev
    pub fn set_clone_dev(clone_id: i8, is_develop_mod: bool) -> bool
    {
        let singleton = CMachine::get_instance();
        let mut m_clone_id = singleton.m_clone_id.try_lock().unwrap();
        *m_clone_id = clone_id;
        println!("singleton init m_clone_id: {}", m_clone_id);
        println!("::::::::::clone dev >>>>>>>>>>>>>> {:?}",CMachine::get_instance().m_should_loop_threads.try_lock());

        // CMachine::get_instance().m_is_develop_mod = Mutex::from(is_develop_mod);
        true
    }


    //func name was shouldLoopThreads
    pub fn should_loop_threads() -> bool {
        println!(">>>>>>>>>>>>>> {:?}",CMachine::get_instance().m_should_loop_threads.try_lock());

        CMachine::get_instance().m_should_loop_threads.try_lock().unwrap().clone()
    }

}

thread_local! {
    static SINGLETON_POOL: Arc<CMachine> = Arc::new(Default::default());
}

fn instance_and_use_singleton() {
    let singleton = CMachine::get_instance();
    let mut count = singleton.count.try_lock().unwrap();
    println!("singleton init value: {}", count);
    *count += 1;
    println!("singleton end value: {}", count);
}

//
// trait Booting{
//     fn parse_args(self, args: Vec<String>, manual_clone_id: i8);
//     fn set_clone_dev(self, clone_id: i8, is_develop_mod: bool) -> bool;
// }





