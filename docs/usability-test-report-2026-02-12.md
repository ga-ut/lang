# Gaut 사용성 테스트 보고서 (2026-02-12)

## 1) 목적
- 현재 저장소 상태에서 Gaut를 "언어/도구"로 실제 사용할 때의 기본 흐름이 동작하는지 검증.
- 변경 범위를 최소화하기 위해 코드 기능 변경 없이, 실제 사용자 시나리오 중심으로 재현 가능한 테스트 절차를 문서화.

## 2) 테스트 범위
- CLI 인터프리트 실행
- C 코드 생성 + 네이티브 바이너리 빌드/실행
- 예제 스모크 스크립트
- self-host 루프(비엄격/엄격)
- 워크스페이스 회귀 테스트

## 3) 환경
- 작업 디렉터리: `/workspace/lang`
- 실행 도구: Cargo, Bash 스크립트, clang(스크립트 경유)

## 4) 테스트 시나리오 및 결과

### 시나리오 A: CLI로 예제 직접 실행
- 명령: `cargo run -p cli -- examples/hello.gaut`
- 기대: 인터프리터 결과가 정상 출력됨
- 결과: `Str("hello world")` 출력, 성공

### 시나리오 B: C 생성 + 빌드 + 실행(사용자 관점 end-to-end)
- 명령: `cargo run -p cli -- --emit-c target/usability/hello.c --build target/usability/hello examples/hello.gaut && ./target/usability/hello`
- 기대: C 산출물과 실행 파일 생성 후 실행 성공
- 결과: `hello world` 출력, 성공

### 시나리오 C: 예제 스모크 스크립트
- 명령: `./scripts/run_examples.sh`
- 기대: interp/cgen 관련 스모크 통과
- 결과: 스크립트 완료(hello/calc/record 및 cgen 샘플 빌드 경로 확인), 성공

### 시나리오 D: self-host 루프(비엄격)
- 명령: `SELF_HOST_COMPILER=1 ./scripts/self_host.sh`
- 기대: stage 루프가 완료되고 산출물 생성
- 결과: 스크립트 성공 종료.
  - stage0/stage1 및 stage1/stage2 해시 불일치 메시지는 출력됨.
  - 비엄격 모드이므로 정보성 경고로 처리되고 전체 흐름은 완료됨.

### 시나리오 E: self-host 루프(엄격)
- 명령: `SELF_HOST_COMPILER=1 SELF_HOST_COMPILER_STRICT=1 ./scripts/self_host.sh`
- 기대: 고정점이면 성공, 아니면 실패
- 결과: **실패(의도된 게이트 확인)**
  - stage0/stage1 C 해시 불일치로 스크립트 exit 1
  - 현재 컴파일러 self-host 고정점 미달 상태를 재현

### 시나리오 F: 회귀 테스트(영향 범위 확인)
- 명령: `cargo test`
- 기대: 워크스페이스 테스트 통과
- 결과: 전체 테스트 통과

## 5) 사용성 관점 결론
- 현재 사용자 관점의 기본 개발 플로우(예제 실행, C 생성/빌드, 스모크 실행)는 동작한다.
- 다만 self-host strict 고정점은 아직 미달이며, 이는 컴파일러 구현이 완전 스텁/부분 이식 단계임을 의미한다.

## 6) 후속 권장 작업(피처 단위)
1. `compiler/ast.gaut` 실체화(최소 subset)
2. `compiler/parser.gaut` 토크나이저 + precedence parser 이식
3. `compiler/typecheck.gaut` 최소 타입 규칙 이식
4. `compiler/cgen.gaut` Rust cgen subset 동등화
5. strict self-host 고정점(`SELF_HOST_COMPILER_STRICT=1`) 통과를 완료 기준으로 승격

## 7) 재현 가이드(요약)
아래 순서대로 실행하면 본 보고서 결과를 재현할 수 있다.
1. `cargo run -p cli -- examples/hello.gaut`
2. `cargo run -p cli -- --emit-c target/usability/hello.c --build target/usability/hello examples/hello.gaut && ./target/usability/hello`
3. `./scripts/run_examples.sh`
4. `SELF_HOST_COMPILER=1 ./scripts/self_host.sh`
5. `SELF_HOST_COMPILER=1 SELF_HOST_COMPILER_STRICT=1 ./scripts/self_host.sh`
6. `cargo test`
