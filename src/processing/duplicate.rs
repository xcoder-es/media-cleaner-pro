use super::hash::hamming_distance;
use std::collections::HashMap;

/// Bucket-based duplicate detection using dHash prefix
/// This reduces O(n²) comparisons to near O(n)
pub struct DuplicateDetector {
    threshold: u32,
    buckets: HashMap<u16, Vec<(String, u64)>>, // prefix bucket -> [(path, dhash)]
}

impl DuplicateDetector {
    pub fn new(threshold: u32) -> Self {
        DuplicateDetector {
            threshold,
            buckets: HashMap::new(),
        }
    }

    /// Add an image to the detector. Returns paths of detected duplicates.
    pub fn add(&mut self, path: String, dhash: u64) -> Vec<String> {
        let prefix = (dhash >> 48) as u16; // Top 16 bits as bucket key
        let mut duplicates = Vec::new();

        // Check against existing items in same bucket
        if let Some(bucket) = self.buckets.get(&prefix) {
            for (existing_path, existing_hash) in bucket {
                let distance = hamming_distance(dhash, *existing_hash);
                if distance <= self.threshold {
                    duplicates.push(existing_path.clone());
                }
            }
        }

        // Also check adjacent buckets for border cases
        for adj in [prefix.wrapping_sub(1), prefix.wrapping_add(1)] {
            if let Some(bucket) = self.buckets.get(&adj) {
                for (existing_path, existing_hash) in bucket {
                    let distance = hamming_distance(dhash, *existing_hash);
                    if distance <= self.threshold {
                        if !duplicates.contains(existing_path) {
                            duplicates.push(existing_path.clone());
                        }
                    }
                }
            }
        }

        self.buckets
            .entry(prefix)
            .or_default()
            .push((path, dhash));

        duplicates
    }

    pub fn clear(&mut self) {
        self.buckets.clear();
    }
}
