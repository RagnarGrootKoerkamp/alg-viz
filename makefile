all: img/suffix-array.gif img/bwt.gif

img/suffix-array/0.bmp: target/debug/alg-viz
	cargo run -- --save img/suffix-array

img/suffix-array.gif: img/suffix-array/0.bmp
	ffmpeg -y -framerate 0.3 -i img/suffix-array/%d.bmp -filter_complex "split[s0][s1];[s0]palettegen=max_colors=64[p];[s1][p]paletteuse=dither=bayer",fps=0.3 img/suffix-array.gif

img/bwt/0.bmp: target/debug/bwt
	cargo run --bin bwt -- --save img/bwt

img/bwt.gif: img/bwt/0.bmp
	ffmpeg -y -framerate 0.3 -i img/bwt/%d.bmp -filter_complex "split[s0][s1];[s0]palettegen=max_colors=64[p];[s1][p]paletteuse=dither=bayer",fps=0.3 img/bwt.gif


img/bibwt/0.bmp: target/debug/bibwt
	cargo run --bin bibwt -- --save img/bibwt

img/bibwt.gif: img/bibwt/0.bmp
	ffmpeg -y -framerate 0.3 -i img/bibwt/%d.bmp -filter_complex "split[s0][s1];[s0]palettegen=max_colors=64[p];[s1][p]paletteuse=dither=bayer",fps=0.3 img/bibwt.gif


wasm:
	wasm-pack build --target web --no-default-features --features wasm
prod:
	wasm-pack build --release --target web --no-default-features --features wasm

run: wasm
	python3 -m http.server
