목표: Gaut로 작성된 `compiler/`가 실제로 “입력 .gaut → C 생성(→ 선택적으로 clang 빌드)”까지 수행하고, `SELF_HOST_COMPILER_STRICT=1`에서 stage0→stage1→stage2 고정점을 통과한다.

진행 현황(2026-02-12)
- [x] (6) `compiler/main.gaut`의 TODO 제거: `--emit-c`/입력 인자 파싱 결과를 사용해 `read_file -> parse -> check -> emit -> write_file` 파이프라인 연결.
- [x] (3-준비) `compiler/parser.gaut` 스텁 고정값(`decls=0`) 제거: 입력 길이(`str_len`)를 Program 메타로 반영해 입력 의존적인 결정적 출력으로 전환.
- [ ] (1)~(5) Gaut 컴파일러 본체(실제 AST/파서/타입체커/C 생성기) 구현.
- [ ] (7) strict self-host 고정점 달성.

추가 진행 필요 항목
1. 현재 `parser/check/emit`가 스텁이므로, 다음 피처 단위는 (2) AST 실체화 + (3) 토크나이저/표현식 파서 이식으로 묶어 진행.
2. 그 다음 (4) 타입체커 최소 규칙 이식 후, (5) C 생성기를 Rust `crates/cgen` subset과 동등하게 확장.
3. 마지막으로 `SELF_HOST_COMPILER_STRICT=1` 경로에서 stage0/1/2 hash 고정점을 CI 스크립트로 고정.

전제(현재 상태)
- `compiler/*.gaut`는 AST/파서/타입체커/cgen/main이 스텁이며, 토큰화/파싱에 필요한 문자열/바이트 조작 프리미티브가 부족하다.
- self-host 루프는 stage0→stage1→stage2 비교/빌드 경로가 준비되어 있으나, 컴파일러가 유효한 C를 내지 못해 stage2 빌드가 실패한다.

작업 계획(다음 단계)

1) 컴파일러 구현을 가능하게 하는 최소 프리미티브 추가(언어/런타임/백엔드 공통)
   - 목표: Gaut 코드에서 “문자열을 바이트 단위로 스캔하고 토큰을 만들 수 있는” 최소 기능 확보.
   - 제안 빌트인(최소 세트)
     - `str_len(s: Str) -> i32`
     - `str_byte_at(s: Str, i: i32) -> u8` (범위 밖은 0 또는 에러 규약 중 택1)
     - `bytes_len(b: Bytes) -> i32`
     - `bytes_push(b: Bytes, x: u8) -> Bytes` (불변 모델이면 새 Bytes 반환)
     - `bytes_slice(b: Bytes, start: i32, len: i32) -> Bytes`
     - `bytes_to_str(b: Bytes) -> Str` (이미 있음: 런타임/코드젠/타입체커/인터프리터까지 일관성 유지)
   - 구현 범위
     - C 런타임: `runtime/c/runtime.{h,c}`에 함수 추가(+ 규약 문서화)
     - Rust cgen: 위 빌트인에 대한 shim/호출 매핑 추가
     - Rust typecheck: 위 빌트인 시그니처 + 필요한 타입(`Bytes`/`u8`) 일치 규칙 확인
     - Rust interpreter: 동일 빌트인 구현(테스트용)
     - std: `std/str.gaut`, `std/bytes.gaut`에 thin wrapper 제공(컴파일러에서 사용)
   - 검증
     - Rust 단위테스트: 각 빌트인 동작(경계값 포함)
     - cgen 테스트: 빌트인 호출이 runtime 심볼로 매핑되는지 확인

2) `compiler/ast.gaut` 실제 이식
   - 목표: Rust `frontend::ast`와 1:1 대응되는 데이터 모델 확보(최소한 컴파일러가 필요로 하는 subset부터).
   - 포함 항목(우선순위)
     - Program/Decl(Func/Type/Import/Let/Global)
     - Expr(리터럴/경로/호출/if/블록/레코드/단항/이항/ref/copy)
     - Type(Named/Ref/Record) 및 Path/Ident
   - 검증
     - 간단한 AST 생성/패턴매칭 스모크(컴파일러 내부 테스트용 예제)

3) `compiler/parser.gaut` (토큰화 + 우선순위 파서)
   - 토크나이저
     - 공백/주석/식별자/키워드/정수/문자열 리터럴/기호 토큰 정의
     - 에러 리포팅(오프셋/라인/컬럼 또는 최소 오프셋)
   - 파서
     - 최상위: import/type/func/let/global
     - 표현식: precedence climbing(단항/곱/합/비교/논리), 호출, 레코드 리터럴, 블록, if-then-else, ref/copy
   - 결정성
     - 파서 자체는 결정적이어야 하며, 에러 메시지도 가능한 한 안정적(동일 입력이면 동일 출력)
   - 검증
     - `examples/*.gaut`의 subset을 파싱하는 스모크(일단 컴파일러 소스의 파싱부터 목표)

4) `compiler/typecheck.gaut` (최소 타입체커 + move/escape/블록 수명)
   - 단계적 구현(“통과 가능한 최소”부터)
     - 기본 타입 검사(i32/bool/Str/Bytes/Unit, record, &T)
     - 함수 시그니처 수집 + 호출 타입 검사
     - block/if 타입 합치기 규칙(현재 Rust typecheck와 맞추기)
   - move/escape
     - Rust `frontend/typecheck.rs`의 규칙을 기준으로 단계별 이식
   - 검증
     - 기존 Rust typecheck 테스트에 대응하는 Gaut-side 스모크(“성공 케이스/실패 케이스” 최소 1쌍씩)

5) `compiler/cgen.gaut` (C 생성)
   - 목표: 현재 Rust `crates/cgen`이 지원하는 subset과 동일한 C를 생성(또는 최소한 self-host 루프에 필요한 동치성 달성).
   - 포함 항목(우선순위)
     - 런타임 include/arena/scope 패턴
     - Str/Bytes concat 런타임 호출
     - record/field 접근(ref면 `->`, 값이면 `.`)
     - 빌트인 shim(println/print/read/write/args/bytes_to_str/try_* 등)
     - `main(argc, argv)` 및 `gaut_args_init` 호출
   - 결정성(필수)
     - 선언/필드/맵 순회는 정렬하여 출력(컴파일러 내부에서 항상 stable order 유지)
   - 검증
     - “동일 입력 2회 emit 시 해시 동일” 체크를 compiler self-host에도 적용

6) `compiler/main.gaut` (CLI + 파일 IO)
   - 목표: `gautc1`(Gaut 컴파일러)이 `--emit-c out.c <in.gaut>` 형태로 동작.
   - 필요한 기능
     - `args()`로 argv 읽기(뉴라인-join 포맷 해석: `std/bytes.gaut`의 `args_str()` 활용)
     - 간단한 argv 파서(`--emit-c`, 입력 파일 경로)
     - `try_read_file`/`try_write_file`로 오류 처리(실패 시 exit code/메시지 규약 결정)
   - 검증
     - `SELF_HOST_COMPILER=1 ./scripts/self_host.sh`에서 stage1 C가 실제로 생성되는지 확인

7) self-host strict 고정점 달성
   - 목표: `SELF_HOST_COMPILER_STRICT=1 SELF_HOST_COMPILER=1 ./scripts/self_host.sh`가 “불일치 시 실패, 일치 시 통과”.
   - 체크 포인트
     - stage0(=Rust) vs stage1(=gautc1) C hash 일치
     - stage1 C로 빌드한 gautc2가 stage2 C를 생성
     - stage1(=gautc1) vs stage2(=gautc2) C hash 일치

리스크/의사결정 포인트
- “문자열/바이트 프리미티브” 규약이 컴파일러 구현 난이도를 좌우한다: 가능한 한 작게 시작하고, 부족하면 단계적으로 확장한다.
- `args()` 포맷(현재: newline join)을 표준으로 문서화할지, 향후 배열/리스트 타입 도입 전 임시 규약으로 둘지 결정이 필요하다.
- IO 에러 처리: `read_file()->Str`의 기존 규약(실패 시 빈 문자열)은 유지하되, 컴파일러 구현에는 `try_read_file`/`try_write_file`를 사용해 오류를 구분한다.

완료 기준(Definition of Done)
- `compiler/main.gaut`가 입력 파일을 읽고, 실제 C를 `--emit-c`로 출력한다.
- `SELF_HOST_COMPILER=1`에서 stage1 C가 유효한 C이며, stage2 컴파일러를 빌드할 수 있다.
- `SELF_HOST_COMPILER_STRICT=1`에서 stage0→stage1→stage2 해시가 모두 일치한다.
