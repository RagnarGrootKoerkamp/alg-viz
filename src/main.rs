use sdl2::pixels::Color;
use suffix_array_construction::*;

const SMALL_COLOUR: Color = Color::GREEN;
const LARGE_COLOUR: Color = Color::RGB(244, 113, 116);

fn main() {
    let mut s = ARGS
        .input
        .clone()
        .unwrap_or("GTCCCGATGTCATGTCAGGA".to_owned());
    s.push('$');
    let s = s.as_bytes();
    let n = s.len();

    let w = n + 4;
    let h = n + 5;

    let ref mut canvas = canvas(w, h);

    let is_small = |i| i == n - 1 || s[i..] < s[i + 1..];
    let is_small_color = |i| {
        if is_small(i) {
            SMALL_COLOUR
        } else {
            LARGE_COLOUR
        }
    };

    let draw_s = |canvas: &mut Canvas| {
        draw_string_with_labels(3, 1, s, is_small_color, canvas);
    };

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

    let draw_sa = |sa: &Vec<Option<usize>>, canvas: &mut Canvas| {
        draw_label(0, 3, "j", canvas);
        draw_label(1, 3, "SA", canvas);
        for j in 0..n {
            draw_label(0, 4 + j, &j.to_string(), canvas);
            if let Some(i) = sa[j] {
                draw_label(1, 4 + j, &i.to_string(), canvas);
                draw_string(3, 4 + j, &s[i..], |i2| is_small_color(i + i2), canvas);
            } else {
                // Find the first letter for this bucket
                let bucket = buckets
                    .iter()
                    .enumerate()
                    .find(|&(_, &cnt)| cnt > j)
                    .unwrap()
                    .0 as u8;
                draw_label(1, 4 + j, &'-'.to_string(), canvas);
                draw_char_box(3, 4 + j, bucket, LARGE_COLOUR, canvas);
            }
        }
    };

    draw_background(canvas);
    draw_s(canvas);
    draw_sa(&sa, canvas);
    wait_for_key(canvas);

    for j in 0..n {
        canvas.clear();
        draw_background(canvas);
        draw_s(canvas);
        draw_sa(&sa, canvas);

        // highlight the current index i and the one before
        let i = sa[j].unwrap();
        draw_highlight(1, 4 + j, Color::RED, canvas);
        draw_highlight(3 + i, 0, Color::RED, canvas);
        draw_highlight(3 + i - 1, 0, Color::BLUE, canvas);

        wait_for_key(canvas);

        if i == 0 || is_small(i - 1) {
            continue;
        }

        // copied from above
        {
            canvas.clear();
            draw_background(canvas);
            draw_s(canvas);
            draw_sa(&sa, canvas);

            // highlight the current index i and the one before
            let i = sa[j].unwrap();
            draw_highlight(1, 4 + j, Color::RED, canvas);
            draw_highlight(3 + i, 0, Color::RED, canvas);
            draw_highlight(3 + i - 1, 0, Color::BLUE, canvas);
        }

        // highlight the new character, and the first empty position in that bucket
        let c = s[i - 1];
        draw_highlight(3 + i - 1, 1, Color::BLUE, canvas);
        let new_j = (buckets[c as usize - 1]..buckets[c as usize])
            .find(|&j| sa[j].is_none())
            .unwrap();
        draw_highlight(1, 4 + new_j, Color::BLUE, canvas);

        wait_for_key(canvas);

        sa[new_j] = Some(i - 1);

        canvas.clear();
        draw_background(canvas);
        draw_s(canvas);
        draw_sa(&sa, canvas);

        wait_for_key(canvas);
    }

    canvas.clear();
    draw_background(canvas);
    draw_s(canvas);
    draw_sa(&sa, canvas);

    wait_for_key(canvas);
}
