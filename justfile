
set shell := [ "nu.exe", "-c" ]

watch:
	watchexec -e rs,toml just git

run:
	RUST_LOG=info cargo make run

git:
	cargo fmt
	git add -A
	git commit -m "save"
	
push:
	git push
