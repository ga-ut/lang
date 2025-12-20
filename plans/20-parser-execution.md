파서 작업 분리(독립 실행 단위) 및 우선순위

1) Token stream/Parser 상태 구조 도입
   - Token 리스트 표현(연결 리스트 또는 Bytes 기반 버퍼) 확정
   - Parser 상태(peek/next/expect) 유틸 정의

2) 기본 AST 빌더 유틸 추가
   - Ident/Path/Type/Expr/Stmt/Decl 생성 헬퍼
   - 연결 리스트 append 헬퍼

3) 타입 파서 구현
   - Named/Ref/Record 타입 파싱
   - FieldType 리스트 빌드

4) 표현식 파서 구현
   - literal/path/call/record/block/if/ref/copy
   - unary/binary precedence 파싱

5) 선언/함수 파서 구현
   - import/type/global/let/func 파싱
   - param 리스트/return 타입 처리

6) 오류/스모크 검증
   - 오프셋 기반 에러 리포팅 규약 고정
   - 최소 스모크 입력 파싱 확인
