use itertools::Itertools;
use sdl2::pixels::Color;
use suffix_array_construction::*;

const SMALL_COLOUR: Color = Color::GREEN;
//const LARGE_COLOUR: Color = Color::RGB(244, 113, 116);
const LARGE_COLOUR: Color = Color::RGB(240, 240, 240);

fn to_c(condition: bool) -> Color {
    if condition {
        SMALL_COLOUR
    } else {
        LARGE_COLOUR
    }
}

fn main() {
    let mut s = ARGS
        .input
        .clone()
        .unwrap_or("GTCCCGATGTCATGTCAGGA".to_owned());
    s.push('$');

    let s = s.as_bytes();
    let n = s.len();

    let alph = {
        let mut alph = s.to_vec();
        alph.sort();
        alph.dedup();
        alph
    };

    let w = n + alph.len() + 7;
    let h = n + 8;

    let ref mut canvas = canvas(w, h);

    // 1. Draw input
    let input = |canvas: &mut Canvas| {
        canvas.clear();
        draw_background(canvas);
        draw_string_with_labels(3, 1, s, |i| to_c(s[i] == '$' as u8), canvas);
    };
    {
        input(canvas);
        draw_text(5, 2, "Input string S.", canvas);
        wait_for_key(canvas);
    }

    // 2. Draw rotations
    input(canvas);
    draw_text(5, 2, "Write down rotations of S.", canvas);

    let mut s2 = s.to_vec();
    s2.extend(s);

    {
        draw_label(0, 2, "j", canvas);
        draw_label(1, 2, "A", canvas);
        for j in 0..n {
            let i = j;
            draw_label(0, 3 + j, &j.to_string(), canvas);
            draw_string(
                3,
                3 + j,
                &s2[i..i + n],
                |idx| to_c(s2[i + idx] == '$' as u8),
                canvas,
            );
        }
        wait_for_key(canvas);
    }

    // 3. Draw sorted rotations
    {
        input(canvas);
        draw_text(5, 2, "Sort rotations via the suffix array of S.", canvas);

        let mut sa: Vec<_> = (0..n).collect();
        sa.sort_by_key(|i| &s[*i..]);

        draw_label(0, 2, "j", canvas);
        draw_label(1, 2, "A", canvas);
        for j in 0..n {
            let i = sa[j];
            draw_label(0, 3 + j, &j.to_string(), canvas);
            draw_label(1, 3 + j, &i.to_string(), canvas);
            draw_string(
                3,
                3 + j,
                &s2[i..i + n],
                |idx| to_c(s2[i + idx] == '$' as u8),
                canvas,
            );
        }
        wait_for_key(canvas);
    }

    // 4. Draw sorted rotations, with first and last column highlighted
    let sa = {
        let mut sa = (0..n).collect_vec();
        sa.sort_by_key(|i| &s[*i..]);
        sa
    };
    let sorted_rotations = |canvas: &mut Canvas| {
        input(canvas);

        draw_label(0, 2, "j", canvas);
        draw_label(1, 2, "A", canvas);
        draw_label(3, 2, "F", canvas);
        draw_label(3 + n - 1, 2, "L", canvas);
        for j in 0..n {
            let i = sa[j];
            draw_label(0, 3 + j, &j.to_string(), canvas);
            draw_label(1, 3 + j, &i.to_string(), canvas);
            draw_string(
                3,
                3 + j,
                &s2[i..i + n],
                |idx| to_c(idx == 0 || idx == n - 1),
                canvas,
            );
        }
    };

    {
        sorted_rotations(canvas);
        draw_text(5, 2, "Store the first and last column.", canvas);
        wait_for_key(canvas);
    }

    // 5. Last-to-first correspondence
    let char_count = alph
        .iter()
        .map(|c| s.iter().filter(|&x| x == c).count())
        .collect_vec();
    let char_start = alph
        .iter()
        .map(|c| s.iter().filter(|&x| x < c).count())
        .collect_vec();
    {
        // Show the mapping for the first k+1 occurrences of ci'th character.
        let ltf = |ci: usize, k: usize, step: usize, canvas: &mut Canvas| {
            sorted_rotations(canvas);
            // Draw a box around char ci.
            let cnt = char_count[ci];
            let start_pos = char_start[ci];
            draw_highlight_box(3, 3 + start_pos, 2, cnt, Color::RED, canvas);

            if k < cnt {
                for i in 0..=k {
                    let start_row = start_pos + i;
                    let idx = sa[start_row];
                    let shift_row = sa.iter().find_position(|&&x| x == idx + 1).unwrap().0;
                    // Blue box around sa[start_row], sa[target_row], 2nd char in start row, 1st char in target row.
                    draw_highlight(3, 3 + start_row, Color::BLACK, canvas);
                    if i == k {
                        draw_highlight(4, 3 + start_row, Color::BLUE, canvas);
                        draw_highlight(1, 3 + start_row, Color::BLUE, canvas);
                    }
                    if i == k && step == 0 {
                        draw_highlight(3 + idx, 1, Color::BLACK, canvas);
                        return;
                    }
                    draw_highlight(3 + n - 1, 3 + shift_row, Color::BLACK, canvas);
                    if i == k {
                        draw_highlight(3, 3 + shift_row, Color::BLUE, canvas);
                        draw_highlight(1, 3 + shift_row, Color::BLUE, canvas);
                        draw_highlight(3 + idx, 1, Color::BLACK, canvas);
                        draw_highlight(3 + idx + 1, 1, Color::BLUE, canvas);
                    }
                }
            }

            // if k == cnt {
            //     for i in 0..cnt {
            //         let start_row = start_pos + i;
            //         let idx = sa[start_row];
            //         let shift_row = sa.iter().find_position(|&&x| x == idx + 1).unwrap().0;
            //         // Blue box around sa[start_row], sa[target_row], 2nd char in start row, 1st char in target row.
            //         draw_highlight(3, 3 + start_row, Color::BLACK, canvas);
            //         draw_highlight(3 + n - 1, 3 + shift_row, Color::BLACK, canvas);
            //     }
            // }
        };

        // Show ltf mapping 1-by-1 for the most common character.
        // Then, show it again per char.

        // Index in alph of max char.
        let ci = char_count.iter().position_max().unwrap();
        for k in 0..char_count[ci] {
            for step in 0..2 {
                ltf(ci, k, step, canvas);
                draw_text(5, 2, "For each char, L and F are sorted the same.", canvas);
                wait_for_key(canvas);
            }
        }
        //let ltf = |canvas: &mut Canvas| ltf(alph.len(), canvas);
    }

    // 6. character counts
    let char_counts = |k: usize, canvas: &mut Canvas| {
        sorted_rotations(canvas);
        draw_label(n + 4, 0, "σ", canvas);
        draw_label(n + 4, 1, "C(σ)", canvas);
        for (i, &c) in alph.iter().enumerate().take(k + 1) {
            let count = char_start[i];
            draw_label(n + 6 + i, 0, &to_label(c), canvas);
            draw_label(n + 6 + i, 1, &count.to_string(), canvas);
            if k == alph.len() || i == k {
                draw_highlight(0, 3 + count, Color::RED, canvas);
                draw_highlight(3, 3 + count, Color::RED, canvas);
            }
        }
        if k < alph.len() {
            draw_highlight_box(n + 6 + k, 0, 1, 2, Color::RED, canvas);
        } else {
            draw_highlight_box(n + 6, 0, alph.len(), 2, Color::RED, canvas);
        }
    };

    for k in 0..=alph.len() {
        char_counts(k, canvas);
        draw_text(
            5,
            2,
            "Count number of smaller characters for each c",
            canvas,
        );
        wait_for_key(canvas);
    }

    let char_counts = |canvas: &mut Canvas| char_counts(alph.len(), canvas);

    // 7. Occurrences
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
    let occurrences = |k: usize, canvas: &mut Canvas| {
        char_counts(canvas);
        draw_label(n + 4, 0, "σ", canvas);
        draw_label(n + 4, 3, "Occ", canvas);
        for (i, &c) in alph.iter().enumerate().take(k + 1) {
            if k < alph.len() && i == k {
                draw_highlight(6 + n + k, 0, Color::BLUE, canvas);
            }
            for j in 0..=n {
                draw_label(n + 6 + i, 3 + j, &occ[i][j].to_string(), canvas);
                if j < n && s2[sa[j] + n - 1] == c {
                    if k < alph.len() && i == k {
                        draw_highlight(2 + n, 3 + j, Color::BLUE, canvas);
                        draw_highlight(6 + n + k, 3 + j + 1, Color::BLUE, canvas);
                    }
                }
            }
        }

        // if k == alph.len() {
        //     draw_highlight_box(2 + n, 3, 1, n, Color::BLUE, canvas);
        //     draw_highlight_box(6 + n, 3, alph.len(), n + 1, Color::BLUE, canvas);
        // }
    };

    for k in 0..=alph.len() {
        occurrences(k, canvas);
        draw_text(
            5,
            2,
            "Count number of occurrences of c in L at pos < j",
            canvas,
        );
        wait_for_key(canvas);
    }
    let occurrences = |canvas: &mut Canvas| occurrences(alph.len(), canvas);

    // Draw query
    let q = ARGS.query.clone().unwrap_or("GTCC".to_string());
    let q = q.as_bytes();
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

    let query = |step: usize, canvas: &mut Canvas| {
        let q_done = &q[q.len() - step..];
        let q_remaining = &q[..q.len() - step];
        occurrences(canvas);
        draw_label(2, n + 4, "Q", canvas);
        draw_string(3, n + 4, q_done, |i| to_c(i == 0), canvas);
        if step == 0 {
            draw_text(
                5,
                3 + n,
                "Initialize the query range as the full text",
                canvas,
            );
        } else if step < ql {
            draw_text(5, 3 + n, "Update s[i-1] = C[c] + Occ[c][s[i]]", canvas);
        }
        draw_string(
            3 + n - q.len() + step,
            n + 4,
            q_remaining,
            |_| LARGE_COLOUR,
            canvas,
        );
        let (j_begin, j_end) = j_begin_end[step];
        // start/end indices at the bottom
        draw_label(2, n + 5, "s", canvas);
        draw_label(2, n + 6, "t", canvas);
        for s in 0..=step {
            draw_label(3 + s, n + 5, &j_begin_end[step - s].0.to_string(), canvas);
            draw_label(3 + s, n + 6, &j_begin_end[step - s].1.to_string(), canvas);
        }

        // start/end labels in row j
        draw_highlight_box(3, 3 + j_begin, n, j_end - j_begin, Color::BLACK, canvas);
        if j_begin < j_end {
            draw_label(2, 3 + j_begin, "s", canvas);
            draw_label(2, 3 + j_end, "t", canvas);
            draw_highlight_box(3, 3 + j_begin, n, j_end - j_begin, Color::BLACK, canvas);
        } else {
            draw_label(2, 3 + j_begin, "s/t", canvas);
            draw_highlight_box(3, 3 + j_begin, n, j_end - j_begin, Color::RED, canvas);
        }
        draw_highlight_box(3, n + 5, 1, 2, Color::BLACK, canvas);

        // the occurrences of the next char to process.
        if step < ql {
            let c = q[ql - 1 - step];
            draw_label(2 + n, 3 + n, "c", canvas);
            let ci = alph.iter().position(|&cc| cc == c).unwrap();
            draw_highlight(2 + n, 4 + n, Color::BLUE, canvas);
            draw_highlight_box(n + 6 + ci, 0, 1, 2, Color::BLUE, canvas);
            draw_label(n + 6 + ci, 2, "+", canvas);
            if j_begin < j_end {
                draw_highlight_box(2 + n, 3 + j_begin, 1, j_end - j_begin, Color::BLUE, canvas);
            }

            draw_highlight(n + 6 + ci, 3 + j_begin, Color::BLUE, canvas);
            draw_highlight(n + 6 + ci, 3 + j_end, Color::BLUE, canvas);
        }

        wait_for_key(canvas);
    };
    for step in 0..=q.len() {
        query(step, canvas);
    }

    let query = |canvas: &mut Canvas| query(ql, canvas);
    query(canvas);
    wait_for_end();
}
