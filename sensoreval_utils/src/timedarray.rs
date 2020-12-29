pub struct TimedArray<'a> {
    id: usize,
    data: &'a [Vec<f64>],
    next_data_time: Option<f64>,
}

impl<'a> TimedArray<'a> {
    pub fn new(data: &'a [Vec<f64>]) -> Self {
        let mut o = Self {
            id: 0,
            data,
            next_data_time: None,
        };
        o.update_next();
        o
    }

    pub fn next(&mut self, t: f64) -> Option<&[f64]> {
        if let Some(next_data_time) = self.next_data_time {
            if t >= next_data_time {
                let ret = Some(&self.data[self.id][1..]);

                self.id += 1;
                self.update_next();
                return ret;
            }
        }

        None
    }

    fn update_next(&mut self) {
        self.next_data_time = if self.id >= self.data.len() {
            None
        } else {
            Some(self.data[self.id][0])
        };
    }
}
