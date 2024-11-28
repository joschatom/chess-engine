

pub struct Profiler {
    completed: HashMap<&'static str, Duration>,
    running: HashMap<&'satic str, (Instant, Duration)>,
    paused: HashMap<&'static str, Duration>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            completed: HashMap::new(),
            running: HashMap::new(),
            paused: HashMap::new(),
        }
    }

    pub fn begin(&mut self, name: &'static str) {
        if running.contains(name) {
            panic!("profiler: task with name \"{}\" is already running");
        }

        if paused.contains(name) {
            panic!("task with name \"{}\" already exists and is paused");
        }

        self.running.insert(name, (Instant::now(), Duration::ZERO));
    }

    pub fn stop(&mut self, name: &'static str) {
        let end = Instant::now();

        let (start, dur) = self.running.get(name).expect("tried to stop profiling a non-existing task");

        self.completed.insert(name, dur + (end - start));
    } 
}

