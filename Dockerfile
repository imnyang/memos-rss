FROM oven/bun:alpine as build

RUN apk add --no-cache tzdata git

WORKDIR /app

# Copy dependency files first (improves layer caching)
COPY package.json bun.lock ./

# Install dependencies (cached until package files change)
RUN bun install --frozen-lockfile

# Copy application code
COPY . .

RUN bun run ./cli/git-commit-build.ts
RUN bun run build

FROM alpine:latest as runtime

RUN apk add --no-cache tzdata

ENV TZ=Asia/Seoul
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

WORKDIR /app

COPY --from=build /app/yanmang .
COPY --from=build /app/version .

CMD ["./yanmang"]