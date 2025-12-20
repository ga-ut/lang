다음 단계 계획

목표: Gaut 컴파일러가 실제로 입력 .gaut를 C로 생성하고 self-host strict 고정점 검증을 통과하도록 만든다.

1) 최소 프리미티브 확장(파서 구현 기반)
   - 런타임/C: bytes_len, bytes_push, bytes_slice 추가
   - Rust cgen/typecheck/interp: 동일 빌트인 시그니처/동작 매핑
   - std/bytes.gaut: thin wrapper 제공
   - 테스트: 경계값 포함 단위 테스트 추가

2) compiler/ast.gaut 이식
   - Rust frontend::ast와 1:1 대응되는 데이터 구조 정의
   - 우선순위: Program/Decl, Expr, Type, Path/Ident

3) compiler/parser.gaut 구현
   - 토크나이저: 공백/주석/식별자/키워드/정수/문자열/기호
   - 파서: precedence climbing, 호출/블록/if/레코드/ref/copy
   - 에러 리포팅: 최소 오프셋 기준
   - 결정성: 동일 입력은 동일 출력

4) compiler/typecheck.gaut 구현
   - 기본 타입 검사(i32/bool/Str/Bytes/Unit, record, &T)
   - 함수 시그니처 수집 + 호출 타입 검사
   - 블록/if 타입 합치기 규칙
   - move/escape/블록 수명 규칙 단계적 이식

5) compiler/cgen.gaut 구현
   - 런타임 include/arena/scope 패턴 적용
   - Str/Bytes concat, record/field 접근, 빌트인 shim 처리
   - 출력 결정성 확보(정렬 유지)

6) compiler/main.gaut 파이프라인 연결
   - read_file -> parse -> typecheck -> cgen -> write_file
   - --emit-c/--build 규약 확정 및 오류 처리(try_* 기반)

7) self-host strict 고정점 검증
   - SELF_HOST_COMPILER=1 경로에서 stage1 C 생성 확인
   - SELF_HOST_COMPILER_STRICT=1에서 stage0/1/2 해시 일치 확인

8) 테스트/문서 정리
   - cgen 골든 테스트에 record/ref/Bytes concat 추가
   - 스모크 예제 및 self_host 스크립트 체크 항목 확장
   - docs/lang-spec.md 또는 plans 문서에 규약 업데이트
