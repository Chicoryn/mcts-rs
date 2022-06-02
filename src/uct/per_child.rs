use std::sync::atomic::{Ordering, AtomicU64};
use super::update::Update;

pub struct PerChild {
    atomic_per_child: AtomicU64,
}

#[inline]
fn pack(value: f32, total_visits: u32) -> u64 {
    ((f32::to_bits(value) as u64) << 32) | total_visits as u64
}

#[inline]
fn unpack(packed: u64) -> (f32, u32) {
    let value = (packed >> 32) as u32;
    let total_visits = (packed & 0xffffffff) as u32;

    (f32::from_bits(value), total_visits)
}

impl Clone for PerChild {
    fn clone(&self) -> Self {
        Self {
            atomic_per_child: AtomicU64::new(self.atomic_per_child.load(Ordering::Relaxed))
        }
    }
}

impl PartialEq for PerChild {
    fn eq(&self, rhs: &Self) -> bool {
        let per_child = self.atomic_per_child.load(Ordering::Relaxed);
        let rhs_per_child = rhs.atomic_per_child.load(Ordering::Relaxed);

        per_child == rhs_per_child
    }
}

impl PerChild {
    pub fn new() -> Self {
        Self {
            atomic_per_child: AtomicU64::new(pack(0.0, 0))
        }
    }

    pub fn update(&self, up: &Update) {
        self.atomic_per_child.fetch_update(Ordering::AcqRel, Ordering::Acquire, |prev_value| {
            let (value, total_visits) = unpack(prev_value);

            Some(pack(value + up.value(), total_visits + 1))
        }).unwrap();
    }

    #[inline]
    pub fn visits(&self) -> u32 {
        unpack(self.atomic_per_child.load(Ordering::Relaxed)).1
    }

    #[inline]
    pub fn total_value(&self) -> f32 {
        unpack(self.atomic_per_child.load(Ordering::Relaxed)).0
    }

    #[inline(always)]
    pub fn win_rate(&self, total_value: f32, visits: u32) -> f32 {
        if visits > 0 {
            total_value / visits as f32
        } else {
            0.0f32
        }
    }

    #[inline(always)]
    pub fn uct(&self, total_visits: u32) -> f32 {
        let ln_n = (total_visits as f32).ln();
        let (value, visits) = unpack(self.atomic_per_child.load(Ordering::Relaxed));

        self.win_rate(value, visits) + (2.0f32 * ln_n / (visits + 1) as f32).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack_pack() {
        let (value, visits) = unpack(pack(3.14, 7));

        assert_eq!(value, 3.14);
        assert_eq!(visits, 7);
    }
}
