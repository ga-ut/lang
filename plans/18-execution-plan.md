실행 요소 분리 및 우선 순위

1) Bytes 프리미티브 확장 (독립)
   - 런타임/C, Rust cgen/typecheck/interp, std/bytes.gaut에 bytes_len/bytes_push/bytes_slice 추가
   - 파서 구현 전에 완료 가능

2) AST 이식 (독립)
   - compiler/ast.gaut에 Rust frontend::ast 대응 구조 정의
   - 파서/타입체커와 독립적으로 진행 가능

3) 토크나이저 구현 (부분 독립)
   - bytes_* 프리미티브 기반으로 lexer 구현
   - 파서와는 인터페이스만 맞으면 독립 구현 가능

4) 파서 구현 (lexer 의존)
   - precedence parser, 표현식/선언 파싱

5) 타입체커 구현 (AST/파서 의존)
   - 타입 규칙, move/escape/블록 수명 검사

6) Cgen 구현 (AST/타입체커 의존)
   - 런타임 shim 및 결정적 C 출력

7) 컴파일 파이프라인 연결 (AST/parser/typecheck/cgen 의존)
   - compiler/main.gaut에서 read_file -> parse -> typecheck -> cgen -> write_file
   - --emit-c/--build 처리 확정

8) self-host strict 검증 (전체 의존)
   - stage0/1/2 해시 비교 경로 통과
   - 실패 시 로그/진단 보강

9) 테스트/문서 정리 (병행 가능)
   - 각 단계별 테스트 추가, 결정성 검증 강화
   - docs/lang-spec.md 또는 plans 문서에 규약 업데이트
