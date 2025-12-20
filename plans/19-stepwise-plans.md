순차 작업별 실행 계획

1) compiler/parser.gaut 토크나이저 구현
   - 목표: 문자열/바이트 기반으로 토큰 시퀀스를 생성하는 최소 lexer 제공
   - 작업:
     - bytes/str 유틸 래퍼 사용 규약 확정(str_len/str_byte_at/str_slice/bytes_* 사용)
     - 토큰 구조(종류/리터럴/오프셋) 정의
     - 공백/주석/식별자/키워드/숫자/문자열/기호 처리
     - 에러 규약(오프셋 기반) 및 테스트용 스모크 입력 추가
   - 완료 기준:
     - 최소 입력(example subset)에서 토큰 배열 생성 가능

2) compiler/parser.gaut 파서 구현
   - 목표: AST를 생성하는 precedence parser 제공
   - 작업:
     - Parser 상태(현재 토큰/peek)와 유틸 정의
     - 최상위 decl 파싱(import/type/global/let/func)
     - 표현식 파싱: unary/binary precedence, if, block, record, call, ref/copy
     - 경로/타입 파싱 및 AST 연결
     - 결정성 확보(동일 입력 동일 AST)
   - 완료 기준:
     - 주요 예제 및 compiler 소스 파싱 통과

3) compiler/typecheck.gaut 구현
   - 목표: 기본 타입/무브/스코프 규칙 검사
   - 작업:
     - 타입 환경/스코프/함수 시그니처 수집
     - 기본 타입/레코드/&T/Unit 검사
     - block/if 타입 합치기 규칙 구현
     - move/escape 규칙 최소 이식
   - 완료 기준:
     - 성공/실패 케이스 최소 스모크 통과

4) compiler/cgen.gaut 구현
   - 목표: Rust cgen과 동등한 C 출력 생성
   - 작업:
     - 런타임 include/shim/arena 패턴 이식
     - Str/Bytes concat, record field 접근, builtin shim 처리
     - 출력 결정성(정렬) 확보
   - 완료 기준:
     - 간단 입력에서 C 출력 생성 + 안정적 해시

5) compiler/main.gaut 파이프라인 연결
   - 목표: gautc1이 실제로 --emit-c 경로 동작
   - 작업:
     - read_file -> parse -> typecheck -> cgen -> write_file 연결
     - --emit-c/--build 처리 규약 확정
     - 오류 처리(try_* 기반) 결정
   - 완료 기준:
     - gautc1이 입력 파일을 읽고 C를 생성

6) self-host strict 검증 + 테스트 보강
   - 목표: stage0/1/2 해시 고정점 통과
   - 작업:
     - self_host.sh strict 경로 확인
     - cgen/test 스모크 추가
     - 문서/규약 업데이트
   - 완료 기준:
     - SELF_HOST_COMPILER_STRICT=1 경로 통과
