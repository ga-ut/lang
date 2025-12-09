# Gaut 언어 예제 실행 가이드

`examples/*.gaut` 파일들을 타입체크/인터프리트/간단 C 코드 생성으로 확인하는 방법을 정리했습니다.

## 1) 빌드/테스트 준비

```bash
cargo test
```

- 워크스페이스 전체 테스트가 통과하면 파서/타입체커/인터프리터/C 트랜스파일러 스켈레톤이 정상 동작합니다.
- 네트워크가 제한된 환경이면 의존성 다운로드가 먼저 필요합니다.

## 2) 인터프리터로 예제 실행 (Rust 테스트 기반)

현재 인터프리터는 Rust 테스트로 예제를 실행합니다. 새 예제를 추가하려면 `crates/interp/src/lib.rs`의 테스트를 참고하세요.

예제 확인:

```bash
cargo test -p interp
```

포함된 예제:
- `examples/calc.gaut` : 기본 계산
- `examples/record.gaut` : 구조체/참조/이동
- `examples/hello.gaut` : 전역 문자열 결합

## 3) CLI와 C 백엔드

- 인터프리터 실행: `cargo run -p cli -- examples/hello.gaut`
- C 코드 생성: `cargo run -p cli -- --emit-c /tmp/hello.c examples/hello.gaut`
- C 코드 생성 후 빌드: `cargo run -p cli -- --emit-c /tmp/hello.c --build /tmp/hello examples/hello.gaut`
- 환경 변수: `GAUT_STD_DIR`(표준 모듈 경로), `GAUT_RUNTIME_C_DIR`(C 런타임 위치). `clang -std=gnu11`로 `runtime/c/runtime.{c,h}`를 함께 빌드합니다.
- 문자열/바이트 결합은 함수/블록 아레나(`GAUT_DEFAULT_ARENA_CAP`)에서 할당하며, 함수 반환 시에는 힙으로 승격해 수명을 보장합니다.

## 4) Self-host 스모크/결정성 체크

`./scripts/self_host.sh`는 예제(`hello`, `calc`, `record`)를 대상으로
1) 두 번 C 코드를 생성해 sha256이 동일한지 확인하고,
2) `--build`로 바이너리를 만든 뒤 실행까지 진행합니다. 산출물은 `target/self_host/`에 저장됩니다.

## 5) std/네트워크 예제

- 표준 스텁: `std/str.gaut`, `std/bytes.gaut`, `std/net.gaut` (net은 타입 시그니처만, 런타임 연결 미구현)
- TCP 예제: `examples/tcp_echo.gaut`는 네트워크 래퍼가 실제로 연결된 후 사용할 수 있습니다.

## 6) 새 .gaut 파일 작성/실행 팁

1. `.gaut` 확장자로 저장합니다.
2. `import foo`는 `foo.gaut`를 같은 디렉터리나 std 경로에서 찾습니다.
3. 실행하려면:
   - CLI: `cargo run -p cli -- my.gaut` (또는 `--emit-c/--build`),
   - 또는 Rust 테스트에 예제를 추가해 `cargo test -p interp`로 실행 결과를 확인합니다.

## 7) 주의사항

- 현재 IO/네트워크는 스텁 수준입니다. 실제 출력/소켓 동작은 런타임과 언어를 더 연결해야 합니다.
- 경고: parser의 Token 가시성과 interp의 `IndexMap::remove` 경고가 남아있지만 기능에는 영향 없습니다.

## 7) CLI 사용법 및 배포

### 실행
- 로컬 빌드 후 실행: `cargo run -p cli -- examples/hello.gaut`
- C 코드만 뽑기: `cargo run -p cli -- --emit-c /tmp/hello.c examples/hello.gaut`
- C 코드 빌드까지: `cargo run -p cli -- --emit-c /tmp/hello.c --build /tmp/hello examples/hello.gaut`
- 설치 후 실행: `gaut examples/hello.gaut` (PATH에 등록 시)
- std 경로 변경: `GAUT_STD_DIR=/path/to/std gaut myfile.gaut`
- C 런타임 경로 변경: `GAUT_RUNTIME_C_DIR=/path/to/runtime/c gaut --emit-c ...`

### 빌드/설치
- 릴리스 빌드: `cargo build -p cli --release` → `target/release/gaut`
- PATH 등록: `ln -sf $(pwd)/target/release/gaut /usr/local/bin/gaut` (또는 PATH 내 디렉터리에 복사)
- Cargo 설치: `cargo install --path crates/cli` → `~/.cargo/bin/gaut`

### 배포(바이너리 묶음)
- `cargo build -p cli --release` 후 `target/release/gaut`와 `std/` 디렉터리를 함께 tar/zip으로 패키징
- 사용자는 압축 해제 후 `gaut` 실행, `std/`는 실행 파일과 동일 루트에 두면 기본 경로로 인식
