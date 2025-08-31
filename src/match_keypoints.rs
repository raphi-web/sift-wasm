fn l2_sq(a: &[f32], b: &[f32]) -> f32 {
    // Calculates squared L2 distance between two vectors
    // Returns sum of squared differences between corresponding elements
    let mut acc = 0.0f32;
    for i in 0..a.len() {
        let d = a[i] - b[i];
        acc += d * d;
    }
    acc
}

fn best_two_in_set<'a>(query: &'a [f32], set: &'a [f32], d: usize) -> Option<(usize, f32, f32)> {
    // Finds the two closest vectors to query in set based on L2 distance
    // Returns index of best match along with best and second best distances
    if d == 0 { return None; }
    let n = set.len() / d;
    if n == 0 { return None; }

    let mut best_idx = usize::MAX;
    let mut best = f32::INFINITY;
    let mut second = f32::INFINITY;

    for j in 0..n {
        let start = j * d;
        let cand = &set[start..start + d];
        let dist = l2_sq(query, cand);
        if dist < best {
            second = best;
            best = dist;
            best_idx = j;
        } else if dist < second {
            second = dist;
        }
    }
    if best_idx == usize::MAX { None } else { Some((best_idx, best, second)) }
}

fn best_index_in_set(query: &[f32], set: &[f32], d: usize) -> Option<usize> {
    // Finds index of closest vector to query in set based on L2 distance
    // Returns index of best match if set is not empty
    if d == 0 { return None; }
    let n = set.len() / d;
    if n == 0 { return None; }

    let mut best_idx = 0usize;
    let mut best = f32::INFINITY;
    for j in 0..n {
        let start = j * d;
        let cand = &set[start..start + d];
        let dist = l2_sq(query, cand);
        if dist < best {
            best = dist;
            best_idx = j;
        }
    }
    Some(best_idx)
}

pub fn match_descriptors_with_scores(desc1: &[f32], desc2: &[f32], d: usize, ratio: f32, cross_check: bool) -> Vec<f32> {
    // Matches descriptors between two sets using ratio test and optional cross checking
    // Returns vector of matched indices and distances as [i1,j1,dist1, i2,j2,dist2, ...]
    assert!(d > 0, "descriptor dimension must be > 0");
    assert_eq!(desc1.len() % d, 0, "desc1 length must be a multiple of d");
    assert_eq!(desc2.len() % d, 0, "desc2 length must be a multiple of d");

    let n1 = desc1.len() / d;
    let n2 = desc2.len() / d;
    if n1 == 0 || n2 == 0 {
        return Vec::new();
    }

    let mut out: Vec<f32> = Vec::with_capacity(n1 * 3);

    for i in 0..n1 {
        let q = &desc1[i * d..(i + 1) * d];
        if let Some((j, best, second)) = best_two_in_set(q, desc2, d) {
            if second.is_finite() && best.is_finite() && second > 0.0 && (best / second) < ratio {
                if cross_check {
                    let q2 = &desc2[j * d..(j + 1) * d];
                    if let Some(bi) = best_index_in_set(q2, desc1, d) {
                        if bi == i {
                            out.push(i as f32);
                            out.push(j as f32);
                            out.push(best);
                        }
                    }
                } else {
                    out.push(i as f32);
                    out.push(j as f32);
                    out.push(best);
                }
            }
        }
    }
    out
}

pub fn match_descriptors_topk_impl(desc1: &[f32], desc2: &[f32], d: usize, ratio: f32, cross_check: bool, top_k: usize) -> Vec<u32> {
    // Finds top-k matches between descriptor sets based on distance
    // Returns vector of matched indices [i1,j1, i2,j2, ...] for top k matches
    let scored = match_descriptors_with_scores(desc1, desc2, d, ratio, cross_check);
    let mut triples: Vec<(u32, u32, f32)> = scored
        .chunks_exact(3)
        .map(|c| (c[0] as u32, c[1] as u32, c[2]))
        .collect();

    triples.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    let take = top_k.min(triples.len());
    let mut out = Vec::with_capacity(take * 2);
    for (i, j, _) in triples.into_iter().take(take) {
        out.push(i);
        out.push(j);
    }
    out
}