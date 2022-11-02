use std::{cmp::max, ops::Range};

use itertools::Itertools;
use sdl2::pixels::Color;
use suffix_array_construction::*;

#[derive(Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
enum QueryStep {
    PreviousDone,
    HighlightChar,
    HighlightMatches,
    EquivalenceFirst,
    CountFirst,
    SmallerCountFirst,
    ExtendFirst,
    SmallerWindowSecond,
    EquivalenceSecond,
    CountSecond,
    ComputeSecond,
    ExtendStartSecond,
    ExtendEndSecond,
}
use QueryStep::*;

#[derive(Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
enum State {
    Init,
    // Bool: put last column before first?
    LeftSA(usize),
    RightSA(usize),
    BothSA,
    CharCounts,
    LeftOcc,
    RightOcc,
    // Show equivalence of chars in first/last column.
    // one per char.
    Equivalence(usize),
    Pause,
    Query(usize, QueryStep),
}
use State::*;

fn s_stats(s: &[u8]) -> (usize, usize) {
    let mut cnts = [0; 256];
    for &c in s {
        cnts[c as usize] += 1;
    }
    let num_chars = cnts.iter().filter(|&&x| x > 0).count();
    let max_char_cnt = *cnts.iter().max().unwrap();
    (num_chars, max_char_cnt)
}

const DEFAULT: Color = Color::RGB(240, 240, 240);
const HIGHLIGHT: Color = Color::GREEN;
const SOFT_HIGHLIGHT: Color = Color::RGB(180, 250, 180);
const NEXT_CHAR: Color = Color::RGB(180, 180, 250);

fn to_c(condition: bool) -> Color {
    if condition {
        HIGHLIGHT
    } else {
        DEFAULT
    }
}

struct BWT<'a> {
    s: &'a [u8],
    q: &'a [u8],
    s2: Vec<u8>,
    alph: Vec<u8>,
    char_count: Vec<usize>,
    char_start: Vec<usize>,
    sa: Vec<usize>,
    occ: Vec<Vec<usize>>,
    sa_r: Vec<usize>,
    occ_r: Vec<Vec<usize>>,
}

fn rev(s: &[u8]) -> Vec<u8> {
    s.iter().rev().copied().collect()
}

impl<'a> BWT<'a> {
    fn new(s: &'a [u8], q: &'a [u8]) -> Self {
        let n = s.len();
        let alph = {
            let mut alph = s.to_vec();
            alph.sort();
            alph.dedup();
            alph
        };

        let mut s2 = s.to_vec();
        s2.extend(s);
        s2.extend(s);
        let sa = {
            let mut sa = (0..n).collect_vec();
            sa.sort_by_key(|i| &s[*i..]);
            sa
        };
        let sa_r = {
            let mut sa_r = (1..=n).collect_vec();
            sa_r.sort_by_key(|i| rev(&s2[..n + *i]));
            sa_r
        };
        let char_count = alph
            .iter()
            .map(|c| s.iter().filter(|&x| x == c).count())
            .collect_vec();
        let char_start = alph
            .iter()
            .map(|c| s.iter().filter(|&x| x < c).count())
            .collect_vec();
        let occ = (0..alph.len())
            .map(|ci| {
                (0..=n)
                    .scan(0, |occ, j| {
                        let old = *occ;
                        if j < n && s2[sa[j] + n - 1] == alph[ci] {
                            *occ += 1;
                        }
                        Some(old)
                    })
                    .collect_vec()
            })
            .collect_vec();
        let occ_r = (0..alph.len())
            .map(|ci| {
                (0..=n)
                    .scan(0, |occ, j| {
                        let old = *occ;
                        if j < n && s2[sa_r[j]] == alph[ci] {
                            *occ += 1;
                        }
                        Some(old)
                    })
                    .collect_vec()
            })
            .collect_vec();

        BWT {
            s,
            q,
            s2,
            alph,
            char_count,
            char_start,
            sa,
            occ,
            sa_r,
            occ_r,
            //j_begin_end,
        }
    }

    fn states(&self) -> Vec<State> {
        let mut v = vec![
            Init,
            LeftSA(0),
            LeftSA(1),
            LeftSA(2),
            RightSA(0),
            RightSA(1),
            RightSA(2),
            BothSA,
            CharCounts,
            LeftOcc,
            RightOcc,
        ];
        for i in 0..s_stats(self.s).0 {
            v.push(Equivalence(i));
        }

        v.push(Pause);
        let ql = self.q.len();
        for i in 1..ql {
            for qs in [
                PreviousDone,
                HighlightChar,
                HighlightMatches,
                EquivalenceFirst,
                CountFirst,
                SmallerCountFirst,
                ExtendFirst,
                SmallerWindowSecond,
                EquivalenceSecond,
                CountSecond,
                ComputeSecond,
                ExtendStartSecond,
                ExtendEndSecond,
            ] {
                v.push(Query(i, qs));
            }
        }
        // Show the last frame for a bit longer in the gif.
        v.push(Query(ql, PreviousDone));
        v.push(Query(ql, PreviousDone));
        v.push(Query(ql, PreviousDone));
        v.push(Query(ql, PreviousDone));
        v
    }

    fn query_ranges(&self, q: &[u8]) -> (Range<usize>, Range<usize>) {
        let n = self.s.len();
        let j_begin = (0..n)
            .find_position(|&j| q <= &self.s2[self.sa[j]..])
            .unwrap_or((n, n))
            .1;
        let j_end = (j_begin..n)
            .find_position(|&j| q != &self.s2[self.sa[j]..self.sa[j] + q.len()])
            .unwrap_or((n, n))
            .1;
        let j_begin_r = (0..n)
            .find_position(|&j| rev(q) <= rev(&self.s2[..n + self.sa_r[j]]))
            .unwrap_or((n, n))
            .1;
        let j_end_r = (j_begin_r..n)
            .find_position(|&j| q != &self.s2[n + self.sa_r[j] - q.len()..n + self.sa_r[j]])
            .unwrap_or((n, n))
            .1;
        ((j_begin..j_end), (j_begin_r..j_end_r))
    }

    fn draw(&self, state: State, canvas: &mut Canvas) {
        canvas.clear();
        draw_background(canvas);

        let s = self.s;
        let n = s.len();

        // Positioning

        // Top left of S at the top.
        let ps = Pos(4, 1);

        let plabel = ps.down(1).right(2);

        // SA.
        let psa = ps.down(2);
        // First entry of j column
        let cj = psa.left(4);
        // First entry of A column
        let ca = psa.left(3);

        let pfirst = psa;
        let plast = psa.left(1);

        // Reverse SA
        let psa_r = psa.right(n);
        // First entry of j column
        let cj_r = psa_r.right(3);
        // First entry of A column
        let ca_r = psa_r.right(2);

        let pfirst_r = psa_r.left(1);
        let plast_r = psa_r;

        // Count array
        let pcnt = ps + Pos(n + 6, 0);
        let pcnt_r = pcnt.right(self.alph.len() + 1);
        // Count array labels
        let rsigma = pcnt.up(1);
        let rsigma_r = pcnt_r.up(1);

        // Occ array
        let pocc = pcnt.down(2);
        let pocc_r = pcnt_r.down(2);

        // Query string
        let pq = psa.down(n + 2);
        let pq_r = psa_r.down(n + 2);
        // Query s
        let pqs = pq.down(1);
        let pqs_r = pq_r.down(1).left(1);
        // Query t
        let pqt = pq.down(2);
        let pqt_r = pq_r.down(2).left(1);
        // Column for s and t
        let cst = plast.left(1);
        let cst_r = plast_r.right(1);

        // Static data

        // 1. Draw input
        draw_string_with_labels(ps, s, |i| to_c(s[i] == '$' as u8), canvas);

        if state == Init {
            draw_text(plabel, "Input string S.", canvas);
            present(canvas);
            return;
        }

        // 2. Fwd
        if let RightSA(_) = state {
        } else {
            draw_label(cj.up(1), "j", canvas);
            draw_label(ca.up(1), "A", canvas);
            draw_label(pfirst.up(1), "F", canvas);

            // Number of chars drawn.
            // Last column copied to first.
            let l = match state {
                LeftSA(0) => n,
                LeftSA(1) => n,
                _ => n / 2 - 1,
            };

            // Label for last column?
            if l == n {
                draw_label(plast.right(n).up(1), "L", canvas);
            }
            // Draw SA.
            for j in 0..n {
                let i = self.sa[j];
                draw_label(cj.down(j), &j.to_string(), canvas);
                draw_label(ca.down(j), &i.to_string(), canvas);
                draw_string(
                    psa.down(j),
                    &self.s2[i..i + l],
                    |idx| to_c(idx == 0 || idx == n - 1),
                    canvas,
                );
            }

            if state == LeftSA(0) {
                draw_text(plabel, "Forward suffix array", canvas);
                present(canvas);
                return;
            }

            // Draw moved L column.
            draw_label(plast.up(1), "L", canvas);
            for j in 0..n {
                let i = self.sa[j];
                draw_char_box(plast.down(j), self.s2[i + n - 1], SOFT_HIGHLIGHT, canvas);
            }
            if state == LeftSA(1) {
                draw_text(plabel, "Move last column before first", canvas);
                present(canvas);
                return;
            }
            if state == LeftSA(2) {
                draw_text(plabel, "Only show prefixes of the array", canvas);
                present(canvas);
                return;
            }
        }

        // 3. Reverse SA
        {
            draw_label(cj_r.up(1), "j", canvas);
            draw_label(ca_r.up(1), "Ar", canvas);
            draw_label(pfirst_r.up(1), "Fr", canvas);

            // Number of chars drawn.
            // Last column copied to first.
            let l = match state {
                RightSA(0) => n,
                RightSA(1) => n,
                _ => n / 2 - 1,
            };

            // Label for last column?
            if l == n {
                draw_label(plast_r.left(n).up(1), "Lr", canvas);
            }
            // Draw SA.
            for j in 0..n {
                let i = self.sa_r[j];
                draw_label(cj_r.down(j), &j.to_string(), canvas);
                draw_label(ca_r.down(j), &i.to_string(), canvas);
                draw_string(
                    psa_r.down(j).left(l),
                    &self.s2[i + n - l..i + n],
                    |idx| to_c(l - 1 - idx == 0 || l - 1 - idx == n - 1),
                    canvas,
                );
            }

            if state == RightSA(0) {
                draw_text(plabel, "Reverse suffix array", canvas);
                present(canvas);
                return;
            }

            // Draw moved L column.
            draw_label(plast_r.up(1), "Lr", canvas);
            for j in 0..n {
                let i = self.sa_r[j];
                draw_char_box(plast_r.down(j), self.s2[i + n], SOFT_HIGHLIGHT, canvas);
            }
            if state == RightSA(1) {
                draw_text(plabel, "Move last column after first", canvas);
                present(canvas);
                return;
            }
            if state == RightSA(2) {
                draw_text(plabel, "Only show suffixes of the array", canvas);
                present(canvas);
                return;
            }
        }
        if state == BothSA {
            draw_text(plabel, "Forward & Reverse suffix array", canvas);
            present(canvas);
            return;
        }

        // 6. character counts
        {
            draw_label(rsigma.left(1), "σ", canvas);
            draw_label(pcnt.left(1), "C", canvas);

            for (i, &c) in self.alph.iter().enumerate() {
                let count = self.char_start[i];
                draw_label(rsigma.right(i), &to_label(c), canvas);
                draw_label(pcnt.right(i), &count.to_string(), canvas);
                draw_highlight_box(pfirst.down(count), 1, 0, Color::RED, canvas);
                draw_highlight_box(pfirst_r.down(count), 1, 0, Color::RED, canvas);
            }
            draw_highlight_box(pfirst.down(n), 1, 0, Color::RED, canvas);
            draw_highlight_box(pfirst_r.down(n), 1, 0, Color::RED, canvas);

            // When RightOcc is shown, duplicate this.
            if state >= RightOcc {
                for (i, &c) in self.alph.iter().enumerate() {
                    let count = self.char_start[i];
                    draw_label(rsigma_r.right(i), &to_label(c), canvas);
                    draw_label(pcnt_r.right(i), &count.to_string(), canvas);
                }
            }

            if state == CharCounts {
                draw_highlight_box(pfirst, 1, n, Color::RED, canvas);
                draw_highlight_box(pfirst_r, 1, n, Color::RED, canvas);
                draw_highlight_box(rsigma, self.alph.len(), 2, Color::RED, canvas);
                draw_text(
                    plabel,
                    "Count number of smaller characters for each c",
                    canvas,
                );
                present(canvas);
                return;
            }
        }

        // 7. Occurrences
        {
            draw_label(pocc.left(1).up(1), "Occ", canvas);
            for i in 0..self.alph.len() {
                for j in 0..=n {
                    draw_label(pocc.right(i).down(j), &self.occ[i][j].to_string(), canvas);
                }
            }

            if state == LeftOcc {
                draw_highlight_box(pocc, self.alph.len(), n + 1, Color::BLUE, canvas);
                draw_highlight_box(plast, 1, n, Color::BLUE, canvas);
                draw_text(plabel, "Count occurrences for L", canvas);
                present(canvas);
                return;
            }

            draw_label(pocc_r.left(1).up(1), "Occr", canvas);
            for i in 0..self.alph.len() {
                for j in 0..=n {
                    draw_label(
                        pocc_r.right(i).down(j),
                        &self.occ_r[i][j].to_string(),
                        canvas,
                    );
                }
            }

            if state == RightOcc {
                draw_highlight_box(pocc_r, self.alph.len(), n + 1, Color::BLUE, canvas);
                draw_highlight_box(plast_r, 1, n, Color::BLUE, canvas);
                draw_text(plabel, "Count occurrences for Lr", canvas);
                present(canvas);
                return;
            }
        };

        if let Equivalence(c) = state {
            let s = self.char_start[c];
            let l = self.char_count[c];
            draw_highlight_box(plast.down(s), 1, l, Color::RED, canvas);
            draw_highlight_box(pfirst_r.left(1).down(s), 1, l, Color::RED, canvas);

            draw_highlight_box(plast_r.down(s), 1, l, Color::BLUE, canvas);
            draw_highlight_box(pfirst.right(1).down(s), 1, l, Color::BLUE, canvas);

            draw_text(plabel, "Sets of chars before/after c are the same.", canvas);
            present(canvas);
            return;
        }
        if state == Pause {
            draw_text(plabel, "Ready for querying", canvas);
            present(canvas);
            return;
        }

        // Draw query
        // We start in the middle, with the first char already matched.
        // Then extend left to the start.
        // Then extend right to the end.

        if let Query(step, qs) = state {
            // Current state
            let q = self.q;
            let ql = q.len();
            let mid = (ql + 1) / 2;

            let done = if step <= mid {
                mid - step..mid
            } else {
                0..step
            };
            let (range, range_r) = self.query_ranges(&q[done.clone()]);

            // Draw query
            if pq.0 >= done.start + 1 {
                draw_label(pq.left(max(2, done.start + 1)), "Q", canvas);
            }
            draw_string(pq.left(done.start), q, |_| DEFAULT, canvas);
            draw_string(pq_r.left(done.end), q, |_| DEFAULT, canvas);

            // Draw current range
            draw_label(pqs.left(2), "s", canvas);
            draw_label(pqt.left(2), "t", canvas);
            draw_label(pqs_r.right(2), "sr", canvas);
            draw_label(pqt_r.right(2), "tr", canvas);
            draw_label(pqs, &range.start.to_string(), canvas);
            draw_label(pqt, &range.end.to_string(), canvas);
            draw_label(pqs_r, &range_r.start.to_string(), canvas);
            draw_label(pqs_r.down(1), &range_r.end.to_string(), canvas);

            // start/end labels in row j
            if range.len() > 0 {
                draw_label(cst.down(range.start), "s", canvas);
                draw_label(cst.down(range.end), "t", canvas);

                draw_label(cst_r.down(range_r.start), "sr", canvas);
                draw_label(cst_r.down(range_r.end), "tr", canvas);
            } else {
                draw_label(cst.down(range.start), "s/t", canvas);
                draw_label(cst_r.down(range_r.start), "sr/tr", canvas);
            }

            // Draw cyan shading for matched chars.
            draw_string(pq, &q[done.clone()], |_| Color::CYAN, canvas);
            draw_string(
                pq_r.left(done.len()),
                &q[done.clone()],
                |_| Color::CYAN,
                canvas,
            );
            for j in range.clone() {
                draw_string(
                    psa.down(j),
                    &self.s2[self.sa[j]..self.sa[j] + step],
                    |_| Color::CYAN,
                    canvas,
                );
            }
            for j in range_r.clone() {
                draw_string(
                    psa_r.left(done.len()).down(j),
                    &self.s2[n + self.sa_r[j] - step..n + self.sa_r[j]],
                    |_| Color::CYAN,
                    canvas,
                );
            }

            // Black lines around current range.
            draw_highlight_box(plast.down(range.start), n / 2, 0, Color::BLACK, canvas);
            draw_highlight_box(plast.down(range.end), n / 2, 0, Color::BLACK, canvas);
            draw_highlight_box(
                plast_r.down(range_r.start).left(n / 2 - 1),
                n / 2,
                0,
                Color::BLACK,
                canvas,
            );
            draw_highlight_box(
                plast_r.down(range_r.end).left(n / 2 - 1),
                n / 2,
                0,
                Color::BLACK,
                canvas,
            );
            draw_highlight_box(pqs, 1, 2, Color::BLACK, canvas);
            draw_highlight_box(pqs_r, 1, 2, Color::BLACK, canvas);

            if qs == PreviousDone {
                if step == 1 {
                    draw_text(
                        plabel,
                        "Start with the range of the first character",
                        canvas,
                    );
                } else {
                    draw_text(plabel, "Matching done", canvas);
                }
                present(canvas);
                return;
            }

            // Position of the next char to be matched.
            let extend_left = step < mid;
            let extend_idx = if extend_left { mid - step - 1 } else { step };
            let next = q[extend_idx];
            let ci = self.alph.iter().position(|&cc| cc == next).unwrap();
            let pnext = pq.left(done.start).right(extend_idx);
            let pnext_r = pq_r.left(done.end).right(extend_idx);
            let cnext = psa.left(done.start).right(extend_idx);
            let cnext_r = psa_r.left(done.end).right(extend_idx);

            // Next First range
            let (ss, tt);
            if extend_left {
                ss = self.char_start[ci] + self.occ[ci][range.start];
                tt = self.char_start[ci] + self.occ[ci][range.end];
            } else {
                ss = self.char_start[ci] + self.occ_r[ci][range_r.start];
                tt = self.char_start[ci] + self.occ_r[ci][range_r.end];
            }

            // Highlight the next character.
            draw_label(pnext.up(1), "c", canvas);
            draw_char_box(pnext, next, NEXT_CHAR, canvas);
            draw_label(pnext_r.up(1), "c", canvas);
            draw_char_box(pnext_r, next, NEXT_CHAR, canvas);

            if qs == HighlightChar {
                if extend_left {
                    draw_text(plabel, "Extend query on the left", canvas);
                } else {
                    draw_text(plabel, "Extend query on the right", canvas);
                }
                present(canvas);
                return;
            }

            // Highlight matches for the next char.
            for j in range.clone() {
                if self.s2[self.sa[j] + n + extend_idx - done.start] == next {
                    draw_char_box(
                        psa.down(j).right(extend_idx).left(done.start),
                        next,
                        NEXT_CHAR,
                        canvas,
                    );
                }
            }
            for j in range_r.clone() {
                if self.s2[self.sa_r[j] + n + extend_idx - done.end] == next {
                    draw_char_box(
                        psa_r.down(j).right(extend_idx).left(done.end),
                        next,
                        NEXT_CHAR,
                        canvas,
                    );
                }
            }
            if qs == HighlightMatches {
                draw_text(plabel, "Matches for next char", canvas);
                present(canvas);
                return;
            }

            // DO THE FIRST EXTEND, IE THE NORMAL BWT ONE.

            // the occurrences of the next char to process.
            if extend_left {
                draw_highlight_box(plast.down(range.start), 1, range.len(), Color::BLUE, canvas);
                for j in ss..tt {
                    draw_char_box(pfirst.down(j), next, NEXT_CHAR, canvas);
                }
            } else {
                draw_highlight_box(
                    plast_r.down(range_r.start),
                    1,
                    range_r.len(),
                    Color::BLUE,
                    canvas,
                );
                for j in ss..tt {
                    draw_char_box(pfirst_r.down(j), next, NEXT_CHAR, canvas);
                }
            }
            if qs == EquivalenceFirst {
                if extend_left {
                    draw_text(
                        plabel,
                        "Fwd: matches in L correspond to matches in F",
                        canvas,
                    );
                } else {
                    draw_text(
                        plabel,
                        "Rev: matches in Lr correspond to matches in Fr",
                        canvas,
                    );
                }
                present(canvas);
                return;
            }

            // the occurrences of the next char to process.
            if extend_left {
                draw_highlight(pocc.right(ci).down(range.start), Color::BLUE, canvas);
                draw_highlight(pocc.right(ci).down(range.end), Color::BLUE, canvas);
            } else {
                draw_highlight(pocc_r.right(ci).down(range_r.start), Color::BLUE, canvas);
                draw_highlight(pocc_r.right(ci).down(range_r.end), Color::BLUE, canvas);
            }
            if qs == CountFirst {
                if extend_left {
                    draw_text(plabel, "Fwd: positions of matches", canvas);
                } else {
                    draw_text(plabel, "Rev: positions of matches", canvas);
                }
                present(canvas);
                return;
            }

            // the occurrences of the next char to process.
            if extend_left {
                draw_highlight_box(rsigma.right(ci), 1, 2, Color::BLUE, canvas);
                draw_label(rsigma.right(ci).down(2), "+", canvas);
            } else {
                draw_highlight_box(rsigma_r.right(ci), 1, 2, Color::BLUE, canvas);
                draw_label(rsigma_r.right(ci).down(2), "+", canvas);
            }
            if qs == SmallerCountFirst {
                if extend_left {
                    draw_text(plabel, "Fwd: number of smaller chars", canvas);
                } else {
                    draw_text(plabel, "Rev: number of smaller chars", canvas);
                }
                present(canvas);
                return;
            }

            // Draw new computed numbers
            if extend_left {
                draw_label(pnext.down(1), &ss.to_string(), canvas);
                draw_label(pnext.down(2), &tt.to_string(), canvas);
                draw_highlight_box(pnext.down(1), 1, 2, Color::BLUE, canvas);
                draw_highlight_box(pfirst.down(ss), 1, tt - ss, Color::BLUE, canvas);
            } else {
                draw_label(pnext_r.down(1), &ss.to_string(), canvas);
                draw_label(pnext_r.down(2), &tt.to_string(), canvas);
                draw_highlight_box(pnext_r.down(1), 1, 2, Color::BLUE, canvas);
                draw_highlight_box(pfirst_r.down(ss), 1, tt - ss, Color::BLUE, canvas);
            }
            if qs == ExtendFirst {
                if extend_left {
                    draw_text(plabel, "Fwd: #smaller + match-positions", canvas);
                } else {
                    draw_text(plabel, "Rev: #smaller + match-positions", canvas);
                }
                present(canvas);
                return;
            }

            // DO THE SECOND EXTEND, IE APPEND ON THE BACK.

            // the occurrences of the next char to process.
            let mut cnt = 0;
            if extend_left {
                for ci in 0..ci {
                    cnt += self.occ[ci][range.end] - self.occ[ci][range.start];
                }
                draw_highlight_box(cnext_r.down(range_r.start), 1, cnt, Color::RED, canvas);
            } else {
                for ci in 0..ci {
                    cnt += self.occ_r[ci][range_r.end] - self.occ_r[ci][range_r.start];
                }
                draw_highlight_box(cnext.down(range.start), 1, cnt, Color::RED, canvas);
            }
            if qs == SmallerWindowSecond {
                if extend_left {
                    draw_text(plabel, "Rev: range shrinks; skip chars < c", canvas);
                } else {
                    draw_text(plabel, "Fwd: range shrinks; skip chars < c", canvas);
                }
                present(canvas);
                return;
            }

            // Equivalence to left column
            if extend_left {
                draw_highlight_box(plast.down(range.start), 1, range.len(), Color::RED, canvas);
            } else {
                draw_highlight_box(
                    plast_r.down(range_r.start),
                    1,
                    range_r.len(),
                    Color::RED,
                    canvas,
                );
            }
            if qs == EquivalenceSecond {
                if extend_left {
                    draw_text(plabel, "Rev: equal to #{<c} in forward range", canvas);
                } else {
                    draw_text(plabel, "Fwd: equal to #{<c} in reverse range", canvas);
                }
                present(canvas);
                return;
            }

            // the occurrences of the next char to process.
            if extend_left {
                draw_highlight_box(rsigma, ci, 1, Color::RED, canvas);
                draw_highlight_box(pocc.down(range.start), ci, 1, Color::RED, canvas);
                draw_highlight_box(pocc.down(range.end), ci, 1, Color::RED, canvas);
            } else {
                draw_highlight_box(rsigma_r, ci, 1, Color::RED, canvas);
                draw_highlight_box(pocc_r.down(range_r.start), ci, 1, Color::RED, canvas);
                draw_highlight_box(pocc_r.down(range_r.end), ci, 1, Color::RED, canvas);
            }
            if qs == CountSecond {
                if extend_left {
                    draw_text(plabel, "Rev: Char counts before/after fwd range", canvas);
                } else {
                    draw_text(plabel, "Fwd: Char counts before/after rev range", canvas);
                }
                present(canvas);
                return;
            }

            // the occurrences of the next char to process.
            if extend_left {
                for ci in 0..ci {
                    let d = self.occ[ci][range.end] - self.occ[ci][range.start];
                    draw_label_color(
                        pocc.down(n + 3).right(ci),
                        &d.to_string(),
                        Color::RED,
                        canvas,
                    );
                }
                draw_label_color(
                    pocc.down(n + 3).left(1),
                    &cnt.to_string(),
                    Color::RED,
                    canvas,
                );
                draw_highlight(pocc.down(n + 3).left(1), Color::RED, canvas);
            } else {
                for ci in 0..ci {
                    let d = self.occ_r[ci][range_r.end] - self.occ_r[ci][range_r.start];
                    draw_label_color(
                        pocc_r.down(n + 3).right(ci),
                        &d.to_string(),
                        Color::RED,
                        canvas,
                    );
                }
                draw_label_color(
                    pocc_r.down(n + 3).left(1),
                    &cnt.to_string(),
                    Color::RED,
                    canvas,
                );
                draw_highlight(pocc_r.down(n + 3).left(1), Color::RED, canvas);
            }
            if qs == ComputeSecond {
                if extend_left {
                    draw_text(plabel, "Rev: Total count #{<c} in forward range", canvas);
                } else {
                    draw_text(plabel, "Fwd: Total count #{<c} in reverse range", canvas);
                }
                present(canvas);
                return;
            }

            // Compute start of range
            let ss;
            if extend_left {
                draw_highlight(pqs_r, Color::RED, canvas);
                ss = range_r.start + cnt;
                draw_label(pnext_r.down(1), &ss.to_string(), canvas);
                draw_highlight(pnext_r.down(1), Color::RED, canvas);
            } else {
                draw_highlight(pqs, Color::RED, canvas);
                ss = range.start + cnt;
                draw_label(pnext.down(1), &ss.to_string(), canvas);
                draw_highlight(pnext.down(1), Color::RED, canvas);
            }
            if qs == ExtendStartSecond {
                if extend_left {
                    draw_text(plabel, "Add #{<c} to current start", canvas);
                } else {
                    draw_text(plabel, "Add #{<c} to current start", canvas);
                }
                present(canvas);
                return;
            }

            // Compute end of range
            let mut cnt = 0;
            if extend_left {
                for ci in 0..=ci {
                    let d = self.occ[ci][range.end] - self.occ[ci][range.start];
                    cnt += d;
                    draw_label_color(
                        pocc.down(n + 4).right(ci),
                        &d.to_string(),
                        Color::RED,
                        canvas,
                    );
                }
                draw_label_color(
                    pocc.down(n + 4).left(1),
                    &cnt.to_string(),
                    Color::RED,
                    canvas,
                );

                draw_highlight(pocc.down(n + 4).left(1), Color::RED, canvas);
                draw_highlight(pqt_r, Color::RED, canvas);
                draw_label(pnext_r.down(2), &tt.to_string(), canvas);
                draw_highlight(pnext_r.down(2), Color::RED, canvas);
            } else {
                for ci in 0..=ci {
                    let d = self.occ_r[ci][range_r.end] - self.occ_r[ci][range_r.start];
                    cnt += d;
                    draw_label_color(
                        pocc_r.down(n + 4).right(ci),
                        &d.to_string(),
                        Color::RED,
                        canvas,
                    );
                }
                draw_label_color(
                    pocc_r.down(n + 4).left(1),
                    &cnt.to_string(),
                    Color::RED,
                    canvas,
                );
                draw_highlight(pocc_r.down(n + 4).left(1), Color::RED, canvas);
                draw_highlight(pqt, Color::RED, canvas);
                draw_label(pnext.down(2), &tt.to_string(), canvas);
                draw_highlight(pnext.down(2), Color::RED, canvas);
            }
            if qs == ExtendEndSecond {
                if extend_left {
                    draw_text(plabel, "Add #{≤c} to current start", canvas);
                } else {
                    draw_text(plabel, "Add #{≤c} to current start", canvas);
                }
                present(canvas);
                return;
            }
        }
    }
}

fn main() {
    let mut s = ARGS
        .input
        .clone()
        .unwrap_or("GTCCCGATGTCATGTCAGGA".to_owned());
    s.push('$');
    let s = s.as_bytes();

    let q = ARGS.query.clone().unwrap_or("GTCC".to_string());
    let q = q.as_bytes();

    let ref mut canvas = canvas(s.len() + 12 + 2 * s_stats(s).0, s.len() + 9);
    let bwt = BWT::new(s, q);
    for state in bwt.states() {
        bwt.draw(state, canvas);
    }
    wait_for_end();
}
