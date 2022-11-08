use crate::{canvas::*, grid::*, viz::Viz};
use itertools::Itertools;

#[derive(Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
pub enum State {
    Init,
    Rotations,
    SortedRotations,
    FirstLast,
    // One state per occurrence of the most frequent character
    LfMap(usize),
    // One state per char
    Counts(usize),
    // Finalize the char counting
    CountsDone,
    // Occurrences, one per char
    Occ(usize),
    // Finalize occurrences
    OccDone,
    // Query, one per char +1 to wrap
    Query(usize),
}

fn s_stats(s: &[u8]) -> (usize, usize) {
    let mut cnts = [0; 256];
    for &c in s {
        cnts[c as usize] += 1;
    }
    let num_chars = cnts.iter().filter(|&&x| x > 0).count();
    let max_char_cnt = *cnts.iter().max().unwrap();
    (num_chars, max_char_cnt)
}

const SMALL_COLOUR: Color = GREEN;
const LARGE_COLOUR: Color = (240, 240, 240);

fn to_c(condition: bool) -> Color {
    if condition {
        SMALL_COLOUR
    } else {
        LARGE_COLOUR
    }
}

pub struct BWT {
    s: Vec<u8>,
    q: Vec<u8>,
    n: usize,
    ql: usize,
    s2: Vec<u8>,
    alph: Vec<u8>,
    char_count: Vec<usize>,
    char_start: Vec<usize>,
    sa: Vec<usize>,
    occ: Vec<Vec<i32>>,
    j_begin_end: Vec<(usize, usize)>,

    pub states: Vec<State>,
}

impl BWT {
    pub fn new(s: Vec<u8>, q: Vec<u8>) -> Self {
        let n = s.len();
        let alph = {
            let mut alph = s.to_vec();
            alph.sort();
            alph.dedup();
            alph
        };

        let mut s2 = s.to_vec();
        s2.extend(&s);
        let sa = {
            let mut sa = (0..n).collect_vec();
            sa.sort_by_key(|i| &s[*i..]);
            sa
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

        let ql = q.len();
        let j_begin_end = (0..=ql)
            .map(|step| {
                let j_begin = (0..n)
                    .find_position(|&j| &q[ql - step..] <= &s2[sa[j]..])
                    .unwrap_or((n, n))
                    .1;
                let j_end = (j_begin..n)
                    .find_position(|&j| &q[ql - step..] != &s2[sa[j]..sa[j] + step])
                    .unwrap_or((n, n))
                    .1;
                (j_begin, j_end)
            })
            .collect_vec();

        use State::*;
        let mut states = vec![Init, Rotations, SortedRotations, FirstLast];

        let (num_chars, max_char_cnt) = s_stats(&s);

        for i in 0..max_char_cnt {
            states.push(LfMap(i));
        }
        for i in 0..num_chars {
            states.push(Counts(i));
        }
        states.push(CountsDone);
        for i in 0..num_chars {
            states.push(Occ(i));
        }
        states.push(OccDone);
        for i in 0..=ql {
            states.push(Query(i));
        }

        BWT {
            s,
            q,
            n,
            ql,
            s2,
            alph,
            char_count,
            char_start,
            sa,
            occ,
            j_begin_end,
            states,
        }
    }
}

impl Viz for BWT {
    fn canvas_size(&self) -> (usize, usize) {
        let n = self.s.len();
        canvas_size(n + 7 + s_stats(&self.s).0, n + 8)
    }

    fn num_states(&self) -> usize {
        self.states.len()
    }

    fn draw(&self, state: usize, canvas: &mut CanvasBox) -> bool {
        let state = self.states[state];
        draw_background(canvas);

        let s = &self.s;
        let n = self.n;
        let ql = self.ql;

        // Positioning

        // Top left of S at the top.
        let ps = Pos(3, 1);

        let plabel = ps.down(1).right(2);
        let pbotlabel = ps.down(1).right(2);

        // Top left of SA.
        let psa = ps.down(2);
        // First entry of j column
        let cj = psa.left(3);
        // First entry of A column
        let ca = psa.left(2);

        let pfirst = psa;
        let plast = psa.right(n - 1);

        // Count array
        let pcnt = ps + Pos(n + 2, 0);
        // Count array labels
        let rsigma = pcnt.up(1);

        // Occ array
        let pocc = pcnt.down(2);

        // Query string
        let pq = psa.down(n + 1);
        // Query s
        let pqs = pq.down(1);
        // Query t
        let pqt = pq.down(2);
        // Column for s and t
        let cst = psa.left(1);
        // Past-the-end of remainder of query
        let pqend = pq.right(n);

        // Static data

        // 1. Draw input
        draw_string_with_labels(ps, &s, |i| to_c(s[i] == '$' as u8), canvas);

        if state == State::Init {
            draw_text(plabel, "Input string S.", canvas);
            return true;
        }

        // 2. Draw rotations

        draw_label(cj.up(1), "j", canvas);
        draw_label(ca.up(1), "A", canvas);
        if state == State::Rotations {
            for j in 0..n {
                let i = j;
                draw_label(cj.down(j), &j.to_string(), canvas);
                draw_string(
                    psa.down(j),
                    &self.s2[i..i + n],
                    |idx| to_c(self.s2[i + idx] == '$' as u8),
                    canvas,
                );
            }
            draw_text(plabel, "Write down rotations of S.", canvas);
            return true;
        }

        // 3. Draw sorted rotations
        if state == State::SortedRotations {
            for j in 0..n {
                let i = self.sa[j];
                draw_label(cj.down(j), &j.to_string(), canvas);
                draw_label(ca.down(j), &i.to_string(), canvas);
                draw_string(
                    psa.down(j),
                    &self.s2[i..i + n],
                    |idx| to_c(self.s2[i + idx] == '$' as u8),
                    canvas,
                );
            }
            draw_text(plabel, "Sort rotations via the suffix array of S.", canvas);
            return true;
        }

        draw_label(pfirst.up(1), "F", canvas);
        draw_label(plast.up(1), "L", canvas);
        for j in 0..n {
            let i = self.sa[j];
            draw_label(cj.down(j), &j.to_string(), canvas);
            draw_label(ca.down(j), &i.to_string(), canvas);
            draw_string(
                psa.down(j),
                &self.s2[i..i + n],
                |idx| to_c(idx == 0 || idx == n - 1),
                canvas,
            );
        }

        if state == State::FirstLast {
            draw_text(plabel, "Store the first and last column.", canvas);
            return true;
        }

        // 5. Last-to-first correspondence
        if let State::LfMap(k) = state {
            let ci = self.char_count.iter().position_max().unwrap();
            let cnt = self.char_count[ci];
            let start_pos = self.char_start[ci];
            assert!(k < cnt);

            // Index in alph of max char.
            // Draw a box around char ci.
            draw_highlight_box(psa.down(start_pos), 2, cnt, RED, canvas);

            for i in 0..=k {
                let start_row = start_pos + i;
                let idx = self.sa[start_row];
                let shift_row = self.sa.iter().find_position(|&&x| x == idx + 1).unwrap().0;
                // Blue box around sa[start_row], sa[target_row], 2nd char in start row, 1st char in target row.
                draw_highlight(psa.down(start_row), BLACK, canvas);
                if i == k {
                    draw_highlight(psa.down(start_row).right(1), BLUE, canvas);
                    draw_highlight(ca.down(start_row), BLUE, canvas);
                }
                draw_highlight(plast.down(shift_row), BLACK, canvas);
                if i == k {
                    draw_highlight(psa.down(shift_row), BLUE, canvas);
                    draw_highlight(ca.down(shift_row), BLUE, canvas);
                    draw_highlight(ps.right(idx), BLACK, canvas);
                    draw_highlight(ps.right(idx + 1), BLUE, canvas);
                }
            }
            draw_text(
                plabel,
                "For each char, L and F are sorted the same.",
                canvas,
            );
            return true;
        }

        // 6. character counts
        {
            draw_label(rsigma.left(1), "σ", canvas);
            draw_label(pcnt.left(1), "C", canvas);

            // Draw chars 0..=k
            let (k, iscount) = match state {
                State::Counts(step) => (step, true),
                State::CountsDone => (self.alph.len(), true),
                _ => (self.alph.len(), false),
            };
            for (i, &c) in self.alph.iter().enumerate().take(k + 1) {
                let count = self.char_start[i];
                draw_label(rsigma.right(i), &to_label(c), canvas);
                draw_label(pcnt.right(i), &count.to_string(), canvas);
                if k < self.alph.len() && i == k {
                    draw_highlight(cj.down(count), RED, canvas);
                    draw_highlight(psa.down(count), RED, canvas);
                }
                draw_highlight_box(psa.down(count), 1, 0, RED, canvas);
            }
            if k == self.alph.len() {
                draw_highlight_box(pfirst.down(n), 1, 0, RED, canvas);
            }
            if let State::Counts(_) = state {
                draw_highlight_box(rsigma.right(k), 1, 2, RED, canvas);
            }

            if iscount {
                draw_text(
                    plabel,
                    "Count number of smaller characters for each c",
                    canvas,
                );
                return true;
            }
        }

        // 7. Occurrences
        {
            // Draw the first k+1 chars
            draw_label(pocc.left(1).up(1), "Occ", canvas);
            let (k, isocc) = match state {
                State::Occ(step) => (step, true),
                State::OccDone => (self.alph.len(), true),
                _ => (self.alph.len(), false),
            };
            for (i, &c) in self.alph.iter().enumerate().take(k + 1) {
                if k < self.alph.len() && i == k {
                    draw_highlight(rsigma.right(k), BLUE, canvas);
                }
                for j in 0..=n {
                    draw_label(pocc.right(i).down(j), &self.occ[i][j].to_string(), canvas);
                    if j < n && self.s2[self.sa[j] + n - 1] == c {
                        if k < self.alph.len() && i == k {
                            draw_highlight(plast.down(j), BLUE, canvas);
                            draw_highlight(pocc.right(k).down(j + 1), BLUE, canvas);
                        }
                    }
                }
            }

            if isocc {
                draw_text(
                    plabel,
                    "Count number of occurrences of c in L at pos < j",
                    canvas,
                );
                return true;
            }
        };

        // Draw query
        {
            let q = &self.q;
            let step = match state {
                State::Query(step) => step,
                _ => unreachable!(),
            };

            let q_done = &q[q.len() - step..];
            let q_remaining = &q[..q.len() - step];
            draw_label(pq.left(1), "Q", canvas);
            draw_string(pq, q_done, |i| to_c(i == 0), canvas);
            draw_string(
                pqend.left(q.len() - step),
                q_remaining,
                |_| LARGE_COLOUR,
                canvas,
            );
            let (j_begin, j_end) = self.j_begin_end[step];
            // start/end indices at the bottom
            draw_label(pqs.left(1), "s", canvas);
            draw_label(pqt.left(1), "t", canvas);
            for s in 0..=step {
                draw_label(
                    pqs.right(s),
                    &self.j_begin_end[step - s].0.to_string(),
                    canvas,
                );
                draw_label(
                    pqt.right(s),
                    &self.j_begin_end[step - s].1.to_string(),
                    canvas,
                );
            }

            // cyan shading for matched chars.
            for j in j_begin..j_end {
                draw_string(
                    psa.down(j),
                    &self.s2[self.sa[j]..self.sa[j] + step],
                    |_| CYAN,
                    canvas,
                );
            }

            // start/end labels in row j
            draw_highlight_box(psa.down(j_begin), n, j_end - j_begin, BLACK, canvas);
            if j_begin < j_end {
                draw_label(cst.down(j_begin), "s", canvas);
                draw_label(cst.down(j_end), "t", canvas);
                draw_highlight_box(psa.down(j_begin), n, j_end - j_begin, BLACK, canvas);
            } else {
                draw_label(cst.down(j_begin), "s/t", canvas);
                draw_highlight_box(psa.down(j_begin), n, j_end - j_begin, RED, canvas);
            }
            draw_highlight_box(pqs, 1, 2, BLACK, canvas);

            // the occurrences of the next char to process.
            if step < ql {
                let c = q[ql - 1 - step];
                draw_label(pqend.left(1).up(1), "c", canvas);
                let ci = self.alph.iter().position(|&cc| cc == c).unwrap();
                draw_highlight(pqend.left(1), BLUE, canvas);
                draw_highlight_box(rsigma.right(ci), 1, 2, BLUE, canvas);
                draw_label(rsigma.right(ci).down(2), "+", canvas);
                if j_begin < j_end {
                    draw_highlight_box(plast.down(j_begin), 1, j_end - j_begin, BLUE, canvas);
                }

                draw_highlight(pocc.right(ci).down(j_begin), BLUE, canvas);
                draw_highlight(pocc.right(ci).down(j_end), BLUE, canvas);
            }

            // NOTE: We save each query step twice since this is a tricky part and
            // queries are typically short.
            if step == 0 {
                draw_text(
                    pbotlabel,
                    "Initialize the query range as the full text",
                    canvas,
                );
                return true;
            }
            if step < ql {
                draw_text(pbotlabel, "Update s[i-1] = C[c] + Occ[c][s[i]]", canvas);
                return true;
            }
            return true;
        }
    }
}
