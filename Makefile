.PHONY: test rust-test go-test build run proto

rust-test:
	cd rust && cargo test && cargo clippy --all-targets -- -D warnings

go-test:
	cd go && go test ./... && go vet ./...

test: rust-test go-test

build:
	cd go && go build -o bin/grounded-guardrails ./cmd/server

run: build
	./go/bin/grounded-guardrails

proto:
	protoc -I proto \
	  --go_out=go/gen/guardrails/v1 --go_opt=paths=source_relative \
	  --go-grpc_out=go/gen/guardrails/v1 --go-grpc_opt=paths=source_relative \
	  proto/guardrails.proto
