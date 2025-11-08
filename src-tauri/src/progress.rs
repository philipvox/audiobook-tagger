use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct ScanProgress {
    pub current: usize,
    pub total: usize,
    pub current_book: String,
    pub elapsed_seconds: u64,
    pub estimated_remaining_seconds: u64,
    pub files_per_second: f64,
    pub cached_hits: usize,
}

impl ScanProgress {
    pub fn new(total: usize) -> Self {
        Self {
            current: 0,
            total,
            current_book: String::new(),
            elapsed_seconds: 0,
            estimated_remaining_seconds: 0,
            files_per_second: 0.0,
            cached_hits: 0,
        }
    }
    
    pub fn update(&mut self, current: usize, book_name: &str, start_time: std::time::Instant, cached: bool) {
        self.current = current;
        self.current_book = book_name.to_string();
        self.elapsed_seconds = start_time.elapsed().as_secs();
        
        if cached {
            self.cached_hits += 1;
        }
        
        if current > 0 {
            self.files_per_second = current as f64 / start_time.elapsed().as_secs_f64();
            let remaining = self.total - current;
            self.estimated_remaining_seconds = (remaining as f64 / self.files_per_second) as u64;
        }
    }
}
