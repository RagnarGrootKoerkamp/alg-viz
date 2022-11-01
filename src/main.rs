use sdl2::pixels::Color;
use suffix_array_construction::*;

const SMALL_COLOUR: Color = Color::GREEN;
const LARGE_COLOUR: Color = Color::RGB(244, 113, 116);

#[derive(Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
enum RowState {
    Step0,
    Step1,
    Step2,
}

#[derive(Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
enum State {
    Init,
    Row(usize, RowState),
    End,
}

fn states(n: usize) -> Vec<State> {
    let mut v = vec![State::Init];
    for j in 0..n {
        v.extend([
            State::Row(j, RowState::Step0),
            State::Row(j, RowState::Step1),
            State::Row(j, RowState::Step2),
        ]);
    }
    v.push(State::End);
    v
}

fn draw(s: &[u8], state: State, canvas: &mut Canvas) {
    canvas.clear();
    draw_background(canvas);

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
        // Do not show the SA entry yet.
        if rs < RowState::Step2 && !skip {
            sa[new_j] = None;
        }

        // highlight the current index i and the one before
        draw_highlight(csa.down(j), Color::RED, canvas);
        draw_highlight(ri.right(i), Color::RED, canvas);
        if i > 0 {
            draw_highlight(ri.right(i - 1), Color::BLUE, canvas);
        }

        if rs > RowState::Step0 {
            if skip {
                return;
            }
            draw_highlight(ps.right(i - 1), Color::BLUE, canvas);
            draw_highlight(csa.down(new_j), Color::BLUE, canvas);
        }
    }
    present(canvas);
}

fn main() {
    let mut s = ARGS
        .input
        .clone()
        .unwrap_or("GTCCCGATGTCATGTCAGGA".to_owned());
    s.push('$');
    let s = s.as_bytes();
    let n = s.len();

    let ref mut canvas = canvas(n + 4, n + 4);
    let states = states(n);
    for state in states {
        draw(s, state, canvas);
    }
    wait_for_end();
}
