pub struct Histogram {
    buckets: Vec<u64>,
}

#[derive(Debug, Default)]
pub struct Summary {
    pub min: u64,
    pub p50: u64,
    pub p90: u64,
    pub p99: u64,
    pub max: u64,
    pub samples: u64,
}

pub fn format_ns(ns: u64) -> String {
    if ns < 1000 {
        format!("{} ns", ns)
    } else if ns < 1_000_000 {
        format!("{} us", ns / 1_000)
    } else if ns < 1_000_000_000 {
        format!("{} ms", ns / 1_000_000)
    } else {
        format!("{} s", ns / 1_000_000_000)
    }
}

pub fn summary_to_table(summary: &Summary) -> comfy_table::Table {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec!["min", "p50", "p90", "p99", "max", "# samples"]);
    table.add_row(
        [
            summary.min,
            summary.p50,
            summary.p90,
            summary.p99,
            summary.max,
        ]
        .iter()
        .map(|&d| format_ns(d))
        .chain(std::iter::once(summary.samples.to_string())),
    );
    table
}

impl Default for Histogram {
    fn default() -> Self {
        Self {
            buckets: vec![0; 512],
        }
    }
}

impl Histogram {
    pub fn add(&mut self, bucket: usize, count: u64) {
        self.buckets[bucket] += count;
    }

    pub fn summary(mut self) -> Summary {
        let b_min = self.buckets.iter().position(|&x| x > 0).unwrap();
        let b_max = self.buckets.iter().rposition(|&x| x > 0).unwrap();

        for i in 1..self.buckets.len() {
            self.buckets[i] += self.buckets[i - 1];
        }

        let sum = *self.buckets.last().unwrap();

        if sum == 0 {
            return Default::default();
        }

        let v50 = sum / 2; // p50 = the first index with buckets[i] >= v50
        let v90 = sum * 9 / 10;
        let v99 = sum * 99 / 100;
        let b50 = self.buckets.partition_point(|&x| x <= v50);
        let b90 = self.buckets.partition_point(|&x| x <= v90);
        let b99 = self.buckets.partition_point(|&x| x <= v99);

        Summary {
            min: bucket_to_value(b_min),
            p50: bucket_to_value(b50),
            p90: bucket_to_value(b90),
            p99: bucket_to_value(b99),
            max: bucket_to_value(b_max),
            samples: sum,
        }
    }
}

fn bucket_to_value(bucket: usize) -> u64 {
    if bucket < 16 {
        bucket as u64
    } else {
        let log = bucket / 8 + 2;
        (((bucket & 0b111) + 8) as u64) << (log - 3)
    }
}
