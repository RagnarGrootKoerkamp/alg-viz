use crate::{
    canvas::{Canvas, BLUE, GREEN, RED, WHITE},
    grid::*,
    viz::Viz,
};

const SMALL_COLOUR: (u8, u8, u8) = GREEN;
const LARGE_COLOUR: (u8, u8, u8) = (244, 113, 116);

#[derive(Ord, PartialEq, PartialOrd, Eq, Clone, Copy, Debug)]
pub enum RowState {
    Step0,
    Step1,
    Step2,
}

#[derive(Ord, PartialEq, PartialOrd, Eq, Clone, Copy, Debug)]
pub enum State {
    Init,
    Row(usize, RowState),
    End,
}

pub struct SA {
    s: Vec<u8>,
    states: Vec<State>,
}

impl SA {
    pub fn new(s: Vec<u8>) -> Self {
        let mut states = vec![State::Init];
        for j in 0..s.len() {
            states.extend([
                State::Row(j, RowState::Step0),
                State::Row(j, RowState::Step1),
                State::Row(j, RowState::Step2),
            ]);
        }
        states.push(State::End);
        Self { s, states }
    }
}

impl Viz for SA {
    fn canvas_size(&self) -> (usize, usize) {
        (self.s.len() + 4, self.s.len() + 4)
    }

    fn num_states(&self) -> usize {
        self.states.len()
    }

    fn draw(&self, state: usize, canvas: &mut Box<dyn Canvas>) -> bool {
        let state = self.states[state];
        canvas.fill_background(WHITE);

        let s = &self.s;
        let n = s.len();

        let is_small = |i| i == n - 1 || s[i..] < s[i + 1..];
        let is_small_color = |i| {
            if is_small(i) {
                SMALL_COLOUR
            } else {
                LARGE_COLOUR
            }
        };

        // Positioning

        // Top left of SA.
        let psa = Pos(3, 3);
        // First entry of j column
        let cj = psa.left(3);
        // First entry of SA column
        let csa = psa.left(2);

        // Top left of S at the top.
        let ps = Pos(3, 1);
        // The first label of S.
        let ri = ps.up(1);

        // Draw the string at the top.
        draw_string_with_labels(ps, s, is_small_color, canvas);

        let mut buckets = [0 as usize; 256];
        for &c in s.iter() {
            buckets[c as usize] += 1;
        }
        for i in 0..254 {
            buckets[i + 1] += buckets[i];
        }

        let mut final_sa: Vec<usize> = (0..n).collect();
        final_sa.sort_by_key(|&i| &s[i..]);

        let mut sa = vec![None; n];
        for j in 0..n {
            if is_small(final_sa[j]) {
                sa[j] = Some(final_sa[j]);
            }
        }

        // Compute the SA as far as needed
        let mut new_j = 0;
        match state {
            State::Init => {}
            State::Row(rj, _) => {
                for j in 0..=rj {
                    let i = sa[j].unwrap();
                    if i == 0 || is_small(i - 1) {
                        continue;
                    }
                    let c = s[i - 1];
                    new_j = (buckets[c as usize - 1]..buckets[c as usize])
                        .find(|&j| sa[j].is_none())
                        .unwrap();
                    sa[new_j] = Some(i - 1);
                }
            }
            State::End => sa = final_sa.iter().map(|&i| Some(i)).collect(),
        }

        if let State::Row(j, rs) = state {
            let i = sa[j].unwrap();
            let skip = i == 0 || is_small(i - 1);
            // Do not show the SA entry yet.
            if rs < RowState::Step2 && !skip {
                sa[new_j] = None;
            }
        }

        // Draw the SA
        draw_label(cj.up(1), "j", canvas);
        draw_label(csa.up(1), "SA", canvas);
        for j in 0..n {
            draw_label(cj.down(j), &j.to_string(), canvas);
            if let Some(i) = sa[j] {
                draw_label(csa.down(j), &i.to_string(), canvas);
                draw_string(psa.down(j), &s[i..], |i2| is_small_color(i + i2), canvas);
            } else {
                // Find the first letter for this bucket
                let bucket = buckets
                    .iter()
                    .enumerate()
                    .find(|&(_, &cnt)| cnt > j)
                    .unwrap()
                    .0 as u8;
                draw_label(csa.down(j), &'-'.to_string(), canvas);
                draw_char_box(psa.down(j), bucket, LARGE_COLOUR, canvas);
            }
        }

        // If needed, draw highlight boxes
        if let State::Row(j, rs) = state {
            let i = sa[j].unwrap();
            let skip = i == 0 || is_small(i - 1);

            // highlight the current index i and the one before
            draw_highlight(csa.down(j), RED, canvas);
            draw_highlight(ri.right(i), RED, canvas);
            if i > 0 {
                draw_highlight(ri.right(i - 1), BLUE, canvas);
            }

            if rs > RowState::Step0 {
                if skip {
                    return false;
                }
                draw_highlight(ps.right(i - 1), BLUE, canvas);
                draw_highlight(csa.down(new_j), BLUE, canvas);
            }
        }
        true
    }
}
