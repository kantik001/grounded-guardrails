# syntax=docker/dockerfile:1
FROM golang:1.25-bookworm AS build
WORKDIR /src
COPY go/go.mod go/go.sum ./
RUN go mod download
COPY go/ ./
RUN CGO_ENABLED=0 go build -o /grounded-guardrails ./cmd/server

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=build /grounded-guardrails /grounded-guardrails
ENV GRPC_PORT=50052
EXPOSE 50052
USER nonroot:nonroot
ENTRYPOINT ["/grounded-guardrails"]
