0. 부트스트랩 전략 (Region/Context 제거 기준 반영)

- pre: Rust로 인터프리터/타입체커/초기 트랜스파일러 작성. `#![forbid(unsafe_code)]` 유지. 이유: borrow checker로 블록 수명 밖으로의 참조/이동을 막고, OOB를 테스트/디버그에서 조기 포착.
- 런타임 메모리: 언어 사용자에게는 "블록 {} = 유일한 수명"만 보이게 하고, 내부 구현은 함수/블록 단위의 지역 아레나(또는 스택 프레임)로 모델링. Region 개념은 컴파일러 내부에만 남겨 두고 공개 문법에는 없음.
- 외부 의존: 네트워크/IO는 Rust std 안전 API thin wrapper. C FFI는 필요한 최소 범위에서만 사용, 내부 아레나 포인터 외부 노출 금지.
- post: C 트랜스파일러가 안정되면 self-host(언어→C→clang/zig) 시도. 2차 이후에만 Region/Effect/Context 같은 고급 기능을 되살릴 후보로 두되, 1차 스펙과 코드에는 삽입하지 않는다.

⸻

1. 1차 언어 코어 스펙 (유지 개념만)

1-1. 문법 – 수식 스타일 / 최소 코어
- 코어: 값/변수, 가변/불변 바인딩, 함수, 조건식, 블록, 레코드(구조체), 참조(&T).
- 기본 문법: 세미콜론 없음. 블록은 `{}` 고정(추후 들여쓰기 설탕 검토). 함수는 마지막 식 반환.
- 예제:

  x: i32 = 10
  mut count: i64 = 0
  count = count + 1

  add(a: i32, b: i32) -> i32 =
    a + b

  max(a: i32, b: i32) -> i32 =
    if a > b then a else b

1-2. 타입 & 수명 규칙 (Region 제거 후)
- 사용자 문법에서 Region/Context 없음. 수명은 오직 블록 {} 스코프로 정의.
- 전역 수명은 `global` 키워드로만 명시.
- 참조는 `&T` 하나만 제공. 참조가 블록을 넘어가지 못하게 타입체커가 검사.
- 이동/복사: 기본은 move, 복사는 `copy`로만 허용. use-after-move 금지.

1-3. Module
- "한 파일 = 한 모듈". `import foo` → `foo.gaut` 로딩. 네임스페이스는 `foo.fn`, `foo.Type` 정도만. 접근제어/패키지/버전은 후속 과제.

1-4. Effect
- 개념은 문서에만 남기고, 1차 구현에서는 문법/체크 모두 비활성. 추후 IO/Alloc 추적, 순수 함수 최적화 때 재도입.

⸻

2. 메모리/실행 모델 (사용자 관점 최소화)

- 블록 수명: {} 내부에서 생성된 값/버퍼는 블록 종료 시 모두 소멸. 전역은 `global`로만 유지.
- 내부 구현: 각 함수/블록에 로컬 아레나(또는 스택) 하나를 두고 bump pointer로 할당, 블록 끝에서 offset reset. Region 이름/선언은 공개 문법에 없음.
- 수명 제약: 값/참조는 생성 블록 밖으로 반환 불가(전역은 예외). 타입체커가 블록 탈출 여부를 검사.
- OOB/초과 할당: Rust 부트스트랩에서 디버그/테스트 시 어설션으로 잡고, C 산출물에도 디버그 어설션 남김(릴리스에선 선택적 제거).

⸻

3. 구현 전략 (Rust 부트스트랩 → C 트랜스파일러)

3-1. Rust 부트스트랩
- core/mem: 블록/함수 단위 아레나 구현 + 안전 API, 단위테스트로 블록 밖 참조/할당 실패 케이스 검증.
- 파서: hand-written recursive descent, 우선순위 `(), unary, *, /, +, -, <, ==, &&, ||`. AST/Type 구조 정의.
- 타입체커: 타입 명시(추론 없음). move/copy/& 규칙, 블록 수명 규칙, use-after-move/dangling 검사. Effect는 무시.
- 인터프리터: 아레나로 메모리 흉내. cap 초과 시 에러를 터뜨려 조기 발견.

3-2. 1차 C 트랜스파일러
- 함수 호출에 숨은 region 인자 없음(사용자 개념이 아니므로). 함수/블록에 로컬 아레나를 생성/리셋하는 코드만 삽입.
- 기본 타입/참조/레코드를 C struct로 매핑, 문자열/바이트는 아레나 기반 헬퍼 사용.
- CI: clang-format + 빌드만 우선 확인.

3-3. std 최소 집합
- 문자열/Bytes, 구조체 조작, 간단 연산만. IO/네트워크는 러퍼 형태로 별도 모듈 제공(파일 단위 모듈 규칙 준수).

3-4. post (2차 이후 후보)
- Effect 재도입, Region-like 수명 구간(요청/태스크) 설탕, 접근제어/패키징, 간단 최적화(copy elision 등), self-host.

⸻

4. 로드맵 (1차 목표 한정)

단계 A. 콘솔 프로그램
- 입력 → 파싱 → "hello, {name}" + 간단 계산. 블록 수명/가변/불변/참조/이동-복사 규칙을 모두 사용.

단계 B. TCP 예제 (최소 러퍼 기반)
- Rust thin wrapper로 net_listen/accept/read/write 제공.
- 언어 측 예제:

  global greeting: Str = "hello"

  handle(conn) = {
    buf = Net.read(conn)
    name = Http.parse_name(buf)
    msg = greeting + ", " + name
    Net.write(conn, msg)
  }

  main() = {
    srv = Net.listen(8080)
    loop {
      conn = Net.accept(srv)
      { handle(conn) } // 블록 스코프로 수명 한정
    }
  }

- 트랜스파일 시 각 블록/함수에 로컬 아레나 생성·리셋만 삽입.

⸻

5. 작업 순서 (업데이트)
1) 1차 코어 문법 스펙 작성: 블록 수명, 가변/불변, 참조/이동/복사, 함수/모듈 문법을 문서로 고정.
2) Rust 부트스트랩: 파서 + AST + 타입체커 + 블록 아레나 기반 인터프리터.
3) 블록 수명/이동 규칙 테스트 강화: use-after-move, 블록 탈출 참조, OOB 등 property/단위 테스트.
4) C 트랜스파일러 스켈레톤: 함수/변수/반환 매핑 + 블록 아레나 생성/리셋 코드 삽입.
5) 최소 std(문자열/Bytes) + Net thin wrapper.
6) 콘솔/간단 TCP 예제까지 실행.

여기까지면: Region/Context/Effect 없이도 블록 수명 기반의 수식형 시스템 언어를 실행·트랜스파일까지 확인 가능.
