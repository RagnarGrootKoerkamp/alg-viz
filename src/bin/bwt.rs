use itertools::Itertools;
use sdl2::pixels::Color;
use suffix_array_construction::*;

#[derive(Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
enum State {
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

fn states(s: &[u8], q: &[u8]) -> Vec<State> {
    use State::*;
    let mut v = vec![Init, Rotations, SortedRotations, FirstLast];

    let (num_chars, max_char_cnt) = s_stats(s);

    for i in 0..max_char_cnt {
        v.push(LfMap(i));
    }
    for i in 0..num_chars {
        v.push(Counts(i));
    }
    v.push(CountsDone);
    for i in 0..num_chars {
        v.push(Occ(i));
    }
    v.push(OccDone);
    for i in 0..=q.len() {
        v.push(Query(i));
    }
    // Show the last frame for a bit longer in the gif.
    v.push(State::Query(q.len()));
    v.push(State::Query(q.len()));
    v.push(State::Query(q.len()));
    v.push(State::Query(q.len()));
    v
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

fn draw(s: &[u8], q: &[u8], state: State, canvas: &mut Canvas) {
    canvas.clear();
    draw_background(canvas);

    let n = s.len();

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
    // End of remainder of query
    let pqend = pq.right(n - 1);

    let alph = {
        let mut alph = s.to_vec();
        alph.sort();
        alph.dedup();
        alph
    };

    // Static data
    let mut s2 = s.to_vec();
    s2.extend(s);
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

    // 1. Draw input
    draw_string_with_labels(ps, s, |i| to_c(s[i] == '$' as u8), canvas);

    if state == State::Init {
        draw_text(plabel, "Input string S.", canvas);
        present(canvas);
        return;
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
                &s2[i..i + n],
                |idx| to_c(s2[i + idx] == '$' as u8),
                canvas,
            );
        }
        draw_text(plabel, "Write down rotations of S.", canvas);
        present(canvas);
        return;
    }

    // 3. Draw sorted rotations
    if state == State::SortedRotations {
        for j in 0..n {
            let i = sa[j];
            draw_label(cj.down(j), &j.to_string(), canvas);
            draw_label(ca.down(j), &i.to_string(), canvas);
            draw_string(
                psa.down(j),
                &s2[i..i + n],
                |idx| to_c(s2[i + idx] == '$' as u8),
                canvas,
            );
        }
        draw_text(plabel, "Sort rotations via the suffix array of S.", canvas);
        present(canvas);
        return;
    }

    draw_label(cj.up(1), "j", canvas);
    draw_label(ca.up(1), "A", canvas);
    draw_label(pfirst.up(1), "F", canvas);
    draw_label(plast.up(1), "L", canvas);
    for j in 0..n {
        let i = sa[j];
        draw_label(cj.down(j), &j.to_string(), canvas);
        draw_label(ca.down(j), &i.to_string(), canvas);
        draw_string(
            psa.down(j),
            &s2[i..i + n],
            |idx| to_c(idx == 0 || idx == n - 1),
            canvas,
        );
    }

    if state == State::FirstLast {
        draw_text(plabel, "Store the first and last column.", canvas);
        present(canvas);
        return;
    }

    // 5. Last-to-first correspondence
    if let State::LfMap(k) = state {
        let ci = char_count.iter().position_max().unwrap();
        let cnt = char_count[ci];
        let start_pos = char_start[ci];
        assert!(k < cnt);

        // Index in alph of max char.
        // Draw a box around char ci.
        draw_highlight_box(psa.down(start_pos), 2, cnt, Color::RED, canvas);

        for i in 0..=k {
            let start_row = start_pos + i;
            let idx = sa[start_row];
            let shift_row = sa.iter().find_position(|&&x| x == idx + 1).unwrap().0;
            // Blue box around sa[start_row], sa[target_row], 2nd char in start row, 1st char in target row.
            draw_highlight(psa.down(start_row), Color::BLACK, canvas);
            if i == k {
                draw_highlight(psa.down(start_row).right(1), Color::BLUE, canvas);
                draw_highlight(ca.down(start_row), Color::BLUE, canvas);
            }
            draw_highlight(plast.down(shift_row), Color::BLACK, canvas);
            if i == k {
                draw_highlight(psa.down(shift_row), Color::BLUE, canvas);
                draw_highlight(ca.down(shift_row), Color::BLUE, canvas);
                draw_highlight(ps.right(idx), Color::BLACK, canvas);
                draw_highlight(ps.right(idx + 1), Color::BLUE, canvas);
            }
        }
        draw_text(
            plabel,
            "For each char, L and F are sorted the same.",
            canvas,
        );
        present(canvas);
        return;
    }

    // 6. character counts
    {
        draw_label(rsigma.left(1), "σ", canvas);
        draw_label(pcnt.left(1), "C(σ)", canvas);

        // Draw chars 0..=k
        let (k, iscount) = match state {
            State::Counts(step) => (step, true),
            State::CountsDone => (alph.len(), true),
            _ => (alph.len(), false),
        };
        for (i, &c) in alph.iter().enumerate().take(k + 1) {
            let count = char_start[i];
            draw_label(rsigma.right(i), &to_label(c), canvas);
            draw_label(pcnt.right(i), &count.to_string(), canvas);
            if k < alph.len() && i == k {
                draw_highlight(cj.down(count), Color::RED, canvas);
                draw_highlight(psa.down(count), Color::RED, canvas);
            }
            draw_highlight_box(psa.down(count), 1, 0, Color::RED, canvas);
        }
        if let State::Counts(_) = state {
            draw_highlight_box(rsigma.right(k), 1, 2, Color::RED, canvas);
        }

        if iscount {
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
        // Draw the first k+1 chars
        let (k, isocc) = match state {
            State::Occ(step) => (step, true),
            State::OccDone => (alph.len(), true),
            _ => (alph.len(), false),
        };
        for (i, &c) in alph.iter().enumerate().take(k + 1) {
            if k < alph.len() && i == k {
                draw_highlight(rsigma.right(k), Color::BLUE, canvas);
            }
            for j in 0..=n {
                draw_label(pocc.right(i).down(j), &occ[i][j].to_string(), canvas);
                if j < n && s2[sa[j] + n - 1] == c {
                    if k < alph.len() && i == k {
                        draw_highlight(plast.down(j), Color::BLUE, canvas);
                        draw_highlight(pocc.right(k).down(j + 1), Color::BLUE, canvas);
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
            present(canvas);
            return;
        }
    };

    // Draw query
    {
        let step = match state {
            State::Query(step) => step,
            _ => unreachable!(),
        };

        let q_done = &q[q.len() - step..];
        let q_remaining = &q[..q.len() - step];
        draw_label(pq.left(1), "Q", canvas);
        draw_string(pq, q_done, |i| to_c(i == 0), canvas);
        draw_string(
            pqend.right(1).left(q.len() - step),
            q_remaining,
            |_| LARGE_COLOUR,
            canvas,
        );
        let (j_begin, j_end) = j_begin_end[step];
        // start/end indices at the bottom
        draw_label(pqs.left(1), "s", canvas);
        draw_label(pqt.left(1), "t", canvas);
        for s in 0..=step {
            draw_label(pqs.right(s), &j_begin_end[step - s].0.to_string(), canvas);
            draw_label(pqt.right(s), &j_begin_end[step - s].1.to_string(), canvas);
        }

        // cyan shading for matched chars.
        for j in j_begin..j_end {
            draw_string(
                psa.down(j),
                &s2[sa[j]..sa[j] + step],
                |_| Color::CYAN,
                canvas,
            );
        }

        // start/end labels in row j
        draw_highlight_box(psa.down(j_begin), n, j_end - j_begin, Color::BLACK, canvas);
        if j_begin < j_end {
            draw_label(cst.down(j_begin), "s", canvas);
            draw_label(cst.down(j_end), "t", canvas);
            draw_highlight_box(psa.down(j_begin), n, j_end - j_begin, Color::BLACK, canvas);
        } else {
            draw_label(cst.down(j_begin), "s/t", canvas);
            draw_highlight_box(psa.down(j_begin), n, j_end - j_begin, Color::RED, canvas);
        }
        draw_highlight_box(pqs, 1, 2, Color::BLACK, canvas);

        // the occurrences of the next char to process.
        if step < ql {
            let c = q[ql - 1 - step];
            draw_label(pqend.up(1), "c", canvas);
            let ci = alph.iter().position(|&cc| cc == c).unwrap();
            draw_highlight(pqend, Color::BLUE, canvas);
            draw_highlight_box(rsigma.right(ci), 1, 2, Color::BLUE, canvas);
            draw_label(rsigma.right(ci).down(2), "+", canvas);
            if j_begin < j_end {
                draw_highlight_box(plast.down(j_begin), 1, j_end - j_begin, Color::BLUE, canvas);
            }

            draw_highlight(pocc.right(ci).down(j_begin), Color::BLUE, canvas);
            draw_highlight(pocc.right(ci).down(j_end), Color::BLUE, canvas);
        }

        // NOTE: We save each query step twice since this is a tricky part and
        // queries are typically short.
        if step == 0 {
            draw_text(
                pbotlabel,
                "Initialize the query range as the full text",
                canvas,
            );
        }
        if step < ql {
            draw_text(pbotlabel, "Update s[i-1] = C[c] + Occ[c][s[i]]", canvas);
        }
        save(canvas);
        present(canvas);
        return;
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

    let ref mut canvas = canvas(s.len() + 7 + s_stats(s).0, s.len() + 8);
    for state in states(s, q) {
        draw(s, q, state, canvas);
    }
    wait_for_end();
}
